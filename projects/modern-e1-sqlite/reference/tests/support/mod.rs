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

// Use one absolute deadline for every phase. Never include request text or credentials in errors.
async fn phase<T>(deadline: tokio::time::Instant, label: &'static str,
                  future: impl std::future::Future<Output = io::Result<T>>) -> io::Result<T> {
    match tokio::time::timeout_at(deadline, future).await {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(error)) => Err(io::Error::new(error.kind(), format!("{label}: {error}"))),
        Err(_) => Err(io::Error::new(io::ErrorKind::TimedOut, format!("{label}: deadline exceeded"))),
    }
}

const MAX_RESPONSE: usize = 1024 * 1024;

/// Recognize only the fixture's HTTP/1.1 Content-Length or connection-close
/// framing. Reject ambiguous lengths and unsupported transfer encodings.
fn response_extent(bytes: &[u8]) -> io::Result<Option<(usize, Option<usize>)>> {
    let Some(header_end) = bytes.windows(4).position(|part| part == b"\r\n\r\n")
        .map(|index| index + 4) else { return Ok(None); };
    let header = std::str::from_utf8(&bytes[..header_end])
        .map_err(|_| io::Error::other("invalid response header"))?;
    let mut length = None;
    for line in header.split("\r\n").skip(1).filter(|s| !s.is_empty()) {
        let (name, value) = line.split_once(':').ok_or_else(|| io::Error::other("malformed response header"))?;
        if name.eq_ignore_ascii_case("transfer-encoding") {
            return Err(io::Error::other("unsupported response transfer encoding in fixture"));
        }
        if name.eq_ignore_ascii_case("content-length") {
            if length.is_some() { return Err(io::Error::other("duplicate content-length")); }
            let size = value.trim().parse::<usize>().map_err(|_| io::Error::other("invalid content-length"))?;
            let end = header_end.checked_add(size).filter(|end| *end <= MAX_RESPONSE)
                .ok_or_else(|| io::Error::other("response too large"))?;
            length = Some(end);
        }
    }
    Ok(Some((header_end, length)))
}

pub async fn request(addr: SocketAddr, trust: &[u8], hostname: &str, request: String) -> io::Result<(u16, String)> {
    let deadline = tokio::time::Instant::now() + LIMIT;
    let socket = phase(deadline, "TCP connect", TcpStream::connect(addr)).await?;
    let name = ServerName::try_from(hostname.to_owned())
        .map_err(|_| io::Error::other("invalid server name"))?;
    let mut tls = phase(deadline, "TLS handshake", connector(trust).connect(name, socket)).await?;
    phase(deadline, "HTTP send", tls.write_all(request.as_bytes())).await?;
    phase(deadline, "HTTP flush", tls.flush()).await?;
    let mut raw = Vec::new();
    let mut buffer = [0u8; 4096];
    loop {
        let frame = response_extent(&raw)?;
        if let Some((_, Some(end))) = frame {
            if raw.len() == end { break; }
            if raw.len() > end { return Err(io::Error::other("unexpected bytes after response body")); }
        }
        let label = if frame.is_some() { "HTTP response body" } else { "HTTP response headers" };
        let n = phase(deadline, label, tls.read(&mut buffer)).await?;
        if n == 0 {
            match frame {
                Some((_, None)) => break, // a close-delimited fixture response
                _ => return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "incomplete HTTP response")),
            }
        }
        if raw.len() + n > MAX_RESPONSE { return Err(io::Error::other("response too large")); }
        raw.extend_from_slice(&buffer[..n]);
    }
    let response = String::from_utf8(raw).map_err(|_| io::Error::other("invalid response"))?;
    let status = response.lines().next().and_then(|line| {
        let mut words = line.split_whitespace();
        if words.next()? != "HTTP/1.1" { return None; }
        words.next()?.parse::<u16>().ok().filter(|code| (100..=599).contains(code))
    }).ok_or_else(|| io::Error::other("no HTTP status"))?;
    Ok((status, response))
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
