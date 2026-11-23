use std::process::Command;

#[test]
fn workbench_help_describes_loopback() {
    let output = Command::new(env!("CARGO_BIN_EXE_labquest"))
        .args(["workbench", "--help"])
        .output()
        .unwrap();
    let help = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success() && help.contains("127.0.0.1"),
        "workbench --help should describe its loopback address: {output:?}"
    );
}

use std::fs;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

struct Server {
    child: Child,
    workspace: PathBuf,
    address: String,
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = fs::remove_dir_all(&self.workspace);
    }
}

impl Server {
    fn start() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let workspace =
            std::env::temp_dir().join(format!("工作台 e2e-{}-{nonce}", std::process::id()));
        fs::create_dir(&workspace).unwrap();
        let log = workspace.join("server.log");
        let child = Command::new(env!("CARGO_BIN_EXE_labquest"))
            .args(["workbench", "--port", "0", "--workspace"])
            .arg(&workspace)
            .current_dir(&workspace)
            .env("CARGO_TARGET_DIR", workspace.join("test-target"))
            .stdin(Stdio::null())
            .stdout(fs::File::create(&log).unwrap())
            .spawn()
            .unwrap();
        let mut server = Self {
            child,
            workspace,
            address: String::new(),
        };
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            let output = fs::read_to_string(&log).unwrap();
            if let Some(url) = output
                .lines()
                .find_map(|line| line.strip_prefix("RUSTAPPLAB_WORKBENCH_URL=http://"))
            {
                assert!(url.starts_with("127.0.0.1:") && url.ends_with('/'));
                server.address = url.trim_end_matches('/').to_string();
                break;
            }
            assert!(
                server.child.try_wait().unwrap().is_none() && Instant::now() < deadline,
                "startup failed: {output}"
            );
            sleep(Duration::from_millis(20));
        }
        assert_eq!(server.request("GET", "/api/health", "").0, 200);
        server
    }
    fn request(&self, method: &str, path: &str, body: &str) -> (u16, String) {
        let mut stream = TcpStream::connect(&self.address).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(15)))
            .unwrap();
        write!(stream, "{method} {path} HTTP/1.1\r\nHost: {host}\r\nOrigin: http://{host}\r\nX-RustAppLab: workbench\r\nContent-Type: application/x-www-form-urlencoded; charset=UTF-8\r\nContent-Length: {size}\r\nConnection: close\r\n\r\n{body}", host=self.address, size=body.len()).unwrap();
        stream.shutdown(Shutdown::Write).unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        let (headers, body) = response.split_once("\r\n\r\n").unwrap();
        (
            headers.split_whitespace().nth(1).unwrap().parse().unwrap(),
            body.to_string(),
        )
    }
    fn post(&self, path: &str, fields: &[(&str, &str)]) -> (u16, String) {
        let data = fields
            .iter()
            .map(|(key, value)| format!("{key}={}", encode(value)))
            .collect::<Vec<_>>()
            .join("&");
        self.request("POST", path, &data)
    }
    fn get(&self, path: &str) -> String {
        let (code, body) = self.request("GET", path, "");
        assert_eq!(code, 200, "{path}: {body}");
        body
    }
    fn save(&self, id: &str, file: &str, content: &str, revision: &str) -> (u16, String) {
        self.post(
            "/api/save",
            &[
                ("id", id),
                ("path", file),
                ("content", content),
                ("revision", revision),
            ],
        )
    }
    fn job(&self, id: &str, action: &str, step: &str, args: &str) -> String {
        let (code, started) = self.post(
            "/api/action",
            &[
                ("id", id),
                ("action", action),
                ("step", step),
                ("args", args),
            ],
        );
        assert!((200..300).contains(&code), "{code}: {started}");
        let number = scalar(&started, "id");
        let deadline = Instant::now() + Duration::from_secs(120);
        loop {
            let body = self.get(&format!("/api/job?id={number}"));
            if scalar(&body, "status") != "running" {
                return body;
            }
            assert!(Instant::now() < deadline, "job timed out: {body}");
            sleep(Duration::from_millis(50));
        }
    }
}

fn encode(value: &str) -> String {
    use std::fmt::Write;
    let mut result = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() {
            result.push(char::from(byte));
        } else {
            write!(result, "%{byte:02X}").unwrap();
        }
    }
    result
}

