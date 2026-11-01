//! NetFetch: loopback-only HTTP/1.1 downloader (synchronous foundation, NOT async/TLS).
//! The deliberately limited protocol boundary is visible in tests and course notes.
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpStream},
    path::{Path, PathBuf},
    time::Duration,
};

pub const MAX_BODY: usize = 2 * 1024 * 1024;
const MAX_HEADERS: usize = 16 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalUrl {
    pub host: String,
    pub port: u16,
    pub path: String,
}
impl LocalUrl {
    fn address(&self) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), self.port)
    }
}

pub fn parse_url(text: &str) -> Result<LocalUrl, String> {
    // LESSON 1 BEGIN
    let rest = text
        .strip_prefix("http://")
        .ok_or("仅支持 http:// 本机地址，不支持 HTTPS 或其它协议")?;
    if rest.contains(['#', '\\', '\r', '\n', '\0']) || rest.chars().any(char::is_control) {
        return Err("URL 含有不允许的控制字符、反斜杠或片段".into());
    }
    let split = rest.find(['/', '?']).unwrap_or(rest.len());
    let authority = &rest[..split];
    let tail = &rest[split..];
    let (host, port_str) = if let Some(p) = authority.strip_prefix("[::1]:") {
        ("[::1]", Some(p))
    } else if authority == "[::1]" {
        ("[::1]", None)
    } else if let Some((h, p)) = authority.split_once(':') {
        (h, Some(p))
    } else {
        (authority, None)
    };
    if !matches!(host, "127.0.0.1" | "localhost" | "[::1]") {
        return Err("为保护学员文件，仅允许 127.0.0.1、localhost 或 [::1]".into());
    }
    let port = match port_str {
        Some(p) if !p.is_empty() && p.bytes().all(|c| c.is_ascii_digit()) => {
            p.parse::<u16>().map_err(|_| "端口超出范围")?
        }
        Some(_) => return Err("端口必须是 1–65535 的整数".into()),
        None => 80,
    };
    if port == 0 {
        return Err("端口不能为 0".into());
    }
    let path = if tail.is_empty() {
        "/".to_string()
    } else if tail.starts_with('?') {
        format!("/{tail}")
    } else {
        tail.to_string()
    };
    if !path.starts_with('/') || path.contains(' ') || !path.is_ascii() || path.contains("//") {
        return Err("HTTP 请求路径必须是 ASCII、无空格的 /path；中文请进行 URL 编码".into());
    }
    Ok(LocalUrl {
        host: host.into(),
        port,
        path,
    })
    // LESSON 1 END
}

pub fn parse_response(bytes: &[u8]) -> Result<Vec<u8>, String> {
    // LESSON 2 BEGIN
    let boundary = bytes
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or("HTTP 响应缺少头部结束符")?;
    if boundary > MAX_HEADERS {
        return Err("HTTP 响应头过长".into());
    }
    let head = std::str::from_utf8(&bytes[..boundary]).map_err(|_| "HTTP 响应头不是 UTF-8")?;
    let mut lines = head.split("\r\n");
    let status = lines.next().ok_or("缺少 HTTP 状态行")?;
    if status != "HTTP/1.1 200 OK" && status != "HTTP/1.0 200 OK" {
        return Err(format!("下载失败，服务器返回状态：{status}"));
    }
    let mut declared = None;
    for line in lines {
        let (key, value) = line.split_once(':').ok_or("HTTP 头格式错误")?;
        if key.trim() != key
            || key.is_empty()
            || key.bytes().any(|b| !b.is_ascii_alphanumeric() && b != b'-')
        {
            return Err("HTTP 响应头字段名错误".into());
        }
        let val = value.trim();
        if key.eq_ignore_ascii_case("transfer-encoding") {
            return Err("暂不支持 chunked 传输；请使用本地内容长度响应".into());
        }
        if key.eq_ignore_ascii_case("content-length") {
            if declared.is_some() || !val.bytes().all(|b| b.is_ascii_digit()) || val.is_empty() {
                return Err("重复或非法 Content-Length".into());
            }
            declared = Some(
                val.parse::<usize>()
                    .map_err(|_| "Content-Length 超出范围")?,
            );
        }
    }
    let body = &bytes[boundary + 4..];
    if body.len() > MAX_BODY {
        return Err("响应内容超过 2 MiB".into());
    }
    if let Some(n) = declared {
        if n != body.len() {
            return Err(format!(
                "Content-Length={n}，实际接收={}，响应不完整或有多余字节",
                body.len()
            ));
        }
    }
    Ok(body.to_vec())
    // LESSON 2 END
}

