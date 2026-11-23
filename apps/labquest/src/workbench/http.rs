//! Bounded HTTP for the local learning workbench. No proxy or cross-origin mode.
use super::ApiError;
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

const MAX_HEADERS: usize = 16 * 1024;
const MAX_BODY: usize = 4 * 1024 * 1024;
pub(super) type Fields = BTreeMap<String, String>;

pub(super) struct Request {
    pub method: String,
    pub target: String,
    pub body: Vec<u8>,
}

pub(super) struct Response {
    pub status: u16,
    pub mime: &'static str,
    pub body: Vec<u8>,
    lesson: bool,
}

impl Response {
    pub fn new(status: u16, mime: &'static str, body: impl Into<Vec<u8>>) -> Self {
        Self {
            status,
            mime,
            body: body.into(),
            lesson: false,
        }
    }
    pub fn json(status: u16, body: String) -> Self {
        Self::new(status, "application/json; charset=utf-8", body.into_bytes())
    }
    pub fn lesson(body: String) -> Self {
        Self {
            lesson: true,
            ..Self::new(200, "text/html; charset=utf-8", body.into_bytes())
        }
    }
    pub fn write(&self, stream: &mut TcpStream) -> std::io::Result<()> {
        let reason = match self.status {
            200 => "OK",
            202 => "Accepted",
            400 => "Bad Request",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            408 => "Request Timeout",
            409 => "Conflict",
            413 => "Content Too Large",
            415 => "Unsupported Media Type",
            431 => "Request Header Fields Too Large",
            503 => "Service Unavailable",
            _ => "Internal Server Error",
        };
        let policy = if self.lesson {
            "default-src 'none'; style-src 'unsafe-inline'; img-src data:; base-uri 'none'; form-action 'none'; frame-ancestors 'self'"
        } else {
            "default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self'; frame-src 'self'; img-src 'self' data:; base-uri 'none'; form-action 'none'; frame-ancestors 'none'"
        };
        write!(stream, "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nReferrer-Policy: no-referrer\r\nContent-Security-Policy: {}\r\n\r\n", self.status, reason, self.mime, self.body.len(), policy)?;
        stream.write_all(&self.body)
    }
}

fn header(header: &str, port: u16) -> Result<(Request, usize), ApiError> {
    let mut lines = header.split("\r\n");
    let first: Vec<_> = lines.next().unwrap_or("").split(' ').collect();
    if first.len() != 3 || first.iter().any(|x| x.is_empty()) || first[2] != "HTTP/1.1" {
        return Err(ApiError::new(400, "需要完整的 HTTP/1.1 请求"));
    }
    if !matches!(first[0], "GET" | "POST") {
        return Err(ApiError::new(405, "只支持 GET 和 POST"));
    }
    if !first[1].starts_with('/')
        || first[1].starts_with("//")
        || first[1].contains('#')
        || first[1].bytes().any(|x| x.is_ascii_control())
    {
        return Err(ApiError::new(400, "请求地址格式不正确"));
    }
    let mut fields = BTreeMap::new();
    for line in lines.filter(|line| !line.is_empty()) {
        if line.starts_with([' ', '\t']) {
            return Err(ApiError::new(400, "不支持折行请求头"));
        }
        let (name, value) = line
            .split_once(':')
            .ok_or_else(|| ApiError::new(400, "请求头格式不正确"))?;
        if name.is_empty()
            || !name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&b))
        {
            return Err(ApiError::new(400, "请求头名称不正确"));
        }
        let value = value.trim();
        if value.bytes().any(|b| b.is_ascii_control() && b != b'\t')
            || fields
                .insert(name.to_ascii_lowercase(), value.to_string())
                .is_some()
        {
            return Err(ApiError::new(400, "请求头重复或包含控制字符"));
        }
    }
    let host = fields
        .get("host")
        .ok_or_else(|| ApiError::new(403, "缺少本机 Host"))?;
    if host != &format!("127.0.0.1:{port}")
        && !host.eq_ignore_ascii_case(&format!("localhost:{port}"))
    {
        return Err(ApiError::new(403, "工作台只接受当前本机地址"));
    }
    if fields.contains_key("transfer-encoding") {
        return Err(ApiError::new(400, "不支持分块请求体"));
    }
    let length = match fields.get("content-length") {
        Some(value) if !value.is_empty() && value.bytes().all(|x| x.is_ascii_digit()) => value
            .parse::<usize>()
            .map_err(|_| ApiError::new(413, "请求体过大"))?,
        Some(_) => return Err(ApiError::new(400, "Content-Length 不正确")),
        None if first[0] == "POST" => return Err(ApiError::new(400, "POST 需要 Content-Length")),
        None => 0,
    };
    if length > MAX_BODY {
        return Err(ApiError::new(413, "请求体超过 4 MiB"));
    }
    if first[0] == "GET" && length != 0 {
        return Err(ApiError::new(400, "GET 不接受请求体"));
    }
    if first[0] == "POST" {
        if fields.get("origin") != Some(&format!("http://{host}"))
            || fields.get("x-rustapplab").map(String::as_str) != Some("workbench")
        {
            return Err(ApiError::new(
                403,
                "请从当前工作台页面发起操作，刷新页面后重试",
            ));
        }
        let mime = fields
            .get("content-type")
            .map(|x| x.split(';').next().unwrap_or("").trim());
        if !mime.is_some_and(|x| x.eq_ignore_ascii_case("application/x-www-form-urlencoded")) {
            return Err(ApiError::new(415, "请求需要表单编码"));
        }
    }
    Ok((
        Request {
            method: first[0].into(),
            target: first[1].into(),
            body: Vec::new(),
        },
        length,
    ))
}

