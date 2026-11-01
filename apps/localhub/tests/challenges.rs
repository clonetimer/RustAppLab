//! RED→GREEN 学员关卡。默认忽略；显式 --ignored 才会检验尚未实现的扩展能力。
use localhub::{data::Snapshot, http};
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_root() -> std::path::PathBuf {
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("localhub-mission-{}-{n}", std::process::id()));
    fs::create_dir(&path).unwrap();
    path
}

#[test]
#[ignore = "任务 13：在 /api/files 支持 ?ext=rs 过滤"]
fn mission_13_extension_filter() {
    let root = temp_root();
    fs::write(root.join("main.rs"), "fn main() {}").unwrap();
    fs::write(root.join("notes.md"), "demo").unwrap();
    let data = Snapshot::load(&root, None).unwrap();
    let res = http::handle_request(
        b"GET /api/files?ext=rs HTTP/1.1\r\nHost: localhost:8765\r\n\r\n",
        &data,
        8765,
    );
    assert_eq!(res.status, 200);
    let output = String::from_utf8(res.body).unwrap();
    assert!(output.contains("main.rs") && !output.contains("notes.md"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
#[ignore = "任务 14：支持 ?min_level=ERROR，明确返回 filtered_count"]
fn mission_14_logs_min_level() {
    let root = temp_root();
    let log = root.join("example.log");
    fs::write(&log, "INFO ok\nERROR down\nFATAL gone\n").unwrap();
    let data = Snapshot::load(&root, Some(&log)).unwrap();
    let res = http::handle_request(
        b"GET /api/logs?min_level=ERROR HTTP/1.1\r\nHost: localhost:8765\r\n\r\n",
        &data,
        8765,
    );
    assert_eq!(res.status, 200);
    assert!(String::from_utf8(res.body)
        .unwrap()
        .contains("\"filtered_count\":2"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
#[ignore = "任务 15：增加 --name 自定义健康检查应用名"]
fn mission_15_custom_service_name() {
    let root = temp_root();
    let mut child = Command::new(env!("CARGO_BIN_EXE_localhub"))
        .args(["serve", "--root"])
        .arg(&root)
        .args(["--name", "MyWorkshop", "--port", "0", "--max-requests", "1"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut first = String::new();
    BufReader::new(child.stdout.take().unwrap())
        .read_line(&mut first)
        .unwrap();
    assert!(
        first.contains("127.0.0.1:"),
        "应正确接受 --name 选项并启动本地服务"
    );
    let port = first
        .trim()
        .split("://127.0.0.1:")
        .nth(1)
        .unwrap()
        .trim_end_matches('/')
        .parse::<u16>()
        .unwrap();
    let mut socket = TcpStream::connect(("127.0.0.1", port)).unwrap();
    socket
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    socket
        .write_all(format!("GET /api/health HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n\r\n").as_bytes())
        .unwrap();
    let mut response = String::new();
    socket.read_to_string(&mut response).unwrap();
    assert!(response.contains("\"app\":\"MyWorkshop\""));
    assert!(child.wait().unwrap().success());
    fs::remove_dir_all(root).unwrap();
}
