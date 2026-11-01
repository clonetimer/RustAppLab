//! Synchronous educational HTTP/1.1 server: bounded input, same-origin mutation header.
use servicedesk::{self, Action};
use std::{
    io::{Read, Write},
    net::TcpStream,
    path::Path,
    time::Duration,
};
const PAGE: &str = include_str!("../samples/index.html");
const HEAD_LIMIT: usize = 16 * 1024;
const BODY_LIMIT: usize = 2048;
struct Request {
    method: String,
    path: String,
    body: Vec<u8>,
    host: String,
    origin: Option<String>,
    client_header: bool,
    content_type: Option<String>,
}
fn parse_request(stream: &mut TcpStream) -> Result<Request, String> {
    let mut raw = Vec::new();
    let mut buf = [0u8; 1024];
    let boundary;
    loop {
        let n = stream
            .read(&mut buf)
            .map_err(|e| format!("读取 HTTP 请求失败：{e}"))?;
        if n == 0 {
            return Err("HTTP 请求头不完整".into());
        }
        raw.extend_from_slice(&buf[..n]);
        if raw.len() > HEAD_LIMIT + BODY_LIMIT + 4 {
            return Err("请求超过限制".into());
        }
        if let Some(i) = raw.windows(4).position(|w| w == b"\r\n\r\n") {
            boundary = i;
            break;
        }
        if raw.len() > HEAD_LIMIT {
            return Err("HTTP 请求头太大".into());
        }
    }
    if boundary > HEAD_LIMIT {
        return Err("HTTP 请求头太大".into());
    }
    let head = std::str::from_utf8(&raw[..boundary]).map_err(|_| "HTTP 请求头必须是 UTF-8")?;
    let mut lines = head.split("\r\n");
    let request_line = lines.next().ok_or("缺少 HTTP 请求行")?;
    let parts = request_line.split(' ').collect::<Vec<_>>();
    if parts.len() != 3
        || parts[2] != "HTTP/1.1"
        || !matches!(parts[0], "GET" | "POST")
        || !parts[1].starts_with('/')
        || parts[1].contains('?')
    {
        return Err("仅支持 GET/POST 与 HTTP/1.1 明确路径".into());
    }
    let (mut host, mut origin, mut client_header, mut content_type, mut length) =
        (None, None, false, None, None);
    for line in lines {
        let (k, v) = line.split_once(':').ok_or("HTTP 请求头语法错误")?;
        if k.is_empty() || k.bytes().any(|b| !b.is_ascii_alphanumeric() && b != b'-') {
            return Err("HTTP 头字段名非法".into());
        }
        let v = v.trim();
        if k.eq_ignore_ascii_case("host") && host.replace(v.to_owned()).is_some() {
            return Err("重复 Host".into());
        }
        if k.eq_ignore_ascii_case("origin") && origin.replace(v.to_owned()).is_some() {
            return Err("重复 Origin".into());
        }
        if k.eq_ignore_ascii_case("x-rustapplab-client") {
            client_header = v == "1";
        }
        if k.eq_ignore_ascii_case("content-type") && content_type.replace(v.to_owned()).is_some() {
            return Err("重复 Content-Type".into());
        }
        if k.eq_ignore_ascii_case("transfer-encoding") {
            return Err("不支持 Transfer-Encoding".into());
        }
        if k.eq_ignore_ascii_case("content-length") {
            if length.is_some() || v.is_empty() || !v.bytes().all(|b| b.is_ascii_digit()) {
                return Err("非法或重复 Content-Length".into());
            }
            length = Some(v.parse::<usize>().map_err(|_| "Content-Length 太大")?);
        }
    }
    let n = length.unwrap_or(0);
    if n > BODY_LIMIT {
        return Err("请求正文超过 2 KiB".into());
    }
    let mut body = raw[boundary + 4..].to_vec();
    if body.len() > n {
        return Err("正文长度与 Content-Length 不符".into());
    }
    while body.len() < n {
        let left = n - body.len();
        let cap = left.min(buf.len());
        let count = stream.read(&mut buf[..cap]).map_err(|e| e.to_string())?;
        if count == 0 {
            return Err("POST 正文未完整到达".into());
        }
        body.extend_from_slice(&buf[..count]);
    }
    if parts[0] == "GET" && n > 0 {
        return Err("GET 不应包含请求正文".into());
    }
    Ok(Request {
        method: parts[0].into(),
        path: parts[1].into(),
        body,
        host: host.ok_or("缺少 Host")?,
        origin,
        client_header,
        content_type,
    })
}
fn reply(stream: &mut TcpStream, status: &str, typ: &str, body: &[u8]) -> Result<(), String> {
    let wire=format!("HTTP/1.1 {status}\r\nContent-Type: {typ}\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nReferrer-Policy: no-referrer\r\nContent-Security-Policy: default-src 'none'; script-src 'unsafe-inline'; style-src 'unsafe-inline'; connect-src 'self'; form-action 'self'\r\n\r\n",body.len());
    stream
        .write_all(wire.as_bytes())
        .and_then(|_| stream.write_all(body))
        .map_err(|e| e.to_string())
}
fn percent_decode(s: &str) -> Result<String, String> {
    let mut out = Vec::new();
    let mut i = 0;
    let b = s.as_bytes();
    while i < b.len() {
        match b[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' => {
                if i + 2 >= b.len() {
                    return Err("不完整的百分号编码".into());
                }
                let n = |c: u8| -> Option<u8> {
                    match c {
                        b'0'..=b'9' => Some(c - b'0'),
                        b'a'..=b'f' => Some(c - b'a' + 10),
                        b'A'..=b'F' => Some(c - b'A' + 10),
                        _ => None,
                    }
                };
                let hi = n(b[i + 1]).ok_or("非法百分号编码")?;
                let lo = n(b[i + 2]).ok_or("非法百分号编码")?;
                out.push((hi << 4) | lo);
                i += 3;
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    String::from_utf8(out).map_err(|_| "表单不是 UTF-8".into())
}
fn form_value(body: &[u8], expected: &str) -> Result<String, String> {
    let s = std::str::from_utf8(body).map_err(|_| "表单编码错误")?;
    let pieces = s.split('&').collect::<Vec<_>>();
    if pieces.len() != 1 {
        return Err("每次只允许一个指定字段".into());
    }
    let (k, v) = pieces[0].split_once('=').ok_or("表单格式应为 key=value")?;
    if k != expected {
        return Err("表单字段不匹配".into());
    }
    percent_decode(v)
}
pub fn handle(stream: &mut TcpStream, port: u16, journal: &Path) -> Result<(), String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_secs(3)))
        .map_err(|e| e.to_string())?;
    let request = match parse_request(stream) {
        Ok(r) => r,
        Err(e) => {
            reply(
                stream,
                "400 Bad Request",
                "text/plain; charset=utf-8",
                e.as_bytes(),
            )?;
            return Ok(());
        }
    };
    let valid_host =
        request.host == format!("127.0.0.1:{port}") || request.host == format!("localhost:{port}");
    if !valid_host {
        reply(
            stream,
            "403 Forbidden",
            "text/plain; charset=utf-8",
            b"Host not allowed",
        )?;
        return Ok(());
    }
    let accept_origin = request.origin.as_ref().map_or(true, |o| {
        o == &format!("http://127.0.0.1:{port}") || o == &format!("http://localhost:{port}")
    });
    if request.method == "POST" {
        let valid_form = request.content_type.as_ref().is_some_and(|x| {
            x.to_ascii_lowercase()
                .starts_with("application/x-www-form-urlencoded")
        });
        if !request.client_header || !accept_origin || !valid_form {
            reply(
                stream,
                "403 Forbidden",
                "text/plain; charset=utf-8",
                b"Write request rejected",
            )?;
            return Ok(());
        }
    }
    match (request.method.as_str(), request.path.as_str()) {
        ("GET", "/") => reply(
            stream,
            "200 OK",
            "text/html; charset=utf-8",
            PAGE.as_bytes(),
        ),
        ("GET", "/api/tickets") => match servicedesk::load(journal) {
            Ok(t) => reply(
                stream,
                "200 OK",
                "application/json; charset=utf-8",
                servicedesk::render_json(&t).as_bytes(),
            ),
            Err(_) => reply(
                stream,
                "500 Internal Server Error",
                "text/plain; charset=utf-8",
                b"Cannot read ticket log",
            ),
        },
        ("POST", "/api/tickets") => {
            let val = form_value(&request.body, "title")
                .and_then(|t| servicedesk::persist(journal, Action::Add(t)));
            match val {
                Ok(ticket) => reply(
                    stream,
                    "201 Created",
                    "application/json; charset=utf-8",
                    servicedesk::render_json(&[ticket]).as_bytes(),
                ),
                Err(e) => reply(
                    stream,
                    "400 Bad Request",
                    "text/plain; charset=utf-8",
                    e.as_bytes(),
                ),
            }
        }
        ("POST", "/api/close") => {
            let val = form_value(&request.body, "id")
                .and_then(|id| id.parse::<u64>().map_err(|_| "id 必须是整数".to_string()))
                .and_then(|id| servicedesk::persist(journal, Action::Close(id)));
            match val {
                Ok(ticket) => reply(
                    stream,
                    "200 OK",
                    "application/json; charset=utf-8",
                    servicedesk::render_json(&[ticket]).as_bytes(),
                ),
                Err(e) => reply(
                    stream,
                    "400 Bad Request",
                    "text/plain; charset=utf-8",
                    e.as_bytes(),
                ),
            }
        }
        _ => reply(
            stream,
            "404 Not Found",
            "text/plain; charset=utf-8",
            b"Not Found",
        ),
    }
}
