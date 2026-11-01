//! 一个刻意保持较小规模的 HTTP/1.1 单连接循环：先学习协议边界，再学习异步。
//! 生产级 HTTP、认证、请求并发和 TLS 应使用成熟框架，不应复用此教学服务器。
use crate::data::{json_string, Snapshot};
use std::io::{self, Read, Write as _};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

const MAX_HEADERS: usize = 8192;
const MAX_FILES_RETURNED: usize = 500;
const INDEX: &str = include_str!("ui/index.html");
const STYLES: &str = include_str!("ui/app.css");
const SCRIPT: &str = include_str!("ui/app.js");

#[derive(Debug)]
pub struct Response {
    pub status: u16,
    pub mime: &'static str,
    pub body: Vec<u8>,
}

impl Response {
    fn text(status: u16, mime: &'static str, text: impl Into<String>) -> Self {
        Self {
            status,
            mime,
            body: text.into().into_bytes(),
        }
    }

    fn error(status: u16, detail: &str) -> Self {
        Self::text(
            status,
            "application/json; charset=utf-8",
            format!("{{\"error\":{}}}", json_string(detail)),
        )
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let reason = match self.status {
            200 => "OK",
            400 => "Bad Request",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            408 => "Request Timeout",
            431 => "Request Header Fields Too Large",
            505 => "HTTP Version Not Supported",
            _ => "Internal Server Error",
        };
        let mut header = format!(
            "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nX-Frame-Options: DENY\r\nReferrer-Policy: no-referrer\r\nContent-Security-Policy: default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self'; base-uri 'none'; frame-ancestors 'none'\r\n",
            self.status,
            reason,
            self.mime,
            self.body.len(),
        );
        if self.status == 405 {
            header.push_str("Allow: GET\r\n");
        }
        header.push_str("\r\n");
        let mut bytes = header.into_bytes();
        bytes.extend_from_slice(&self.body);
        bytes
    }
}

#[derive(Debug)]
struct Request<'a> {
    method: &'a str,
    target: &'a str,
}

fn parse_request(header: &str, local_port: u16) -> Result<Request<'_>, Response> {
    let mut lines = header.split("\r\n");
    let first = lines.next().unwrap_or("");
    let parts: Vec<&str> = first.split(' ').collect();
    if parts.len() != 3 || parts.iter().any(|part| part.is_empty()) {
        return Err(Response::error(400, "非法 HTTP 请求行"));
    }
    if parts[2] != "HTTP/1.1" {
        return Err(Response::error(505, "只支持 HTTP/1.1"));
    }
    if !parts[1].starts_with('/') || parts[1].bytes().any(|byte| byte.is_ascii_control()) {
        return Err(Response::error(400, "仅允许 origin-form 请求目标"));
    }
    let mut host = None;
    for line in lines {
        if line.is_empty() {
            break;
        }
        if line.starts_with([' ', '\t']) {
            return Err(Response::error(400, "不支持折行的 HTTP 头"));
        }
        let (name, value) = line
            .split_once(':')
            .ok_or_else(|| Response::error(400, "非法 HTTP 头"))?;
        let value = value.trim();
        if name.eq_ignore_ascii_case("host") && host.replace(value).is_some() {
            return Err(Response::error(400, "Host 头不能重复"));
        }
        if (name.eq_ignore_ascii_case("content-length") && value != "0")
            || name.eq_ignore_ascii_case("transfer-encoding")
        {
            return Err(Response::error(400, "只接受无请求体的 GET"));
        }
    }
    let host = host.ok_or_else(|| Response::error(400, "缺少 Host 头"))?;
    if host != format!("127.0.0.1:{local_port}")
        && !host.eq_ignore_ascii_case(&format!("localhost:{local_port}"))
    {
        return Err(Response::error(
            403,
            "Host 必须是 localhost 或 127.0.0.1 与当前端口",
        ));
    }
    if parts[0] != "GET" {
        return Err(Response::error(405, "此服务器仅实现 GET"));
    }
    Ok(Request {
        method: parts[0],
        target: parts[1],
    })
}

