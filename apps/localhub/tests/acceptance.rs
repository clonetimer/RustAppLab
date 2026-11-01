use localhub::{data::Snapshot, http};
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

struct Sandbox(PathBuf);
impl Sandbox {
    fn new() -> Self {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("localhub-acceptance-{}-{n}", std::process::id()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn path(&self) -> &Path {
        &self.0
    }
    fn write(&self, name: &str, content: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, content).unwrap();
        path
    }
}
impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn get(path: &str, port: u16) -> String {
    format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n\r\n")
}

#[test]
fn combines_real_file_and_log_snapshots() {
    let demo = Sandbox::new();
    demo.write("project/src/main.rs", "fn main() {}\n");
    demo.write("project/notes.md", "Hello\n");
    let log = demo.write(
        "candidate.log",
        "INFO ready\nERROR timeout\nFATAL crashed\nnot a log\n",
    );
    let state = Snapshot::load(&demo.path().join("project"), Some(&log)).unwrap();
    let files = state.files_json(1);
    assert!(files.contains("\"files\":2"));
    assert!(files.contains("\"shown\":1,\"truncated\":true"));
    assert!(files.contains("src/main.rs") || files.contains("notes.md"));
    let logs = state.logs_json();
    assert!(logs.contains("\"records\":3"));
    assert!(logs.contains("\"errors\":2"));
    assert!(logs.contains("\"skipped\":1"));
    assert!(logs.contains("\"error_rate\":66.67"));
}

#[test]
fn refuses_invalid_root_and_does_not_follow_root_symlink() {
    let box_ = Sandbox::new();
    assert!(Snapshot::load(&box_.path().join("missing"), None).is_err());
    let folder = box_.path().join("real");
    fs::create_dir(&folder).unwrap();
    assert!(Snapshot::load(&folder, Some(&box_.path().join("not-exists.log"))).is_err());
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let link = box_.path().join("link");
        symlink(&folder, &link).unwrap();
        assert!(Snapshot::load(&link, None).is_err());
    }
}

#[test]
fn routes_are_read_only_limit_validated_and_do_not_accept_browser_paths() {
    let box_ = Sandbox::new();
    box_.write("file.txt", "data");
    let state = Snapshot::load(box_.path(), None).unwrap();
    let run = |request: &str| http::handle_request(request.as_bytes(), &state, 8765);
    for endpoint in [
        "/",
        "/assets/app.css",
        "/assets/app.js",
        "/api/files",
        "/api/files?limit=1",
        "/api/health",
        "/api/logs",
    ] {
        let result = run(&get(endpoint, 8765));
        assert_eq!(result.status, 200, "{endpoint}");
        assert!(!result.body.is_empty());
    }
    for endpoint in [
        "/api/files?limit=0",
        "/api/files?limit=501",
        "/api/files?limit=abc",
        "/api/files?root=/",
        "/api/files?limit=2&root=/",
    ] {
        assert_eq!(run(&get(endpoint, 8765)).status, 400, "{endpoint}");
    }
    assert_eq!(run(&get("/api/unknown", 8765)).status, 404);
    assert_eq!(run(&get("/../../etc/passwd", 8765)).status, 404);
    assert!(String::from_utf8(run(&get("/api/logs", 8765)).body)
        .unwrap()
        .contains("\"enabled\":false"));
    let spoofed = "GET /api/files HTTP/1.1\r\nHost: attacker.example\r\n\r\n";
    assert_eq!(run(spoofed).status, 403);
    assert_eq!(
        run("POST /api/files HTTP/1.1\r\nHost: localhost:8765\r\n\r\n").status,
        405
    );
}

#[test]
fn real_tcp_http_round_trip_and_graceful_limit() {
    let fixture = Sandbox::new();
    fixture.write("src/test.rs", "a");
    let state = Snapshot::load(fixture.path(), None).unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::scope(|scope| {
        let worker = scope.spawn(|| http::serve(listener, &state, Some(3)).unwrap());
        for (path, expected) in [
            ("/", "text/html"),
            ("/api/files?limit=1", "application/json"),
            ("/api/health", "application/json"),
        ] {
            let mut socket = TcpStream::connect(("127.0.0.1", port)).unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            socket.write_all(get(path, port).as_bytes()).unwrap();
            let mut reply = Vec::new();
            socket.read_to_end(&mut reply).unwrap();
            let reply = String::from_utf8(reply).unwrap();
            assert!(reply.starts_with("HTTP/1.1 200 OK"), "{reply}");
            assert!(reply.contains(expected), "{reply}");
            assert!(reply.contains("Content-Security-Policy:"));
            assert!(reply.contains("Connection: close"));
        }
        worker.join().unwrap();
    });
}

#[test]
fn rejects_excessive_request_headers_over_real_tcp() {
    let box_ = Sandbox::new();
    box_.write("hello.md", "hello");
    let state = Snapshot::load(box_.path(), None).unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::scope(|scope| {
        let worker = scope.spawn(|| http::serve(listener, &state, Some(1)).unwrap());
        let mut socket = TcpStream::connect(("127.0.0.1", port)).unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let large = format!(
            "GET / HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nX-Long: {}\r\n\r\n",
            "x".repeat(8192)
        );
        socket.write_all(large.as_bytes()).unwrap();
        let mut reply = Vec::new();
        socket.read_to_end(&mut reply).unwrap();
        assert!(String::from_utf8(reply)
            .unwrap()
            .starts_with("HTTP/1.1 431"));
        worker.join().unwrap();
    });
}

#[test]
fn safely_returns_special_characters_in_json() {
    let box_ = Sandbox::new();
    box_.write("name\\quote.txt", "a");
    #[cfg(unix)]
    box_.write("<img src=x>.rs", "x");
    let state = Snapshot::load(box_.path(), None).unwrap();
    let json = state.files_json(500);
    assert!(json.contains("name\\\\quote.txt"));
    #[cfg(unix)]
    assert!(json.contains("<img src=x>.rs"));
    // 用户可控的名称只以 JSON 字符串输出，前端仅通过 textContent 插入 DOM。
}