// Only read known ASCII scalar fields in test responses. Learner source and
// compiler diagnostics are asserted against the real file / raw response.
fn scalar<'a>(body: &'a str, key: &str) -> &'a str {
    let value = body
        .split_once(&format!("\"{key}\""))
        .unwrap_or_else(|| panic!("missing {key}: {body}"))
        .1
        .trim_start()
        .strip_prefix(':')
        .unwrap()
        .trim_start();
    if let Some(text) = value.strip_prefix('"') {
        text.split('"').next().unwrap()
    } else {
        value.split([',', '}']).next().unwrap().trim()
    }
}

fn assert_job(job: &str, action: &str, success: bool) {
    assert_eq!(scalar(job, "action"), action, "{job}");
    assert_eq!(
        scalar(job, "status"),
        if success { "succeeded" } else { "failed" },
        "{job}"
    );
    assert_eq!(scalar(job, "exit_code") == "0", success, "{job}");
    assert_eq!(scalar(job, "stale"), "false", "{job}");
}

fn initialize_old_project(command: &[&str], path: &Path) {
    let result = Command::new(env!("CARGO_BIN_EXE_labquest"))
        .args(command)
        .arg(path)
        .output()
        .unwrap();
    assert!(result.status.success(), "{result:?}");
}

#[test]
fn firstcard_import_save_conflict_check_and_personal_run() {
    let s = Server::start();
    let path = s.workspace.join("我自己的 First Card");
    initialize_old_project(&["beginner", "start"], &path);
    let source = path.join("src/main.rs");
    let original = fs::read_to_string(&source).unwrap();
    // Projects created before newer learning-entry documents still work.
    fs::remove_file(path.join("README.md")).unwrap();
    let (_, opened) = s.post(
        "/api/open",
        &[("id", "firstcard"), ("path", path.to_str().unwrap())],
    );
    assert_eq!(scalar(&opened, "id"), "firstcard");
    assert_eq!(fs::read_to_string(&source).unwrap(), original);
    assert!(!path.join("README.md").exists());
    let ordinary = s.job("firstcard", "run", "1", "小林\n本周小事");
    assert_job(&ordinary, "run", true);
    assert!(ordinary.contains("等待完成第一张任务卡"));
    assert!(!path.join("FIRST_WIN.md").exists());
    assert_job(&s.job("firstcard", "check", "1", ""), "check", false);
    let file_url = "/api/file?id=firstcard&path=src%2Fmain.rs";
    let file = s.get(file_url);
    assert!(file.contains("等待完成第一张任务卡"));
    let draft = original.replace("等待完成第一张任务卡", "今日任务卡 | {name} | {task}");
    let external = format!("{original}\n// 外部编辑\n");
    fs::write(&source, &external).unwrap();
    assert_eq!(
        s.save(
            "firstcard",
            "src/main.rs",
            &draft,
            scalar(&file, "revision")
        )
        .0,
        409
    );
    assert_eq!(fs::read_to_string(&source).unwrap(), external);
    let latest = s.get(file_url);
    let (code, saved) = s.save(
        "firstcard",
        "src/main.rs",
        &draft,
        scalar(&latest, "revision"),
    );
    assert_eq!(code, 200, "{saved}");
    assert_eq!(fs::read_to_string(&source).unwrap(), draft);
    assert_job(&s.job("firstcard", "check", "1", ""), "check", true);
    let personal = s.job("firstcard", "run", "1", "小林\n本周小事");
    assert_job(&personal, "run", true);
    assert!(
        personal.contains("今日任务卡 | 小林 | 本周小事"),
        "{personal}"
    );
}