/// 独立、可直接单元测试的分发函数。重要：状态只来自启动时选择的目录与日志。
pub fn handle_request(header_bytes: &[u8], snapshot: &Snapshot, local_port: u16) -> Response {
    let header = match std::str::from_utf8(header_bytes) {
        Ok(value) => value,
        Err(_) => return Response::error(400, "HTTP 头不是有效 UTF-8"),
    };
    let req = match parse_request(header, local_port) {
        Ok(request) => request,
        Err(response) => return response,
    };
    debug_assert_eq!(req.method, "GET");
    match req.target {
        "/" => Response::text(200, "text/html; charset=utf-8", INDEX),
        "/assets/app.css" => Response::text(200, "text/css; charset=utf-8", STYLES),
        "/assets/app.js" => Response::text(200, "text/javascript; charset=utf-8", SCRIPT),
        "/api/health" => Response::text(200, "application/json; charset=utf-8",
            "{\"status\":\"ok\",\"app\":\"localhub\",\"version\":\"0.3.0\",\"snapshot\":\"startup\"}"),
        "/api/logs" => Response::text(200, "application/json; charset=utf-8", snapshot.logs_json()),
        "/api/files" => Response::text(200, "application/json; charset=utf-8", snapshot.files_json(100)),
        target if target.starts_with("/api/files?") => {
            let query = &target["/api/files?".len()..];
            let Some(number) = query.strip_prefix("limit=") else {
                return Response::error(400, "只支持 ?limit=N 参数");
            };
            let Ok(limit) = number.parse::<usize>() else {
                return Response::error(400, "limit 必须是整数");
            };
            if !(1..=MAX_FILES_RETURNED).contains(&limit) {
                return Response::error(400, "limit 必须在 1..=500 之间");
            }
            Response::text(200, "application/json; charset=utf-8", snapshot.files_json(limit))
        }
        target if target.starts_with("/api/") => Response::error(404, "不存在该 API"),
        _ => Response::error(404, "页面不存在"),
    }
}

fn handle_socket(stream: &mut TcpStream, snapshot: &Snapshot, port: u16) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(3)))?;
    stream.set_write_timeout(Some(Duration::from_secs(3)))?;
    let mut buffer = Vec::with_capacity(1024);
    let response = loop {
        let mut chunk = [0u8; 1024];
        match stream.read(&mut chunk) {
            Ok(0) => break Response::error(400, "请求在 HTTP 头结束前关闭"),
            Ok(len) => {
                buffer.extend_from_slice(&chunk[..len]);
                if buffer.len() > MAX_HEADERS {
                    break Response::error(431, "请求头超过 8192 字节");
                }
                if let Some(index) = buffer.windows(4).position(|bytes| bytes == b"\r\n\r\n") {
                    break handle_request(&buffer[..index + 4], snapshot, port);
                }
            }
            Err(err)
                if matches!(
                    err.kind(),
                    io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
                ) =>
            {
                break Response::error(408, "读取请求超时");
            }
            Err(err) => return Err(err),
        }
    };
    stream.write_all(&response.to_bytes())
}

/// 同步、串行 HTTP 服务：支持 --max-requests 使测试可停止，不允许改变监听地址。
pub fn serve(
    listener: TcpListener,
    snapshot: &Snapshot,
    max_requests: Option<usize>,
) -> io::Result<()> {
    let port = listener.local_addr()?.port();
    for (served, incoming) in listener.incoming().enumerate() {
        let mut stream = incoming?;
        if let Err(err) = handle_socket(&mut stream, snapshot, port) {
            eprintln!("LocalHub：处理本地连接失败：{err}");
        }
        if max_requests.is_some_and(|max| served + 1 >= max) {
            break;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_has_valid_http_framing_and_security_headers() {
        let response = Response::text(200, "text/plain", "你好");
        let raw = response.to_bytes();
        let marker = raw.windows(4).position(|part| part == b"\r\n\r\n").unwrap();
        let headers = String::from_utf8(raw[..marker].to_vec()).unwrap();
        assert!(headers.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(headers.contains("Content-Length: 6\r\n"));
        assert!(headers.contains("Content-Security-Policy:"));
        assert_eq!(&raw[marker + 4..], "你好".as_bytes());
    }

    #[test]
    fn rejects_bad_hosts_methods_and_unsupported_http_versions() {
        for (request, code) in [
            (
                "GET /api/health HTTP/1.1\r\nHost: example.org:8765\r\n\r\n",
                403,
            ),
            ("GET /api/health HTTP/1.1\r\n\r\n", 400),
            (
                "GET /api/health HTTP/1.1\r\nHost: localhost:8765\r\nHost: localhost:8765\r\n\r\n",
                400,
            ),
            (
                "POST /api/health HTTP/1.1\r\nHost: localhost:8765\r\n\r\n",
                405,
            ),
            ("GET / HTTP/1.0\r\nHost: localhost:8765\r\n\r\n", 505),
            (
                "GET / HTTP/1.1\r\nHost: localhost:8765\r\nTransfer-Encoding: chunked\r\n\r\n",
                400,
            ),
        ] {
            let result = parse_request(request, 8765);
            assert!(
                matches!(result, Err(Response { status, .. }) if status == code),
                "{request}"
            );
        }
    }

    #[test]
    fn rejects_invalid_target_and_accepts_normal_localhost() {
        let valid = parse_request(
            "GET /api/health HTTP/1.1\r\nHost: LocalHost:8765\r\n\r\n",
            8765,
        )
        .unwrap();
        assert_eq!(valid.target, "/api/health");
        assert!(parse_request(
            "GET http://evil/ HTTP/1.1\r\nHost: localhost:8765\r\n\r\n",
            8765
        )
        .is_err());
    }
}
