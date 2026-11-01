use servicedesk::{self, Action, Ticket};
use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::Duration,
};
#[path = "../src/http.rs"]
mod http;
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "applab-servicedesk-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
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
#[test]
fn step1_validate_titles() {
    assert_eq!(
        servicedesk::validate_title("  我的需求 ").unwrap(),
        "我的需求"
    );
    assert!(servicedesk::validate_title("  \n ").is_err());
    assert!(servicedesk::validate_title("x".repeat(121).as_str()).is_err());
}
#[test]
fn step1_valid_unicode_and_rejects_control() {
    assert_eq!(
        servicedesk::validate_title("需求：网络✓").unwrap(),
        "需求：网络✓"
    );
    for s in ["a\nb", "\t坏标题", "\u{0000}"] {
        assert!(servicedesk::validate_title(s).is_err());
    }
}
#[test]
fn step2_add_close_and_reject_invalid() {
    let mut data = vec![];
    let a = servicedesk::apply(&mut data, Action::Add("写测试".into())).unwrap();
    assert_eq!(a.id, 1);
    assert_eq!(data.len(), 1);
    assert!(!a.done);
    assert!(
        servicedesk::apply(&mut data, Action::Close(1))
            .unwrap()
            .done
    );
    assert!(servicedesk::apply(&mut data, Action::Close(1)).is_err());
    assert!(servicedesk::apply(&mut data, Action::Close(99)).is_err());
    assert_eq!(data.len(), 1);
}
#[test]
fn step2_storage_persists_and_refuses_torn_log() {
    let t = Temp::new();
    let file = t.0.join("数据.log");
    servicedesk::init(&file).unwrap();
    assert!(servicedesk::init(&file).is_err());
    servicedesk::persist(&file, Action::Add("中文\"标题<>&".into())).unwrap();
    servicedesk::persist(&file, Action::Close(1)).unwrap();
    assert_eq!(servicedesk::load(&file).unwrap()[0].title, "中文\"标题<>&");
    assert!(servicedesk::load(&file).unwrap()[0].done);
    let mut bytes = fs::read(&file).unwrap();
    bytes.extend_from_slice(b"A\t2\t414");
    fs::write(&file, bytes.clone()).unwrap();
    assert!(servicedesk::load(&file).is_err());
    assert!(servicedesk::persist(&file, Action::Add("bad".into())).is_err());
    assert_eq!(fs::read(&file).unwrap(), bytes);
}
#[test]
fn step3_json_escape_and_counts() {
    let input = [
        Ticket {
            id: 1,
            title: "Hello <img> \"quote\" &".into(),
            done: true,
        },
        Ticket {
            id: 2,
            title: "中文".into(),
            done: false,
        },
    ];
    let json = servicedesk::render_json(&input);
    assert!(json.contains("\\u003cimg\\u003e"));
    assert!(json.contains("\\\"quote\\\""));
    assert!(json.contains("\"count\":2"));
    assert!(!json.contains("<img>"));
}
fn call(journal: &std::path::Path, request: impl FnOnce(u16) -> String) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let file = journal.to_owned();
    let join = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        http::handle(&mut stream, port, &file).unwrap();
    });
    let mut client = TcpStream::connect(("127.0.0.1", port)).unwrap();
    client
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    client.write_all(request(port).as_bytes()).unwrap();
    let mut received = String::new();
    client.read_to_string(&mut received).unwrap();
    join.join().unwrap();
    received
}
fn get(p: u16, path: &str) -> String {
    format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{p}\r\nConnection: close\r\n\r\n")
}
fn post(p: u16, path: &str, body: &str, extra: &str) -> String {
    format!("POST {path} HTTP/1.1\r\nHost: 127.0.0.1:{p}\r\nContent-Type: application/x-www-form-urlencoded\r\nX-RustAppLab-Client: 1\r\nContent-Length: {}\r\n{extra}\r\n{body}",body.len())
}
#[test]
fn step3_real_http_get_and_post() {
    let t = Temp::new();
    let file = t.0.join("t.log");
    servicedesk::init(&file).unwrap();
    let home = call(&file, |p| get(p, "/"));
    assert!(home.starts_with("HTTP/1.1 200 OK"));
    assert!(home.contains("ServiceDesk"));
    let added = call(&file, |p| {
        post(p, "/api/tickets", "title=%E4%B8%AD%E6%96%87+task", "")
    });
    assert!(added.starts_with("HTTP/1.1 201 Created"), "{added}");
    assert!(added.contains("中文 task"));
    let list = call(&file, |p| get(p, "/api/tickets"));
    assert!(list.contains("\"count\":1"));
    let closed = call(&file, |p| post(p, "/api/close", "id=1", ""));
    assert!(closed.contains("\"done\":true"));
    assert!(servicedesk::load(&file).unwrap()[0].done);
}
#[test]
fn step3_reject_host_and_browser_cross_origin() {
    let t = Temp::new();
    let file = t.0.join("t.log");
    servicedesk::init(&file).unwrap();
    let resp = call(&file, |_p| {
        "POST /api/tickets HTTP/1.1\r\nHost: evil.example\r\nContent-Length: 7\r\nContent-Type: application/x-www-form-urlencoded\r\nX-RustAppLab-Client: 1\r\n\r\ntitle=x".into()
    });
    assert!(resp.starts_with("HTTP/1.1 403"));
    let resp = call(&file, |p| {
        post(p, "/api/tickets", "title=x", "Origin: http://bad.site\r\n")
    });
    assert!(resp.starts_with("HTTP/1.1 403"));
    let resp = call(&file, |p| {
        format!("POST /api/tickets HTTP/1.1\r\nHost: 127.0.0.1:{p}\r\nContent-Length: 7\r\nContent-Type: application/x-www-form-urlencoded\r\n\r\ntitle=x")
    });
    assert!(resp.starts_with("HTTP/1.1 403"));
    assert!(servicedesk::load(&file).unwrap().is_empty());
}
#[test]
fn step3_reject_bad_body_encoding_and_length() {
    let t = Temp::new();
    let file = t.0.join("t.log");
    servicedesk::init(&file).unwrap();
    let r = call(&file, |p| post(p, "/api/tickets", "title=%ZZ", ""));
    assert!(r.starts_with("HTTP/1.1 400"));
    let r = call(&file, |p| {
        format!("POST /api/tickets HTTP/1.1\r\nHost: 127.0.0.1:{p}\r\nX-RustAppLab-Client: 1\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: 7\r\nContent-Length: 7\r\n\r\ntitle=x")
    });
    assert!(r.starts_with("HTTP/1.1 400"));
    assert!(servicedesk::load(&file).unwrap().is_empty());
}
#[test]
fn step3_cli_demo_is_real() {
    let t = Temp::new();
    let dir = t.0.join("my service");
    let r = std::process::Command::new(env!("CARGO_BIN_EXE_servicedesk"))
        .arg("demo")
        .arg(&dir)
        .output()
        .unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    assert_eq!(
        servicedesk::load(&dir.join("tickets.log")).unwrap().len(),
        2
    );
    let json = fs::read_to_string(dir.join("tickets.json")).unwrap();
    assert!(json.contains("\"count\":2"));
}
#[test]
fn step3_cli_missing_db_is_error() {
    let t = Temp::new();
    let r = std::process::Command::new(env!("CARGO_BIN_EXE_servicedesk"))
        .arg("list")
        .arg(t.0.join("missing.log"))
        .output()
        .unwrap();
    assert!(!r.status.success());
}
