use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Ticket {
    pub id: u64,
    pub title: String,
    pub done: bool,
}
impl From<servicedesk::Ticket> for Ticket {
    fn from(value: servicedesk::Ticket) -> Self {
        Self { id: value.id, title: value.title, done: value.done }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiError {
    Invalid(String),
    Forbidden,
    NotFound,
    Conflict,
    Storage,
    Unavailable,
    Lifecycle(String),
}
impl ApiError {
    pub fn domain(message: String) -> Self {
        if message.contains("编号不存在") {
            Self::NotFound
        } else if message.contains("已关闭") {
            Self::Conflict
        } else if message.contains("标题") || message.contains("最多 1000") || message.contains("编号耗尽") {
            Self::Invalid(message)
        } else {
            Self::Storage
        }
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::Invalid(message) => (StatusCode::UNPROCESSABLE_ENTITY, message),
            Self::Forbidden => (StatusCode::FORBIDDEN, "写请求必须来自本地页面或显式客户端".into()),
            Self::NotFound => (StatusCode::NOT_FOUND, "工单编号不存在".into()),
            Self::Conflict => (StatusCode::CONFLICT, "工单已关闭".into()),
            Self::Storage => (StatusCode::SERVICE_UNAVAILABLE, "持久化存储不可用或记录已损坏".into()),
            Self::Unavailable => (StatusCode::SERVICE_UNAVAILABLE, "单写者已关闭".into()),
            Self::Lifecycle(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
        };
        (status, Json(serde_json::json!({"error": message}))).into_response()
    }
}
impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for ApiError {}

pub(crate) enum Command {
    List(oneshot::Sender<Result<Vec<Ticket>, ApiError>>),
    Create(String, oneshot::Sender<Result<Ticket, ApiError>>),
    Close(u64, oneshot::Sender<Result<Ticket, ApiError>>),
}
#[derive(Clone)]
pub struct AppState {
    pub(crate) tx: mpsc::Sender<Command>,
}
impl AppState {
    pub async fn list(&self) -> Result<Vec<Ticket>, ApiError> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(Command::List(tx)).await.map_err(|_| ApiError::Unavailable)?;
        rx.await.map_err(|_| ApiError::Unavailable)?
    }
    pub async fn create(&self, title: String) -> Result<Ticket, ApiError> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(Command::Create(title, tx)).await.map_err(|_| ApiError::Unavailable)?;
        rx.await.map_err(|_| ApiError::Unavailable)?
    }
    pub async fn close(&self, id: u64) -> Result<Ticket, ApiError> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(Command::Close(id, tx)).await.map_err(|_| ApiError::Unavailable)?;
        rx.await.map_err(|_| ApiError::Unavailable)?
    }
}