pub(super) fn read(stream: &mut TcpStream, port: u16) -> Result<Request, ApiError> {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| ApiError::new(500, e.to_string()))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| ApiError::new(500, e.to_string()))?;
    let mut bytes = Vec::new();
    let (end, mut request, length) = loop {
        let mut chunk = [0u8; 4096];
        let count = stream.read(&mut chunk).map_err(read_error)?;
        if count == 0 {
            return Err(ApiError::new(400, "请求尚未接收完整"));
        }
        bytes.extend_from_slice(&chunk[..count]);
        if let Some(pos) = bytes.windows(4).position(|x| x == b"\r\n\r\n") {
            let end = pos + 4;
            if end > MAX_HEADERS {
                return Err(ApiError::new(431, "请求头超过 16 KiB"));
            }
            let text = std::str::from_utf8(&bytes[..end])
                .map_err(|_| ApiError::new(400, "请求头编码不正确"))?;
            let (request, length) = header(text, port)?;
            break (end, request, length);
        }
        if bytes.len() > MAX_HEADERS {
            return Err(ApiError::new(431, "请求头超过 16 KiB"));
        }
    };
    if bytes.len() - end > length {
        return Err(ApiError::new(400, "请求体长度与声明不符"));
    }
    request.body.extend_from_slice(&bytes[end..]);
    while request.body.len() < length {
        let mut chunk = [0u8; 8192];
        let remaining = (length - request.body.len()).min(chunk.len());
        let count = stream.read(&mut chunk[..remaining]).map_err(read_error)?;
        if count == 0 {
            return Err(ApiError::new(400, "请求体提前结束"));
        }
        request.body.extend_from_slice(&chunk[..count]);
    }
    Ok(request)
}

fn read_error(error: std::io::Error) -> ApiError {
    if matches!(
        error.kind(),
        std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
    ) {
        ApiError::new(408, "读取请求超时，请重试")
    } else {
        ApiError::new(400, format!("读取请求失败：{error}"))
    }
}

fn decode(value: &str) -> Result<String, ApiError> {
    fn hex(b: u8) -> Option<u8> {
        match b {
            b'0'..=b'9' => Some(b - b'0'),
            b'a'..=b'f' => Some(b - b'a' + 10),
            b'A'..=b'F' => Some(b - b'A' + 10),
            _ => None,
        }
    }
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => decoded.push(b' '),
            b'%' => {
                let high = bytes.get(i + 1).and_then(|&b| hex(b));
                let low = bytes.get(i + 2).and_then(|&b| hex(b));
                match (high, low) {
                    (Some(high), Some(low)) => {
                        decoded.push(high * 16 + low);
                        i += 2;
                    }
                    _ => return Err(ApiError::new(400, "表单包含不完整的百分号编码")),
                }
            }
            byte => decoded.push(byte),
        }
        i += 1;
    }
    if decoded.contains(&0) {
        return Err(ApiError::new(400, "表单不能包含空字符"));
    }
    String::from_utf8(decoded).map_err(|_| ApiError::new(400, "表单需要有效的 UTF-8 文本"))
}

