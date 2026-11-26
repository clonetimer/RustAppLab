//! Fixed local TLS test client and generated certificate fixture.
//! This raw HTTP/1.1 helper is for deterministic teaching tests, not a production HTTP client.
#![allow(dead_code)]
use rustls::{ClientConfig, RootCertStore, pki_types::{CertificateDer, ServerName}};
use std::{io, net::SocketAddr, path::PathBuf, sync::Arc, time::Duration};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};
use tokio_rustls::TlsConnector;

pub const LIMIT: Duration = Duration::from_secs(8);

pub struct Cert {
    pub der: Vec<u8>,
    pub key: Vec<u8>,
}
pub fn certificate() -> Cert {
    let pair = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    Cert { der: pair.cert.der().as_ref().to_vec(), key: pair.signing_key.serialize_der() }
}

pub fn token() -> String {
    let mut bytes = [0u8; 32];
    rustls::crypto::ring::default_provider().secure_random.fill(&mut bytes).unwrap();
    bytes.iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>().concat()
}

pub fn connector(trust: &[u8]) -> TlsConnector {
    let mut roots = RootCertStore::empty();
    roots.add(CertificateDer::from(trust.to_vec())).unwrap();
    let mut config = ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
        .with_protocol_versions(&[&rustls::version::TLS13]).unwrap()
        .with_root_certificates(roots).with_no_client_auth();
    config.alpn_protocols = vec![b"http/1.1".to_vec()];
    TlsConnector::from(Arc::new(config))
}

pub async fn request(addr: SocketAddr, trust: &[u8], hostname: &str, request: String) -> io::Result<(u16, String)> {
    tokio::time::timeout(LIMIT, async {
        let socket = TcpStream::connect(addr).await?;
        let name = ServerName::try_from(hostname.to_owned())
            .map_err(|_| io::Error::other("invalid server name"))?;
        let mut tls = connector(trust).connect(name, socket).await?;
        tls.write_all(request.as_bytes()).await?;
        let mut raw = Vec::new();
        // A complete HTTP response with Connection: close is required by the fixture.
        tls.read_to_end(&mut raw).await?;
        let response = String::from_utf8(raw).map_err(|_| io::Error::other("invalid response"))?;
        let status = response.lines().next().and_then(|l| l.split_whitespace().nth(1))
            .and_then(|v| v.parse::<u16>().ok()).ok_or_else(|| io::Error::other("no HTTP status"))?;
        Ok((status, response))
    }).await.map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "local request timed out"))?
}

pub fn http(method: &str, path: &str, token: Option<&str>, body: &str, extra: &str) -> String {
    let auth = token.map(|t| format!("Authorization: Bearer {t}\r\n")).unwrap_or_default();
    format!("{method} {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n{auth}X-RustAppLab-Client: 1\r\n{extra}Content-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}", body.len())
}

pub struct Temp(pub PathBuf);
impl Temp {
    pub fn new() -> Self {
        let path = std::env::temp_dir().join(format!("rustapplab-e1-{}", token()));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); }
}
