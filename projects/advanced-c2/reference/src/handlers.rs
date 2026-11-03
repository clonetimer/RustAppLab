//! Fixed teaching infrastructure: learners implement state ownership, route registration and serve.
use crate::{ApiError, AppState, Ticket};
use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};

const HTML: &str = include_str!("../assets/index.html");
const JS: &str = include_str!("../assets/client.js");
const CSS: &str = include_str!("../assets/app.css");

pub async fn index() -> ([(header::HeaderName, &'static str); 2], &'static str) {
    ([
        (header::CONTENT_TYPE, "text/html; charset=utf-8"),
        (header::CONTENT_SECURITY_POLICY, "default-src 'none'; script-src 'self'; connect-src 'self'; style-src 'self'; base-uri 'none'; form-action 'self'"),
    ], HTML)
}
pub async fn script() -> ([(header::HeaderName, &'static str); 2], &'static str) {
    ([(header::CONTENT_TYPE, "application/javascript; charset=utf-8"), (header::X_CONTENT_TYPE_OPTIONS, "nosniff")], JS)
}
pub async fn stylesheet() -> ([(header::HeaderName, &'static str); 2], &'static str) {
    ([(header::CONTENT_TYPE, "text/css; charset=utf-8"), (header::X_CONTENT_TYPE_OPTIONS, "nosniff")], CSS)
}
pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok":true,"service":"C2"}))
}
#[derive(Serialize)]
pub struct TicketList {
    pub tickets: Vec<Ticket>,
    pub count: usize,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateTicket {
    pub title: String,
}
pub async fn list(State(state): State<AppState>) -> Result<Json<TicketList>, ApiError> {
    let tickets = state.list().await?;
    Ok(Json(TicketList { count: tickets.len(), tickets }))
}
pub fn validate_write(headers: &HeaderMap) -> Result<(), ApiError> {
    if headers.get("x-rustapplab-client").and_then(|v| v.to_str().ok()) != Some("1") {
        return Err(ApiError::Forbidden);
    }
    if let Some(origin) = headers.get(header::ORIGIN) {
        let host = headers.get(header::HOST).and_then(|x| x.to_str().ok())
            .ok_or(ApiError::Forbidden)?;
        if origin.to_str().ok() != Some(format!("http://{host}").as_str()) {
            return Err(ApiError::Forbidden);
        }
    }
    Ok(())
}
pub async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CreateTicket>,
) -> Result<(StatusCode, Json<Ticket>), ApiError> {
    validate_write(&headers)?;
    let result = state.create(body.title).await?;
    Ok((StatusCode::CREATED, Json(result)))
}
pub async fn close(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<u64>,
) -> Result<Json<Ticket>, ApiError> {
    validate_write(&headers)?;
    Ok(Json(state.close(id).await?))
}
