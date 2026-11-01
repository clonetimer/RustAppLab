//! Dependency/ABI readiness probe only. NOT NetFetch or ServiceDesk.
//! This source must be compiled on a connected machine before its behavior is considered verified.
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Health {
    pub status: String,
}
pub fn router() -> axum::Router {
    axum::Router::new().route(
        "/health",
        axum::routing::get(|| async {
            axum::Json(Health {
                status: "ready".into(),
            })
        }),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    #[tokio::test]
    async fn real_loopback_http_json_and_shutdown() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (stop, stopped) = tokio::sync::oneshot::channel::<()>();
        let task = tokio::spawn(async move {
            axum::serve(listener, router())
                .with_graceful_shutdown(async {
                    let _ = stopped.await;
                })
                .await
                .unwrap();
        });
        let client = reqwest::Client::builder()
            .no_proxy()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        let response = client
            .get(format!("http://{address}/health"))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), reqwest::StatusCode::OK);
        assert_eq!(
            response.json::<Health>().await.unwrap(),
            Health {
                status: "ready".into()
            }
        );
        stop.send(()).unwrap();
        tokio::time::timeout(Duration::from_secs(5), task)
            .await
            .unwrap()
            .unwrap();
    }
    #[tokio::test]
    async fn timeout_and_cancel_do_not_claim_completion() {
        assert!(
            tokio::time::timeout(Duration::from_millis(10), std::future::pending::<()>())
                .await
                .is_err()
        );
        let task = tokio::spawn(std::future::pending::<()>());
        task.abort();
        assert!(task.await.unwrap_err().is_cancelled());
    }
    #[test]
    fn json_roundtrip() {
        let value = Health {
            status: "本机验证".into(),
        };
        assert_eq!(
            serde_json::from_str::<Health>(&serde_json::to_string(&value).unwrap()).unwrap(),
            value
        );
    }
}