pub(super) fn fields(body: &[u8]) -> Result<Fields, ApiError> {
    let text =
        std::str::from_utf8(body).map_err(|_| ApiError::new(400, "表单需要有效的 UTF-8 文本"))?;
    let mut fields = Fields::new();
    if text.is_empty() {
        return Ok(fields);
    }
    for pair in text.split('&') {
        let (key, value) = pair
            .split_once('=')
            .ok_or_else(|| ApiError::new(400, "表单字段缺少等号"))?;
        let key = decode(key)?;
        let value = decode(value)?;
        if key.is_empty() || key.len() > 64 || value.len() > 1024 * 1024 || fields.len() >= 16 {
            return Err(ApiError::new(413, "表单字段超过允许范围"));
        }
        if fields.insert(key, value).is_some() {
            return Err(ApiError::new(400, "表单字段不能重复"));
        }
    }
    Ok(fields)
}

pub(super) fn required<'a>(fields: &'a Fields, key: &str) -> Result<&'a str, ApiError> {
    fields
        .get(key)
        .map(String::as_str)
        .ok_or_else(|| ApiError::new(400, format!("缺少字段：{key}")))
}
pub(super) fn only(fields: &Fields, keys: &[&str]) -> Result<(), ApiError> {
    if let Some(key) = fields.keys().find(|key| !keys.contains(&key.as_str())) {
        return Err(ApiError::new(400, format!("不支持字段：{key}")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn post(extra: &str) -> String {
        format!("POST /api/save HTTP/1.1\r\nHost: 127.0.0.1:9123\r\nOrigin: http://127.0.0.1:9123\r\nX-RustAppLab: workbench\r\nContent-Type: application/x-www-form-urlencoded;charset=UTF-8\r\nContent-Length: 0\r\n{extra}\r\n")
    }
    #[test]
    fn matching_local_form_is_accepted() {
        assert!(header(&post(""), 9123).is_ok());
    }
    #[test]
    fn cross_origin_and_missing_intent_cannot_mutate() {
        for bad in [
            post("").replace(
                "Origin: http://127.0.0.1:9123",
                "Origin: https://example.com",
            ),
            post("").replace("X-RustAppLab: workbench\r\n", ""),
        ] {
            assert_eq!(header(&bad, 9123).err().unwrap().status, 403);
        }
    }
    #[test]
    fn rebinding_host_is_rejected() {
        let bad = post("").replace("Host: 127.0.0.1:9123", "Host: example.com:9123");
        assert_eq!(header(&bad, 9123).err().unwrap().status, 403);
    }
    #[test]
    fn ambiguous_framing_is_rejected() {
        for extra in [
            "Content-Length: 0\r\n",
            "Transfer-Encoding: chunked\r\n",
            "Host: localhost:9123\r\n",
        ] {
            assert_eq!(header(&post(extra), 9123).err().unwrap().status, 400);
        }
    }
    #[test]
    fn large_body_is_rejected_before_reading() {
        let bad = post("").replace("Content-Length: 0", "Content-Length: 4194305");
        assert_eq!(header(&bad, 9123).err().unwrap().status, 413);
    }
    #[test]
    fn form_decoding_preserves_source_unicode_and_literal_symbols() {
        let parsed =
            fields(b"path=src%2Flib.rs&content=%E4%B8%AD%E6%96%87+%2B%26%3D%25%0A").unwrap();
        assert_eq!(parsed["path"], "src/lib.rs");
        assert_eq!(parsed["content"], "中文 +&=%\n");
    }
    #[test]
    fn invalid_or_duplicate_form_data_is_rejected() {
        for text in [
            "id=x&id=y",
            "content=%FF",
            "content=%0",
            "content=%GG",
            "path=%00",
            "id",
        ] {
            assert!(fields(text.as_bytes()).is_err(), "{text}");
        }
    }
    #[test]
    fn query_and_body_keys_are_strict() {
        let values = fields(b"id=unitlab&unexpected=x").unwrap();
        assert!(only(&values, &["id"]).is_err());
        assert!(required(&values, "step").is_err());
    }
}
