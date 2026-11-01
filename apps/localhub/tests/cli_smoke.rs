use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[test]
fn demo_to_real_http_api_and_process_exit() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("localhub-cli-{}-{stamp}", std::process::id()));
    let exe = PathBuf::from(env!("CARGO_BIN_EXE_localhub"));
    let demo = Command::new(&exe).arg("demo").arg(&root).output().unwrap();
    assert!(
        demo.status.success(),
        "{}",
        String::from_utf8_lossy(&demo.stderr)
    );
    assert!(root.join("files/report.html").is_file());
    assert!(root.join("logs/candidate.log").is_file());
    assert!(
        !Command::new(&exe)
            .arg("demo")
            .arg(&root)
            .output()
            .unwrap()
            .status
            .success(),
        "必须拒绝覆盖已存在的演示目录"
    );
    let mut child = Command::new(&exe)
        .arg("serve")
        .arg("--root")
        .arg(root.join("files"))
        .arg("--log")
        .arg(root.join("logs/candidate.log"))
        .arg("--port")
        .arg("0")
        .arg("--max-requests")
        .arg("2")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut lines = BufReader::new(child.stdout.take().unwrap());
    let mut first = String::new();
    lines.read_line(&mut first).unwrap();
    let port = first
        .trim()
        .split("://127.0.0.1:")
        .nth(1)
        .unwrap()
        .trim_end_matches('/')
        .parse::<u16>()
        .unwrap();
    for (path, expected) in [
        ("/api/health", "\"status\":\"ok\""),
        ("/api/logs", "\"error_rate\":37.50"),
    ] {
        let mut socket = TcpStream::connect(("127.0.0.1", port)).unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        socket
            .write_all(format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n\r\n").as_bytes())
            .unwrap();
        let mut response = String::new();
        socket.read_to_string(&mut response).unwrap();
        assert!(response.contains(expected), "{path}: {response}");
    }
    let result = child.wait().unwrap();
    assert!(result.success(), "服务端退出失败：{result}");
    fs::remove_dir_all(&root).unwrap();
}
