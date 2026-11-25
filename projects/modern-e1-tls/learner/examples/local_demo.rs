//! A real in-process loopback HTTPS demo; generated keys/tokens are never displayed.
#[path = "../tests/support/mod.rs"]
mod support;
use e1_secure_lab::*;
use support::*;
use tokio::{net::TcpListener, sync::oneshot};

#[tokio::main]
async fn main() {
    let directory = Temp::new();
    let cert = certificate();
    let reader = token(); let writer = token();
    let auth = Credentials::new(&reader, &writer).unwrap();
    let tcp = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let addr = tcp.local_addr().unwrap();
    let tls = TlsListener::new(tcp, server_config(&cert.der, &cert.key).unwrap()).unwrap();
    let state = servicedesk_c2::start_store(&directory.0.join("tickets.db")).unwrap();
    let (stop, signal) = oneshot::channel::<()>();
    let server = tokio::spawn(serve(tls, protected_router(state, auth), async { let _ = signal.await; }));
    let missing = request(addr, &cert.der, "localhost", http("GET", "/api/tickets", None, "", "")).await.unwrap().0;
    let denied = request(addr, &cert.der, "localhost", http("POST", "/api/tickets", Some(&reader), r#"{"title":"demo"}"#, "")).await.unwrap().0;
    let created = request(addr, &cert.der, "localhost", http("POST", "/api/tickets", Some(&writer), r#"{"title":"demo"}"#, "")).await.unwrap().0;
    let read = request(addr, &cert.der, "localhost", http("GET", "/api/tickets", Some(&reader), "", "")).await.unwrap().0;
    assert_eq!((missing, denied, created, read), (401, 403, 201, 200));
    println!("HTTPS verified; missing={missing}; reader_write={denied}; writer_create={created}; reader_list={read}");
    stop.send(()).unwrap();
    tokio::time::timeout(LIMIT, server).await.unwrap().unwrap().unwrap();
}
