//! E1-A: local HTTPS and explicit authorization around the existing C2 router.
//! Teaching scope: two configured bearer credentials, not a production identity service.
use axum::{
    Router,
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, Method, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    serve::Listener,
};
use rustls::ServerConfig;
use std::{future::Future, io, net::SocketAddr, sync::Arc, time::Duration};
use tokio::{net::{TcpListener, TcpStream}, task::JoinSet};
use tokio_rustls::{TlsAcceptor, server::TlsStream};

const TOKEN_LEN: usize = 64;
const HANDSHAKE_LIMIT: usize = 16;
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Reader,
    Writer,
}

// Deliberately no Debug implementation: credentials must never be printed.
#[derive(Clone)]
pub struct Credentials(Arc<Secrets>);
struct Secrets {
    reader: [u8; TOKEN_LEN],
    writer: [u8; TOKEN_LEN],
}

fn valid_token(value: &str) -> bool {
    value.len() == TOKEN_LEN && value.bytes().all(|b| b.is_ascii_hexdigit())
}

impl Credentials {
    pub fn new(reader: &str, writer: &str) -> Result<Self, &'static str> {
        if !valid_token(reader) || !valid_token(writer) || reader == writer {
            return Err("configure two distinct 64-character hex credentials");
        }
        let reader = reader.as_bytes().try_into().map_err(|_| "invalid credential")?;
        let writer = writer.as_bytes().try_into().map_err(|_| "invalid credential")?;
        Ok(Self(Arc::new(Secrets { reader, writer })))
    }
}

pub fn authenticate(credentials: &Credentials, headers: &HeaderMap) -> Option<Role> {
    // TODO(1): exactly one bearer credential, validate length/format, compare both configured values.
    let _ = (&credentials.0.reader, &credentials.0.writer, headers);
    None
}

pub fn decide(method: &Method, path: &str, role: Option<Role>, has_origin: bool)
    -> Result<(), StatusCode>
{
    // TODO(2): allow public GET /health, authenticate known routes, enforce write role and no Origin.
    let _ = (method, path, role, has_origin);
    Err(StatusCode::UNAUTHORIZED)
}

pub fn server_config(cert_der: &[u8], key_pkcs8_der: &[u8]) -> Result<Arc<ServerConfig>, &'static str> {
    // TODO(3): rustls ring provider, TLS13, DER certificate/PKCS8 key, HTTP/1.1 ALPN.
    let _ = (cert_der, key_pkcs8_der);
    Err("TLS configuration TODO")
}

pub async fn negotiate(acceptor: TlsAcceptor, stream: TcpStream, timeout: Duration)
    -> io::Result<TlsStream<TcpStream>>
{
    // TODO(4): use timeout around actual TlsAcceptor; no plaintext fallback.
    let _ = (acceptor, stream, timeout);
    Err(io::Error::other("TLS negotiation TODO"))
}

// Supplied integration. This wraps the real, unchanged C2 HTTP router and actor.
async fn guard(State(credentials): State<Credentials>, request: Request, next: Next) -> Response {
    let role = authenticate(&credentials, request.headers());
    let status = decide(request.method(), request.uri().path(), role,
                        request.headers().contains_key(header::ORIGIN));
    let mut response = match status {
        Ok(()) => next.run(request).await,
        Err(code) => code.into_response(),
    };
    response.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    if response.status() == StatusCode::UNAUTHORIZED {
        response.headers_mut().insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
    }
    response
}

/// Apply exactly the E1-A route/role policy to a supplied router.
/// E1-B changes the storage backend, not the authorization or TLS contract.
pub fn protect_router(app: Router, credentials: Credentials) -> Router {
    app.layer(middleware::from_fn_with_state(credentials, guard))
}

pub fn protected_router(state: servicedesk_c2::AppState, credentials: Credentials) -> Router {
    protect_router(servicedesk_c2::router(state), credentials)
}

/// Loopback-only, bounded pending TLS handshakes. This is not a global HTTP
/// connection/rate limit; E1-B will address additional resource limits.
pub struct TlsListener {
    tcp: TcpListener,
    acceptor: TlsAcceptor,
    pending: JoinSet<io::Result<(TlsStream<TcpStream>, SocketAddr)>>,
}

impl TlsListener {
    pub fn new(tcp: TcpListener, config: Arc<ServerConfig>) -> io::Result<Self> {
        if !tcp.local_addr()?.ip().is_loopback() {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, "loopback required"));
        }
        Ok(Self { tcp, acceptor: TlsAcceptor::from(config), pending: JoinSet::new() })
    }
}

impl Listener for TlsListener {
    type Io = TlsStream<TcpStream>;
    type Addr = SocketAddr;

    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        loop {
            tokio::select! {
                result = self.pending.join_next(), if !self.pending.is_empty() => {
                    if let Some(Ok(Ok(connection))) = result {
                        return connection;
                    }
                    // Do not log credential material or unauthenticated peer data.
                }
                incoming = self.tcp.accept(), if self.pending.len() < HANDSHAKE_LIMIT => {
                    match incoming {
                        Ok((socket, peer)) => {
                            let acceptor = self.acceptor.clone();
                            self.pending.spawn(async move {
                                let stream = negotiate(acceptor, socket, HANDSHAKE_TIMEOUT).await?;
                                Ok((stream, peer))
                            });
                        }
                        Err(_) => tokio::time::sleep(Duration::from_millis(50)).await,
                    }
                }
            }
        }
    }

    fn local_addr(&self) -> io::Result<Self::Addr> {
        self.tcp.local_addr()
    }
}

pub async fn serve(listener: TlsListener, app: Router, shutdown: impl Future<Output = ()> + Send + 'static)
    -> io::Result<()>
{
    axum::serve(listener, app).with_graceful_shutdown(shutdown).await
}
