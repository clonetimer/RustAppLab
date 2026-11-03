//! C2 · true Axum HTTP/JSON with a single serialized, persistent business actor.
//! Bound to 127.0.0.1 by the CLI; API clients must not treat the header as authentication.
mod handlers;
pub mod model;
pub use model::{ApiError, AppState, Ticket};

use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
};
use model::Command;
use std::path::Path;
use tokio::{
    net::TcpListener,
    sync::{mpsc, watch},
};

/// Reuse the existing ServiceDesk journal and its validation rules. All synchronous
/// disk work is done inside the dedicated blocking actor, not a Tokio I/O worker.
pub fn start_store(path: &Path) -> Result<AppState, ApiError> {
    servicedesk::ensure_store(path).map_err(|_| ApiError::Storage)?;
    let mut tickets = servicedesk::load(path).map_err(|_| ApiError::Storage)?;
    let path = path.to_path_buf();
    let (tx, mut rx) = mpsc::channel::<Command>(64);
    let runtime =
        tokio::runtime::Handle::try_current().map_err(|e| ApiError::Lifecycle(e.to_string()))?;
    runtime.spawn_blocking(move || {
        let mut failed = false;
        while let Some(command) = rx.blocking_recv() {
            match command {
                Command::List(reply) => {
                    let result = if failed {
                        Err(ApiError::Storage)
                    } else {
                        Ok(tickets.iter().cloned().map(Ticket::from).collect())
                    };
                    let _ = reply.send(result);
                }
                Command::Create(title, reply) => {
                    let result = if failed {
                        Err(ApiError::Storage)
                    } else {
                        servicedesk::persist(&path, servicedesk::Action::Add(title))
                            .map(Ticket::from)
                            .map_err(ApiError::domain)
                    };
                    if let Ok(ref ticket) = result {
                        tickets.push(servicedesk::Ticket {
                            id: ticket.id,
                            title: ticket.title.clone(),
                            done: ticket.done,
                        });
                    }
                    if matches!(result, Err(ApiError::Storage)) {
                        failed = true;
                    }
                    let _ = reply.send(result);
                }
                Command::Close(id, reply) => {
                    let result = if failed {
                        Err(ApiError::Storage)
                    } else {
                        servicedesk::persist(&path, servicedesk::Action::Close(id))
                            .map(Ticket::from)
                            .map_err(ApiError::domain)
                    };
                    if let Ok(ref ticket) = result {
                        if let Some(original) = tickets.iter_mut().find(|row| row.id == ticket.id) {
                            original.done = ticket.done;
                        } else {
                            failed = true;
                        }
                    }
                    if matches!(result, Err(ApiError::Storage)) {
                        failed = true;
                    }
                    let _ = reply.send(if failed {
                        Err(ApiError::Storage)
                    } else {
                        result
                    });
                }
            }
        }
    });
    Ok(AppState { tx })
}

/// Attach all endpoints while keeping the business core and HTML assets fixed.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(handlers::index))
        .route("/assets/client.js", get(handlers::script))
        .route("/assets/app.css", get(handlers::stylesheet))
        .route("/health", get(handlers::health))
        .route("/api/tickets", get(handlers::list).post(handlers::create))
        .route("/api/tickets/{id}/close", post(handlers::close))
        .layer(DefaultBodyLimit::max(2048))
        .with_state(state)
}

/// Accept real TCP connections and finish in-flight work on a true cancellation signal.
/// Dropping the false watch sender does not spuriously stop the service or spin.
pub async fn serve(
    listener: TcpListener,
    app: Router,
    mut shutdown: watch::Receiver<bool>,
) -> Result<(), ApiError> {
    let stop = async move {
        loop {
            if *shutdown.borrow_and_update() {
                return;
            }
            if shutdown.changed().await.is_err() {
                std::future::pending::<()>().await;
            }
        }
    };
    axum::serve(listener, app)
        .with_graceful_shutdown(stop)
        .await
        .map_err(|e| ApiError::Lifecycle(e.to_string()))
}
