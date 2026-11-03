//! C2 · Axum JSON routes, single-writer persisted state and graceful shutdown.
pub mod model;
mod handlers;
pub use model::{ApiError, AppState, Ticket};
use std::path::Path;
use axum::Router;
use tokio::{net::TcpListener, sync::watch};

/// C2 TODO(1): open a valid legacy journal and start a bounded, single-writer actor.
/// Disk I/O must not run on a Tokio reactor thread.
pub fn start_store(_path: &Path) -> Result<AppState, ApiError> {
    Err(ApiError::Lifecycle("C2 TODO: store actor".into()))
}
/// C2 TODO(2): attach all fixed handlers to an Axum Router with shared state.
pub fn router(_state: AppState) -> Router {
    Router::new()
}
/// C2 TODO(3): actually serve TCP HTTP and await graceful shutdown.
pub async fn serve(
    _listener: TcpListener,
    _app: Router,
    _shutdown: watch::Receiver<bool>,
) -> Result<(), ApiError> {
    Err(ApiError::Lifecycle("C2 TODO: tcp graceful shutdown".into()))
}
