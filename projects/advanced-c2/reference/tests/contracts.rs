use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
    routing::get,
    Router,
};
use servicedesk_c2::{router, serve, start_store, ApiError};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};
use tokio::{
    net::TcpListener,
    sync::{watch, Notify},
};
use tower::ServiceExt;

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "applab-c2-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn db(&self) -> PathBuf {
        self.0.join("tickets.db")
    }
    fn init(&self) -> PathBuf {
        let path = self.db();
        servicedesk::init(&path).unwrap();
        path
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
async fn oneshot(
    app: Router,
    method: &str,
    url: &str,
    body: &str,
    client: bool,
    content_type: bool,
) -> axum::response::Response {
    let mut builder = Request::builder().method(method).uri(url);
    if client {
        builder = builder.header("X-RustAppLab-Client", "1");
    }
    if content_type {
        builder = builder.header("content-type", "application/json");
    }
    app.oneshot(builder.body(Body::from(body.to_owned())).unwrap())
        .await
        .unwrap()
}
async fn payload(res: axum::response::Response) -> serde_json::Value {
    let bytes = to_bytes(res.into_body(), 16 * 1024).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}
async fn shutdown_server(
    tx: watch::Sender<bool>,
    task: tokio::task::JoinHandle<Result<(), ApiError>>,
) {
    tx.send(true).unwrap();
    tokio::time::timeout(Duration::from_secs(5), task)
        .await
        .expect("server did not shut down")
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn c2_1_requires_valid_existing_journal_and_never_resets_it() {
    let dir = Scratch::new();
    let path = dir.db();
    assert!(matches!(start_store(&path), Err(ApiError::Storage)));
    fs::write(&path, b"corrupt\n").unwrap();
    assert!(matches!(start_store(&path), Err(ApiError::Storage)));
    assert_eq!(fs::read(&path).unwrap(), b"corrupt\n");
    assert!(servicedesk::init(&path).is_err());
}
#[cfg(unix)]
#[tokio::test]
async fn c2_1_rejects_symlink_journal_without_following_it() {
    let dir=Scratch::new();
    let real=dir.init();
    let alias=dir.0.join("alias.db");
    std::os::unix::fs::symlink(&real,&alias).unwrap();
    assert!(matches!(start_store(&alias),Err(ApiError::Storage)));
    assert_eq!(servicedesk::load(&real).unwrap().len(),0);
}
#[tokio::test]
async fn c2_1_persists_create_close_and_restart() {
    let dir = Scratch::new();
    let db = dir.init();
    let app = start_store(&db).expect("C2 TODO(1): state actor");
    let first = app.create(" 初次任务 ".into()).await.unwrap();
    assert_eq!(
        (first.id, first.title.as_str(), first.done),
        (1, "初次任务", false)
    );
    let second = app.create("再次任务".into()).await.unwrap();
    assert_eq!(second.id, 2);
    let closed = app.close(1).await.unwrap();
    assert!(closed.done);
    drop(app);
    let reopened = start_store(&db).unwrap();
    let rows = reopened.list().await.unwrap();
    assert_eq!(rows.len(), 2);
    assert!(rows[0].done);
    assert_eq!(rows[1].title, "再次任务");
    assert_eq!(servicedesk::load(&db).unwrap().len(), 2);
}
#[tokio::test]
async fn c2_1_preserves_domain_validation_and_notfound() {
    let dir = Scratch::new();
    let db = dir.init();
    let app = start_store(&db).unwrap();
    assert!(matches!(
        app.create(" ".into()).await,
        Err(ApiError::Invalid(_))
    ));
    assert!(matches!(
        app.create("x\nbad".into()).await,
        Err(ApiError::Invalid(_))
    ));
    assert!(matches!(app.close(1).await, Err(ApiError::NotFound)));
    let ticket = app.create("good".into()).await.unwrap();
    app.close(ticket.id).await.unwrap();
    assert!(matches!(
        app.close(ticket.id).await,
        Err(ApiError::Conflict)
    ));
    assert_eq!(app.list().await.unwrap().len(), 1);
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn c2_1_concurrent_actor_updates_are_unique() {
    let dir = Scratch::new();
    let db = dir.init();
    let app = start_store(&db).unwrap();
    let mut tasks = tokio::task::JoinSet::new();
    for i in 0..48 {
        let app = app.clone();
        tasks.spawn(async move { app.create(format!("task-{i}")).await.unwrap() });
    }
    let mut ids = Vec::new();
    while let Some(value) = tasks.join_next().await {
        ids.push(value.unwrap().id);
    }
    ids.sort_unstable();
    assert_eq!(ids, (1..=48).collect::<Vec<_>>());
    assert_eq!(app.list().await.unwrap().len(), 48);
    assert_eq!(servicedesk::load(&db).unwrap().len(), 48);
}

#[tokio::test]
async fn c2_2_health_and_static_assets_are_real_routes() {
    let dir = Scratch::new();
    let db = dir.init();
    let app = router(start_store(&db).unwrap());
    let r = oneshot(app.clone(), "GET", "/health", "", false, false).await;
    assert_eq!(r.status(), StatusCode::OK);
    assert_eq!(payload(r).await["ok"], true);
    let page = oneshot(app.clone(), "GET", "/", "", false, false).await;
    assert_eq!(page.status(), StatusCode::OK);
    assert!(page.headers().get("content-security-policy").is_some());
    let bytes = to_bytes(page.into_body(), 20 * 1024).await.unwrap();
    assert!(String::from_utf8_lossy(&bytes).contains("ServiceDesk"));
    for url in ["/assets/client.js", "/assets/app.css"] {
        assert_eq!(
            oneshot(app.clone(), "GET", url, "", false, false)
                .await
                .status(),
            StatusCode::OK
        );
    }
    assert_eq!(
        oneshot(app, "GET", "/nonesuch", "", false, false)
            .await
            .status(),
        StatusCode::NOT_FOUND
    );
}
#[tokio::test]
async fn c2_2_json_create_list_close_and_conflict() {
    let dir = Scratch::new();
    let db = dir.init();
    let app = router(start_store(&db).unwrap());
    let response = oneshot(
        app.clone(),
        "POST",
        "/api/tickets",
        r#"{"title":"hi <script>"}"#,
        true,
        true,
    )
    .await;
    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(payload(response).await["title"], "hi <script>");
    let result = oneshot(app.clone(), "GET", "/api/tickets", "", false, false).await;
    assert_eq!(result.status(), StatusCode::OK);
    let body = payload(result).await;
    assert_eq!(body["count"], 1);
    assert_eq!(body["tickets"][0]["id"], 1);
    let closed = oneshot(app.clone(), "POST", "/api/tickets/1/close", "", true, false).await;
    assert_eq!(closed.status(), StatusCode::OK);
    assert_eq!(payload(closed).await["done"], true);
    assert_eq!(
        oneshot(app.clone(), "POST", "/api/tickets/1/close", "", true, false)
            .await
            .status(),
        StatusCode::CONFLICT
    );
    assert_eq!(
        oneshot(app, "POST", "/api/tickets/999/close", "", true, false)
            .await
            .status(),
        StatusCode::NOT_FOUND
    );
}
#[tokio::test]
async fn c2_2_missing_client_header_denies_mutation() {
    let dir = Scratch::new();
    let db = dir.init();
    let app = router(start_store(&db).unwrap());
    assert_eq!(
        oneshot(
            app.clone(),
            "POST",
            "/api/tickets",
            r#"{"title":"bad"}"#,
            false,
            true
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        oneshot(
            app.clone(),
            "POST",
            "/api/tickets/1/close",
            "",
            false,
            false
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
    assert!(servicedesk::load(&db).unwrap().is_empty());
}
#[tokio::test]
async fn c2_2_invalid_json_body_limits_and_schema() {
    let dir = Scratch::new();
    let db = dir.init();
    let app = router(start_store(&db).unwrap());
    for body in ["{", r#"{"title":"ok","admin":true}"#] {
        let r = oneshot(app.clone(), "POST", "/api/tickets", body, true, true).await;
        assert!(!r.status().is_success());
    }
    let r = oneshot(
        app.clone(),
        "POST",
        "/api/tickets",
        &format!(r#"{{"title":"{}"}}"#, "x".repeat(3000)),
        true,
        true,
    )
    .await;
    assert_eq!(r.status(), StatusCode::PAYLOAD_TOO_LARGE);
    let r = oneshot(
        app.clone(),
        "POST",
        "/api/tickets",
        r#"{"title":"ok"}"#,
        true,
        false,
    )
    .await;
    assert!(!r.status().is_success());
    assert_eq!(
        payload(oneshot(app, "GET", "/api/tickets", "", false, false).await).await["count"],
        0
    );
}
#[tokio::test]
async fn c2_2_cross_origin_preflight_is_not_enabled() {
    let dir = Scratch::new();
    let db = dir.init();
    let app = router(start_store(&db).unwrap());
    let r = oneshot(app.clone(), "OPTIONS", "/api/tickets", "", false, false).await;
    assert!(!r.status().is_success());
    assert!(r.headers().get("access-control-allow-origin").is_none());
    let request = Request::builder()
        .method("POST")
        .uri("/api/tickets")
        .header("X-RustAppLab-Client", "1")
        .header("content-type", "application/json")
        .header("host", "127.0.0.1:1234")
        .header("origin", "http://attacker.example")
        .body(Body::from(r#"{"title":"bad"}"#))
        .unwrap();
    let r = app.oneshot(request).await.unwrap();
    assert_eq!(r.status(), StatusCode::FORBIDDEN);
    assert!(servicedesk::load(&db).unwrap().is_empty());
}

#[tokio::test]
async fn c2_3_real_http_and_restart() {
    let dir = Scratch::new();
    let db = dir.init();
    let state = start_store(&db).unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = watch::channel(false);
    let server = tokio::spawn(serve(listener, router(state), rx));
    let client = reqwest::Client::builder().no_proxy().build().unwrap();
    let r = client.get(format!("{base}/health")).send().await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let r = client
        .post(format!("{base}/api/tickets"))
        .header("X-RustAppLab-Client", "1")
        .json(&serde_json::json!({"title":"real request"}))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    shutdown_server(tx, server).await;
    let rows = servicedesk::load(&db).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].title, "real request");
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn c2_3_network_concurrent_requests_persist_without_losses() {
    let dir = Scratch::new();
    let db = dir.init();
    let state = start_store(&db).unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = watch::channel(false);
    let server = tokio::spawn(serve(listener, router(state), rx));
    let client = reqwest::Client::builder().no_proxy().build().unwrap();
    let mut jobs = tokio::task::JoinSet::new();
    for i in 0..48 {
        let client = client.clone();
        let url = format!("{base}/api/tickets");
        jobs.spawn(async move {
            let r = client
                .post(url)
                .header("X-RustAppLab-Client", "1")
                .json(&serde_json::json!({"title":format!("work-{i}")}))
                .send()
                .await
                .unwrap();
            assert_eq!(r.status(), StatusCode::CREATED);
            r.json::<serde_json::Value>().await.unwrap()["id"]
                .as_u64()
                .unwrap()
        });
    }
    let mut ids = Vec::new();
    while let Some(result) = jobs.join_next().await {
        ids.push(result.unwrap());
    }
    ids.sort_unstable();
    assert_eq!(ids, (1..=48).collect::<Vec<_>>());
    shutdown_server(tx, server).await;
    assert_eq!(servicedesk::load(&db).unwrap().len(), 48);
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn c2_3_graceful_shutdown_drains_inflight_request() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/slow", listener.local_addr().unwrap());
    let entered = std::sync::Arc::new(Notify::new());
    let release = std::sync::Arc::new(Notify::new());
    let e = entered.clone();
    let r = release.clone();
    let app = Router::new().route(
        "/slow",
        get(move || {
            let e = e.clone();
            let r = r.clone();
            async move {
                e.notify_one();
                r.notified().await;
                "finished"
            }
        }),
    );
    let (tx, rx) = watch::channel(false);
    let server = tokio::spawn(serve(listener, app, rx));
    let call = tokio::spawn(async move {
        reqwest::Client::builder()
            .no_proxy()
            .build()
            .unwrap()
            .get(url)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap()
    });
    tokio::time::timeout(Duration::from_secs(3), entered.notified())
        .await
        .unwrap();
    tx.send(true).unwrap();
    tokio::time::sleep(Duration::from_millis(60)).await;
    assert!(
        !server.is_finished(),
        "graceful shutdown killed an inflight request"
    );
    release.notify_one();
    assert_eq!(call.await.unwrap(), "finished");
    tokio::time::timeout(Duration::from_secs(3), server)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
}
#[tokio::test]
async fn c2_3_pre_cancelled_server_quits_without_accepting() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let (_tx, rx) = watch::channel(true);
    tokio::time::timeout(Duration::from_secs(3), serve(listener, Router::new(), rx))
        .await
        .unwrap()
        .unwrap();
}
#[test]
fn c2_3_cli_init_refuses_overwrite_and_help_is_readable() {
    let dir = Scratch::new();
    let db = dir.db();
    let exe = env!("CARGO_BIN_EXE_servicedesk-c2");
    let output = std::process::Command::new(exe)
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("loopback Axum"));
    let first = std::process::Command::new(exe)
        .arg("init")
        .arg(&db)
        .output()
        .unwrap();
    assert!(
        first.status.success(),
        "{:?}",
        String::from_utf8_lossy(&first.stderr)
    );
    let old = fs::read(&db).unwrap();
    let second = std::process::Command::new(exe)
        .arg("init")
        .arg(&db)
        .output()
        .unwrap();
    assert!(!second.status.success());
    assert_eq!(fs::read(db).unwrap(), old);
}
