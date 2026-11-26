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

/// TODO target 1: authenticate exactly one well-formed bearer header.
/// Constant-time comparison is used on both equal-length configured values.
pub fn authenticate(credentials: &Credentials, headers: &HeaderMap) -> Option<Role> {
    use subtle::ConstantTimeEq;
    let mut values = headers.get_all(header::AUTHORIZATION).iter();
    let value = values.next()?.to_str().ok()?;
    if values.next().is_some() {
        return None;
    }
    let (scheme, token) = value.split_once(' ')?;
    if !scheme.eq_ignore_ascii_case("Bearer") || !valid_token(token) {
        return None;
    }
    let reader = bool::from(credentials.0.reader.as_slice().ct_eq(token.as_bytes()));
    let writer = bool::from(credentials.0.writer.as_slice().ct_eq(token.as_bytes()));
    if writer {
        Some(Role::Writer)
    } else if reader {
        Some(Role::Reader)
    } else {
        None
    }
}

/// TODO target 2: permit only the explicitly listed C2 API routes.
/// The legacy C2 browser write-origin policy stays unchanged: E1-A is API-only.
pub fn decide(method: &Method, path: &str, role: Option<Role>, has_origin: bool)
    -> Result<(), StatusCode>
{
    if method == Method::GET && path == "/health" {
        return Ok(());
    }
    let list = method == Method::GET && path == "/api/tickets";
    let close = path.strip_prefix("/api/tickets/")
        .and_then(|suffix| suffix.strip_suffix("/close"))
        .is_some_and(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()) && id.parse::<u64>().is_ok());
    let write = method == Method::POST && (path == "/api/tickets" || close);
    if !list && !write {
        return Err(StatusCode::NOT_FOUND);
    }
    let role = role.ok_or(StatusCode::UNAUTHORIZED)?;
    if write && (role != Role::Writer || has_origin) {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(())
}

/// TODO target 3: construct a TLS 1.3 configuration using the actual library.
pub fn server_config(cert_der: &[u8], key_pkcs8_der: &[u8]) -> Result<Arc<ServerConfig>, &'static str> {
    use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let mut config = ServerConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13])
        .map_err(|_| "TLS configuration unavailable")?
        .with_no_client_auth()
        .with_single_cert(
            vec![CertificateDer::from(cert_der.to_vec())],
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key_pkcs8_der.to_vec())),
        )
        .map_err(|_| "invalid certificate or private key")?;
    config.alpn_protocols = vec![b"http/1.1".to_vec()];
    Ok(Arc::new(config))
}

/// TODO target 4: bound each TLS negotiation; never fall back to plaintext.
pub async fn negotiate(acceptor: TlsAcceptor, stream: TcpStream, timeout: Duration)
    -> io::Result<TlsStream<TcpStream>>
{
    tokio::time::timeout(timeout, acceptor.accept(stream)).await
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "TLS handshake timed out"))?
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

pub fn protected_router(state: servicedesk_c2::AppState, credentials: Credentials) -> Router {
    servicedesk_c2::router(state).layer(middleware::from_fn_with_state(credentials, guard))
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