#[test]
fn unitlab_create_refuses_overwrite_and_checks_each_real_step() {
    let s = Server::start();
    let (code, created) = s.post("/api/create", &[("id", "unitlab")]);
    assert!((200..300).contains(&code), "{created}");
    let path = fs::read_dir(&s.workspace)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|p| p.join(".applab-studio").is_file())
        .unwrap();
    let source = path.join("src/lib.rs");
    let original = fs::read_to_string(&source).unwrap();
    assert!(s.post("/api/create", &[("id", "unitlab")]).0 >= 400);
    assert_eq!(fs::read_to_string(&source).unwrap(), original);
    assert_job(&s.job("unitlab", "check", "1", ""), "check", false);
    let first_step = original.replace("let _ = celsius;\n    0.0", "celsius * 9.0 / 5.0 + 32.0");
    assert_ne!(first_step, original);
    let file = s.get("/api/file?id=unitlab&path=src%2Flib.rs");
    assert_eq!(
        s.save(
            "unitlab",
            "src/lib.rs",
            &first_step,
            scalar(&file, "revision")
        )
        .0,
        200
    );
    assert_eq!(fs::read_to_string(&source).unwrap(), first_step);
    let passed = s.job("unitlab", "check", "1", "");
    assert_job(&passed, "check", true);
    assert_eq!(scalar(&passed, "step"), "1");
    let failed = s.job("unitlab", "check", "2", "");
    assert_job(&failed, "check", false);
    assert_eq!(scalar(&failed, "step"), "2");
    let course = s.get("/api/course?id=unitlab&step=2");
    assert_eq!(scalar(&course, "passed_steps"), "1", "{course}");
    assert_eq!(scalar(&course, "same_source"), "true", "{course}");
}

#[test]
fn running_job_stays_responsive_and_detects_external_changes() {
    let s = Server::start();
    let path = s.workspace.join("临时 run");
    initialize_old_project(&["beginner", "start"], &path);
    let (code, body) = s.post(
        "/api/open",
        &[("id", "firstcard"), ("path", path.to_str().unwrap())],
    );
    assert_eq!(code, 200, "{body}");
    let file = path.join("src/main.rs");
    fs::write(&file, "fn main() { std::thread::sleep(std::time::Duration::from_secs(1)); println!(\"personal output\"); }").unwrap();
    let fields = [
        ("id", "firstcard"),
        ("action", "run"),
        ("step", "1"),
        ("args", ""),
    ];
    let (code, job) = s.post("/api/action", &fields);
    assert!((200..300).contains(&code), "{job}");
    assert_eq!(scalar(&job, "status"), "running");
    let poll = format!("/api/job?id={}", scalar(&job, "id"));
    assert_eq!(s.post("/api/action", &fields).0, 409);
    assert_eq!(s.request("GET", "/api/health", "").0, 200);
    // An external edit to an unused module changes the project revision while
    // preserving the running program's behavior, even during a cold build.
    fs::write(path.join("src/later.rs"), "// changed during run\n").unwrap();
    let deadline = Instant::now() + Duration::from_secs(120);
    loop {
        let finished = s.get(&poll);
        if scalar(&finished, "status") != "running" {
            assert_eq!(scalar(&finished, "status"), "succeeded", "{finished}");
            assert_eq!(scalar(&finished, "stale"), "true", "{finished}");
            assert!(finished.contains("personal output"));
            break;
        }
        assert!(Instant::now() < deadline);
        sleep(Duration::from_millis(50));
    }
}

#[test]
fn old_tab_cannot_save_or_run_after_switching_to_another_copy() {
    let s = Server::start();
    let previous = s.workspace.join("第一份 FirstCard");
    let selected = s.workspace.join("第二份 FirstCard");
    for path in [&previous, &selected] {
        initialize_old_project(&["beginner", "start"], path);
    }
    let open = |path: &Path| {
        let (code, body) = s.post(
            "/api/open",
            &[("id", "firstcard"), ("path", path.to_str().unwrap())],
        );
        assert_eq!(code, 200, "{body}");
    };
    open(&previous);
    let file = s.get("/api/file?id=firstcard&path=src%2Fmain.rs");
    open(&selected);
    let original = fs::read(selected.join("src/main.rs")).unwrap();
    let conflict = s.save(
        "firstcard",
        "src/main.rs",
        "// draft from first copy",
        scalar(&file, "revision"),
    );
    assert_eq!(conflict.0, 409, "{}", conflict.1);
    assert_eq!(original, fs::read(selected.join("src/main.rs")).unwrap());
    let (code, error) = s.post(
        "/api/action",
        &[
            ("id", "firstcard"),
            ("action", "run"),
            ("step", "1"),
            ("args", ""),
            ("project_path", previous.to_str().unwrap()),
        ],
    );
    assert_eq!(code, 409, "{error}");
    assert!(!selected.join("target").exists());
}
