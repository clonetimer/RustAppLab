use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, header};
use e1_secure_lab::*;
use std::time::Duration;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream}, sync::oneshot};
use tokio_rustls::TlsAcceptor;
mod support;
use support::*;

fn bearer(value: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(header::AUTHORIZATION, HeaderValue::from_str(value).unwrap());
    headers
}

#[test]
fn step1_authentication_rejects_bad_and_duplicate_credentials() {
    let r = token(); let w = token(); let invalid = token();
    let credentials = Credentials::new(&r, &w).unwrap();
    assert!(Credentials::new(&r, &r).is_err());
    assert!(Credentials::new("short", &w).is_err());
    assert!(Credentials::new(&"z".repeat(64), &w).is_err());
    assert_eq!(authenticate(&credentials, &bearer(&format!("Bearer {r}"))), Some(Role::Reader));
    assert_eq!(authenticate(&credentials, &bearer(&format!("bEaReR {w}"))), Some(Role::Writer));
    assert_eq!(authenticate(&credentials, &HeaderMap::new()), None);
    for value in [format!("Bearer {invalid}"), format!("Basic {r}"), format!("Bearer  {r}"), format!("Bearer {r} "), "Bearer short".into()] {
        assert_eq!(authenticate(&credentials, &bearer(&value)), None);
    }
    let mut duplicate = bearer(&format!("Bearer {r}"));
    duplicate.append(header::AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {w}")).unwrap());
    assert_eq!(authenticate(&credentials, &duplicate), None);
}

#[test]
fn step2_policy_fails_closed_and_distinguishes_401_from_403() {
    let get = Method::GET; let post = Method::POST;
    assert_eq!(decide(&get, "/health", None, false), Ok(()));
    assert_eq!(decide(&get, "/api/tickets", None, false), Err(StatusCode::UNAUTHORIZED));
    assert_eq!(decide(&get, "/api/tickets", Some(Role::Reader), false), Ok(()));
    assert_eq!(decide(&get, "/api/tickets", Some(Role::Writer), false), Ok(()));
    for path in ["/api/tickets", "/api/tickets/1/close"] {
        assert_eq!(decide(&post, path, None, false), Err(StatusCode::UNAUTHORIZED));
        assert_eq!(decide(&post, path, Some(Role::Reader), false), Err(StatusCode::FORBIDDEN));
        assert_eq!(decide(&post, path, Some(Role::Writer), false), Ok(()));
        assert_eq!(decide(&post, path, Some(Role::Writer), true), Err(StatusCode::FORBIDDEN));
    }
    for path in ["/", "/assets/client.js", "/api/tickets/1/close/extra", "/api/tickets/not-id/close", "/api/tickets/18446744073709551616/close"] {
        assert_eq!(decide(&post, path, Some(Role::Writer), false), Err(StatusCode::NOT_FOUND));
    }
    assert_eq!(decide(&Method::PUT, "/api/tickets", Some(Role::Writer), false), Err(StatusCode::NOT_FOUND));
}

#[test]
fn step3_tls_configuration_rejects_invalid_der_and_mismatched_key() {
    let good = certificate(); let other = certificate();
    assert!(server_config(&good.der, &good.key).is_ok());
    assert!(server_config(b"not a cert", &good.key).is_err());
    assert!(server_config(&good.der, b"not a key").is_err());
    assert!(server_config(&good.der, &other.key).is_err());
}

#[tokio::test]
async fn step4_handshake_negotiates_tls_and_times_out_without_plaintext() {
    // Construct the fixed fixture independently so this tests the negotiation TODO alone.
    let cert = certificate();
    let config = rustls::ServerConfig::builder_with_provider(std::sync::Arc::new(rustls::crypto::ring::default_provider()))
        .with_protocol_versions(&[&rustls::version::TLS13]).unwrap().with_no_client_auth()
        .with_single_cert(vec![rustls::pki_types::CertificateDer::from(cert.der.clone())],
            rustls::pki_types::PrivateKeyDer::Pkcs8(rustls::pki_types::PrivatePkcs8KeyDer::from(cert.key))).unwrap();
    let acceptor = TlsAcceptor::from(std::sync::Arc::new(config));
    let tcp = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let addr = tcp.local_addr().unwrap();
    let accept = acceptor.clone();
    let server = tokio::spawn(async move {
        let (stream, _) = tcp.accept().await.unwrap();
        let tls = negotiate(accept, stream, LIMIT).await.unwrap();
        assert_eq!(tls.get_ref().1.protocol_version(), Some(rustls::ProtocolVersion::TLSv1_3));
        let (stream, _) = tcp.accept().await.unwrap();
        let result = negotiate(acceptor, stream, Duration::from_millis(100)).await;
        assert_eq!(result.err().unwrap().kind(), std::io::ErrorKind::TimedOut);
    });
    let socket = TcpStream::connect(addr).await.unwrap();
    let name = rustls::pki_types::ServerName::try_from("localhost").unwrap();
    let stream = tokio::time::timeout(LIMIT, connector(&cert.der).connect(name, socket)).await.unwrap().unwrap();
    assert_eq!(stream.get_ref().1.protocol_version(), Some(rustls::ProtocolVersion::TLSv1_3));
    drop(stream);
    let idle = TcpStream::connect(addr).await.unwrap();
    tokio::time::timeout(LIMIT, server).await.unwrap().unwrap();
    drop(idle);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn step5_real_https_c2_api_permissions_and_persistence() {
    let directory = Temp::new();
    let journal = directory.0.join("tickets.db");
    let cert = certificate(); let wrong = certificate();
    let reader = token(); let writer = token(); let invalid = token();
    let config = server_config(&cert.der, &cert.key).unwrap();
    let auth = Credentials::new(&reader, &writer).unwrap();
    // Exercise the real C2 lifecycle: startup must not silently create a store.
    assert!(matches!(servicedesk_c2::start_store(&journal), Err(servicedesk_c2::ApiError::Storage)));
    servicedesk_c2::init_store(&journal).unwrap();
    let state = servicedesk_c2::start_store(&journal).unwrap();
    let before = std::fs::read(&journal).unwrap();
    let tcp = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let addr = tcp.local_addr().unwrap();
    let listener = TlsListener::new(tcp, config).unwrap();
    let (stop, signal) = oneshot::channel::<()>();
    let server = tokio::spawn(serve(listener, protected_router(state, auth), async { let _ = signal.await; }));
    let health = || http("GET", "/health", None, "", "");
    assert!(request(addr, &wrong.der, "localhost", health()).await.is_err());
    assert!(request(addr, &cert.der, "wrong.invalid", health()).await.is_err());

    let mut plain = TcpStream::connect(addr).await.unwrap();
    plain.write_all(health().as_bytes()).await.unwrap();
    let mut data = [0u8; 128];
    let read = tokio::time::timeout(LIMIT, plain.read(&mut data)).await.unwrap();
    if let Ok(n) = read { assert!(!data[..n].starts_with(b"HTTP/")); }
    drop(plain);
    let idle = TcpStream::connect(addr).await.unwrap();
    assert_eq!(request(addr, &cert.der, "localhost", health()).await.unwrap().0, 200);
    drop(idle);

    let call = |method: &str, path: &str, secret: Option<&str>, body: &str, extra: &str| {
        request(addr, &cert.der, "localhost", http(method, path, secret, body, extra))
    };
    let (status, response) = call("GET", "/api/tickets", None, "", "").await.unwrap();
    assert_eq!(status, 401);
    assert!(response.to_ascii_lowercase().contains("www-authenticate: bearer"));
    assert!(response.to_ascii_lowercase().contains("cache-control: no-store"));
    let title = r#"{"title":"E1 local test"}"#;
    assert_eq!(call("POST", "/api/tickets", Some(&invalid), title, "").await.unwrap().0, 401);
    assert_eq!(call("POST", "/api/tickets", Some(&reader), title, "").await.unwrap().0, 403);
    assert_eq!(call("POST", "/api/tickets", Some(&writer), title, "Origin: https://localhost\r\n").await.unwrap().0, 403);
    let duplicate = format!("Authorization: Bearer {writer}\r\n");
    assert_eq!(call("POST", "/api/tickets", Some(&reader), title, &duplicate).await.unwrap().0, 401);
    assert_eq!(std::fs::read(&journal).unwrap(), before, "denied requests must not modify persistent state");
    assert_eq!(call("POST", "/api/tickets", Some(&writer), title, "").await.unwrap().0, 201);
    let (status, response) = call("GET", "/api/tickets", Some(&reader), "", "").await.unwrap();
    assert_eq!(status, 200); assert!(response.contains("E1 local test"));
    assert!(!response.contains(&reader) && !response.contains(&writer));
    assert_eq!(call("POST", "/api/tickets/1/close", Some(&writer), "", "").await.unwrap().0, 200);
    assert_eq!(call("GET", "/", Some(&writer), "", "").await.unwrap().0, 404);

    stop.send(()).unwrap();
    tokio::time::timeout(LIMIT, server).await.unwrap().unwrap().unwrap();
    let reopened = servicedesk_c2::start_store(&journal).unwrap().list().await.unwrap();
    assert_eq!(reopened.len(), 1);
    assert_eq!(reopened[0].title, "E1 local test");
    assert!(reopened[0].done);
}