pub fn request(url: &str) -> Result<Vec<u8>, String> {
    let target = parse_url(url)?;
    // DNS is not consulted: 'localhost' always connects to a literal loopback address.
    let addr = if target.host == "[::1]" {
        SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), target.port)
    } else {
        target.address()
    };
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(3))
        .map_err(|e| format!("连接 {addr} 失败：{e}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_secs(3)))
        .map_err(|e| e.to_string())?;
    let wire = format!(
        "GET {} HTTP/1.1\r\nHost: {}:{}\r\nConnection: close\r\nAccept: */*\r\n\r\n",
        target.path, target.host, target.port
    );
    stream
        .write_all(wire.as_bytes())
        .map_err(|e| format!("发送请求失败：{e}"))?;
    let mut received = Vec::new();
    stream
        .take((MAX_BODY + MAX_HEADERS + 4096) as u64)
        .read_to_end(&mut received)
        .map_err(|e| format!("读取响应失败：{e}"))?;
    if received.len() >= MAX_BODY + MAX_HEADERS + 4096 {
        return Err("响应超过安全读取上限".into());
    }
    parse_response(&received)
}

pub fn download(url: &str, output: &Path) -> Result<usize, String> {
    // LESSON 3 BEGIN
    let parent = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let meta = fs::symlink_metadata(parent).map_err(|e| format!("输出父目录不存在：{e}"))?;
    if !meta.is_dir() || meta.file_type().is_symlink() {
        return Err("输出父路径必须是普通目录".into());
    }
    if fs::symlink_metadata(output).is_ok() {
        return Err(format!("目标已存在，不覆盖：{}", output.display()));
    }
    let data = request(url)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .map_err(|e| format!("创建目标失败（不会覆盖）：{e}"))?;
    let write_result = file.write_all(&data).and_then(|_| file.sync_all());
    drop(file);
    if let Err(e) = write_result {
        let _ = fs::remove_file(output);
        return Err(format!("写入未成功：{e}"));
    }
    Ok(data.len())
    // LESSON 3 END
}

fn file_name(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name.len() > 64
        || matches!(name, "." | "..")
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_'))
    {
        return Err(
            "输出文件名只能使用 ASCII 字母、数字、点、下划线、短横线，且不能为 . 或 ..".into(),
        );
    }
    Ok(())
}
struct Stage(PathBuf);
impl Drop for Stage {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
/// Atomically publishes a multi-file result only if all requests succeed.
pub fn batch(manifest: &str, destination: &Path) -> Result<Vec<(String, usize)>, String> {
    let mut jobs = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for (i, line) in manifest.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let (name, url) = line
            .split_once('\t')
            .ok_or_else(|| format!("清单第 {} 行需要 文件名<TAB>URL", i + 1))?;
        file_name(name)?;
        parse_url(url)?;
        if !seen.insert(name.to_string()) {
            return Err("清单含重复输出文件名".into());
        }
        if jobs.len() == 31 {
            return Err("批量最多 32 个文件".into());
        }
        jobs.push((name.to_owned(), url.to_owned()));
    }
    if jobs.is_empty() {
        return Err("清单没有任何任务".into());
    }
    let parent = destination
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if !parent.is_dir() {
        return Err("批量输出的父目录不存在".into());
    }
    if fs::symlink_metadata(destination).is_ok() {
        return Err("批量目标目录已存在，不会覆盖".into());
    }
    let name = destination
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("目标目录名称无效")?;
    let stage = parent.join(format!(
        ".{name}.netfetch-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos()
    ));
    fs::create_dir(&stage).map_err(|e| e.to_string())?;
    let guard = Stage(stage);
    let mut report = Vec::new();
    for (name, url) in jobs {
        let n = download(&url, &guard.0.join(&name))?;
        report.push((name, n));
    }
    if fs::symlink_metadata(destination).is_ok() {
        return Err("批量目标创建期间被占用，拒绝提交".into());
    }
    fs::rename(&guard.0, destination).map_err(|e| format!("提交结果失败：{e}"))?;
    Ok(report)
}
