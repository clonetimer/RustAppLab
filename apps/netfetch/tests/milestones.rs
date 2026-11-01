use netfetch::*;
use std::{
    fs,
    io::{Read, Write},
    net::TcpListener,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    thread,
};
static COUNTER: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "applab-netfetch-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn fixture(reply: Vec<u8>) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let join = thread::spawn(move || {
        let (mut client, _) = listener.accept().unwrap();
        let mut buf = [0u8; 1024];
        let n = client.read(&mut buf).unwrap();
        assert!(std::str::from_utf8(&buf[..n])
            .unwrap()
            .starts_with("GET /lesson HTTP/1.1"));
        client.write_all(&reply).unwrap();
    });
    (format!("http://127.0.0.1:{port}/lesson"), join)
}
#[test]
fn step1_parses_local_address() {
    let url = parse_url("http://localhost:8080/hello?x=1").unwrap();
    assert_eq!(url.port, 8080);
    assert_eq!(url.path, "/hello?x=1");
    assert_eq!(parse_url("http://127.0.0.1").unwrap().path, "/");
}
#[test]
fn step1_rejects_external_and_injection() {
    for s in [
        "https://localhost:443/x",
        "http://example.com/a",
        "http://localhost@evil/x",
        "http://127.0.0.1:0/x",
        "http://127.0.0.1:65536/x",
        "http://localhost/ab\r\nX-Injected: yes",
        "http://localhost/a b",
    ] {
        assert!(parse_url(s).is_err(), "unexpected acceptance: {s:?}");
    }
}
#[test]
fn step2_parses_real_http_bytes() {
    let input = b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 5\r\n\r\nhello";
    assert_eq!(parse_response(input).unwrap(), b"hello");
}
#[test]
fn step2_refuses_bad_content() {
    for x in [
        b"HTTP/1.1 302 Found\r\nContent-Length: 0\r\n\r\n".as_slice(),
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n0\r\n\r\n",
        b"HTTP/1.1 200 OK\r\nContent-Length: 12\r\n\r\nhi",
        b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nContent-Length: 2\r\n\r\nhi",
    ] {
        assert!(parse_response(x).is_err());
    }
}
#[test]
fn step3_fetches_real_socket_and_preserves_existing() {
    let t = Temp::new();
    let target = t.0.join("真实文件.txt");
    let response =
        b"HTTP/1.1 200 OK\r\nContent-Length: 6\r\nConnection: close\r\n\r\nABC\n12".to_vec();
    let (url, join) = fixture(response);
    assert_eq!(download(&url, &target).unwrap(), 6);
    join.join().unwrap();
    assert_eq!(fs::read(&target).unwrap(), b"ABC\n12");
    assert!(download(&url, &target).is_err());
    assert_eq!(fs::read(&target).unwrap(), b"ABC\n12");
}
#[test]
fn step3_failed_response_creates_nothing() {
    let t = Temp::new();
    let target = t.0.join("no.txt");
    let (url, join) = fixture(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n".to_vec());
    assert!(download(&url, &target).is_err());
    join.join().unwrap();
    assert!(!target.exists());
}
#[test]
fn step3_batch_validates_before_creating_folder() {
    let t = Temp::new();
    let dest = t.0.join("bundle");
    for manifest in [
        "../escape\thttp://localhost:80/\n",
        "good.txt\thttp://evil/\n",
        "one\thttp://localhost/\none\thttp://localhost/\n",
    ] {
        assert!(batch(manifest, &dest).is_err());
        assert!(!dest.exists());
    }
}
#[test]
fn step3_cli_demo_produces_real_files() {
    let t = Temp::new();
    let path = t.0.join("中文 网络课程");
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_netfetch"))
        .arg("demo")
        .arg(&path)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        fs::read_to_string(path.join("files/intro.txt")).unwrap(),
        "你好，Rust 网络！\n"
    );
    assert!(path.join("files/notes.txt").exists());
}
#[test]
fn step3_cli_rejects_external() {
    let t = Temp::new();
    let output = t.0.join("bad");
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_netfetch"))
        .args(["fetch", "http://example.com/test"])
        .arg(&output)
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(!output.exists());
}
