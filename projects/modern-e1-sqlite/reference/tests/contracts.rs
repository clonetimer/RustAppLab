use axum::http::{Method, StatusCode};
use e1_secure_lab::{Credentials, TlsListener, server_config, serve};
use e1_sqlite_lab::*;
use rusqlite::{Connection, params};
use tokio::{net::TcpListener, sync::{mpsc, oneshot}, task::JoinSet};
mod support;
use support::*;

fn strings(values: &[&str]) -> Vec<String> { values.iter().map(|v| (*v).to_owned()).collect() }

#[test]
fn step1_atomic_batch_and_audit_with_real_sqlite() {
    let directory = Temp::new();
    let path = directory.0.join("工单 数据.db");
    assert!(Store::open(&path).is_err());
    Store::init(&path).unwrap();
    assert!(Store::init(&path).is_err(), "init must not overwrite");
    assert!(std::fs::read(&path).unwrap().starts_with(b"SQLite format 3\0"));
    let mut db = Store::open(&path).unwrap();
    let literal = "中文 '); DROP TABLE tickets; --";
    let rows = db.create_batch(&strings(&[" first ", literal])).unwrap();
    assert_eq!(rows.iter().map(|t| t.id).collect::<Vec<_>>(), [1,2]);
    assert_eq!(rows[0].title, "first");
    assert_eq!(rows[1].title, literal);
    let events = db.audit().unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[1].operation, "create");
    assert_eq!(events[1].ticket_id, 2);
    assert_eq!(db.create_batch(&strings(&["would be inserted", "  "])), Err(Error::Invalid));
    assert_eq!(db.create_batch(&[]), Err(Error::Invalid));
    assert_eq!(db.create_batch(&vec!["too many".into(); 17]), Err(Error::Invalid));
    assert_eq!(db.list().unwrap(), rows);
    assert_eq!(db.audit().unwrap(), events);
    // An actual SQLite constraint failure on the audit insertion must roll back ticket insertion.
    let external = Connection::open(&path).unwrap();
    external.execute_batch("CREATE TRIGGER reject_create BEFORE INSERT ON audit_events
        WHEN NEW.operation='create' BEGIN SELECT RAISE(ABORT,'fixture failure'); END;").unwrap();
    assert_eq!(db.create_batch(&strings(&["audit fails"])), Err(Error::Storage));
    external.execute_batch("DROP TRIGGER reject_create").unwrap();
    drop(external); drop(db);
    let mut reopened = Store::open(&path).unwrap();
    assert_eq!(reopened.list().unwrap(), rows);
    assert_eq!(reopened.audit().unwrap(), events);
    assert_eq!(reopened.create_batch(&strings(&["after rollback"])).unwrap()[0].id, 3);
}

