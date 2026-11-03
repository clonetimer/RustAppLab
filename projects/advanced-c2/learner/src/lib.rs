//! C2 学员代码。只需要完成三个核心函数；也可以自行增加 Rust 模块。
pub mod model;
mod handlers;
pub use model::{ApiError, AppState, Ticket};
use axum::Router;
use std::path::Path;
use tokio::{net::TcpListener, sync::watch};

/// TODO(1): 加载有效的旧日志，启动 bounded mpsc + spawn_blocking 单写者。
pub fn start_store(_path: &Path) -> Result<AppState, ApiError> {
    Err(ApiError::Lifecycle("C2 TODO: store actor".into()))
}
/// TODO(2): 注册所有 Axum handler，绑定共享状态及 2048 字节正文上限。
pub fn router(_state: AppState) -> Router {
    Router::new()
}
/// TODO(3): axum::serve 实际 TCP，watch true 时 graceful shutdown。
pub async fn serve(
    _listener: TcpListener,
    _app: Router,
    _shutdown: watch::Receiver<bool>,
) -> Result<(), ApiError> {
    Err(ApiError::Lifecycle("C2 TODO: tcp graceful shutdown".into()))
}