#[test]
fn step2_close_transaction_and_conflict_do_not_duplicate_audit() {
    let directory = Temp::new();
    let path = directory.0.join("tickets.db");
    Store::init(&path).unwrap();
    // Seed independently of TODO(1) so a learner can isolate TODO(2).
    let seed = Connection::open(&path).unwrap();
    seed.execute("INSERT INTO tickets(id,title,done) VALUES(?1,?2,0)", params![1,"seed"]).unwrap();
    seed.execute("INSERT INTO audit_events(operation,ticket_id) VALUES('create',1)", []).unwrap();
    seed.execute_batch("CREATE TRIGGER reject_close BEFORE INSERT ON audit_events
        WHEN NEW.operation='close' BEGIN SELECT RAISE(ABORT,'fixture failure'); END;").unwrap();
    let mut db = Store::open(&path).unwrap();
    assert_eq!(db.close(1), Err(Error::Storage));
    assert!(!db.list().unwrap()[0].done);
    assert_eq!(db.audit().unwrap().len(), 1);
    seed.execute_batch("DROP TRIGGER reject_close").unwrap();
    drop(seed);
    assert_eq!(db.close(99), Err(Error::NotFound));
    assert!(db.close(1).unwrap().done);
    assert_eq!(db.close(1), Err(Error::Conflict));
    drop(db);
    let reopened = Store::open(&path).unwrap();
    assert!(reopened.list().unwrap()[0].done);
    assert_eq!(reopened.audit().unwrap().iter().map(|e| e.operation.as_str()).collect::<Vec<_>>(), ["create","close"]);
}

#[tokio::test]
async fn step3_queue_backpressure_reports_full_and_closed() {
    let (sender, mut receiver) = mpsc::channel(1);
    assert_eq!(enqueue(&sender, 7), Ok(()));
    assert_eq!(enqueue(&sender, 8), Err(Error::Busy));
    assert_eq!(receiver.recv().await, Some(7));
    assert_eq!(enqueue(&sender, 9), Ok(()));
    assert_eq!(receiver.recv().await, Some(9));
    drop(receiver);
    assert_eq!(enqueue(&sender, 10), Err(Error::Closed));
}

#[test]
fn step4_json_events_are_bounded_and_have_no_raw_path_or_credentials() {
    assert!(AccessLog::new(0).is_err());
    assert!(AccessLog::new(257).is_err());
    let log = AccessLog::new(2).unwrap();
    let first = log.record(&Method::GET, "/health", StatusCode::OK, 2);
    let second = log.record(&Method::POST, "/api/tickets/7/close", StatusCode::FORBIDDEN, 3);
    let third = log.record(&Method::GET, "/private/SECRET-token-should-not-appear", StatusCode::NOT_FOUND, 9);
    assert_eq!((first, second, third), (1,2,3));
    let (total, dropped, events) = log.snapshot();
    assert_eq!((total, dropped, events.len()), (3,1,2));
    assert_eq!(events[0].route, "ticket_close");
    assert_eq!(events[1].route, "other");
    let text = log.json_lines();
    assert!(!text.contains("SECRET") && !text.contains("/private") && !text.contains("/api/tickets"));
    for line in text.lines() {
        let object = serde_json::from_str::<serde_json::Value>(line).unwrap();
        assert_eq!(object.as_object().unwrap().len(), 5);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn step5_https_sqlite_permissions_and_concurrent_writes_are_real() {
    let directory = Temp::new();
    let path = directory.0.join("tickets.db");
    Store::init(&path).unwrap();
    let cert = certificate();
    let reader = token(); let writer = token();
    let auth = Credentials::new(&reader, &writer).unwrap();
    let (backend, worker) = start_store(&path).unwrap();
    let log = AccessLog::new(64).unwrap();
    let tcp = TcpListener::bind(("127.0.0.1",0)).await.unwrap();
    let addr = tcp.local_addr().unwrap();
    let listener = TlsListener::new(tcp, server_config(&cert.der, &cert.key).unwrap()).unwrap();
    let (stop, signal) = oneshot::channel::<()>();
    let server = tokio::spawn(serve(listener, router(backend, auth, log.clone()), async { let _ = signal.await; }));
    let sensitive_title = "private title";
    let body = serde_json::json!({"title":sensitive_title}).to_string();
    assert!(request(addr, &certificate().der, "localhost", http("GET","/health",None,"","")).await.is_err());
    assert_eq!(request(addr, &cert.der, "localhost", http("POST","/api/tickets",None,&body,"")).await.unwrap().0, 401);
    assert_eq!(request(addr, &cert.der, "localhost", http("POST","/api/tickets",Some(&reader),&body,"")).await.unwrap().0, 403);
    assert!(Store::open(&path).unwrap().list().unwrap().is_empty());
    assert!(Store::open(&path).unwrap().audit().unwrap().is_empty());
    assert_eq!(request(addr, &cert.der, "localhost", http("POST","/api/tickets",Some(&writer),&body,"")).await.unwrap().0, 201);
    let mut tasks = JoinSet::new();
    for index in 0..12 {
        let trust = cert.der.clone(); let writer = writer.clone();
        tasks.spawn(async move {
            let body = serde_json::json!({"title":format!("concurrent-{index}")}).to_string();
            let status = request(addr,&trust,"localhost",http("POST","/api/tickets",Some(&writer),&body,"")).await.unwrap().0;
            assert!(status == 201 || status == 503, "unexpected status {status}");
            (index, status)
        });
    }
    let mut successful = Vec::new();
    while let Some(result) = tasks.join_next().await {
        let (index,status) = result.unwrap();
        if status == 201 { successful.push(format!("concurrent-{index}")); }
    }
    assert_eq!(request(addr,&cert.der,"localhost",http("POST","/api/tickets/1/close",Some(&writer),"","")).await.unwrap().0,200);
    let (status, response) = request(addr,&cert.der,"localhost",http("GET","/api/tickets",Some(&reader),"","")).await.unwrap();
    assert_eq!(status,200);
    assert!(response.contains(sensitive_title));
    assert!(response.to_ascii_lowercase().contains("x-e1-request-id:"));
    stop.send(()).unwrap();
    tokio::time::timeout(LIMIT,server).await.unwrap().unwrap().unwrap();
    tokio::time::timeout(LIMIT,worker).await.unwrap().unwrap();
    let db = Store::open(&path).unwrap();
    let rows = db.list().unwrap();
    assert_eq!(rows.len(), successful.len()+1);
    assert!(rows[0].done);
    assert_eq!(rows.iter().map(|t| t.id).collect::<Vec<_>>(), (1..=rows.len() as u64).collect::<Vec<_>>());
    for title in successful { assert!(rows.iter().any(|t| t.title == title)); }
    assert_eq!(db.audit().unwrap().len(),rows.len()+1);
    let (total,dropped,events)=log.snapshot();
    assert_eq!(total,17); // 2 denied, 1 create, 12 concurrent, close, list; no HTTP on bad TLS.
    assert_eq!(dropped,0);
    assert!(events.iter().any(|e| e.status==401));
    assert!(events.iter().any(|e| e.status==403));
    let logs=log.json_lines();
    assert!(!logs.contains(&reader) && !logs.contains(&writer) && !logs.contains(sensitive_title));
    assert!(!logs.contains("Authorization") && !logs.contains("concurrent-"));
}
