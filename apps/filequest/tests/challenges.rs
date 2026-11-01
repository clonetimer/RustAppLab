//! 这些是“尚未实现”的真实产品扩展验收测试。默认忽略，不影响现有可用软件。
//! 每完成一章的扩展任务，单独运行对应测试，体验 RED -> GREEN。
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static SEQUENCE: AtomicUsize = AtomicUsize::new(0);

fn command(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_filequest"))
        .args(args)
        .output()
        .expect("执行 FileQuest")
}

struct Sandbox(PathBuf);

impl Sandbox {
    fn new() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let unique = SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "filequest-challenge-{}-{nanos}-{unique}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        Self(root)
    }
    fn path(&self) -> &Path {
        &self.0
    }
    fn create(&self, relative: &str, content: &str) {
        let target = self.path().join(relative);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(target, content).unwrap();
    }
    fn arg(&self) -> String {
        self.path().to_string_lossy().into_owned()
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
#[ignore = "任务 01：需要学员添加 version 命令"]
fn mission_01_version_command() {
    let result = command(&["version"]);
    assert!(result.status.success());
    assert!(String::from_utf8_lossy(&result.stdout).contains("FileQuest 0.1.0"));
}

#[test]
#[ignore = "任务 02：需要学员实现 --ext 扫描过滤"]
fn mission_02_scan_extension_filter() {
    let box_ = Sandbox::new();
    box_.create("a.rs", "fn main() {}\n");
    box_.create("b.rs", "fn f() {}\n");
    box_.create("readme.md", "hello\n");
    let result = command(&["scan", &box_.arg(), "--ext", "rs"]);
    assert!(result.status.success());
    let output = String::from_utf8_lossy(&result.stdout);
    assert!(output.contains("普通文件：2"), "输出：{output}");
}

#[test]
#[ignore = "任务 04：需要学员输出每种扩展名的字节合计"]
fn mission_04_extension_size_summary() {
    let box_ = Sandbox::new();
    box_.create("a.log", "12345");
    box_.create("b.log", "1234567");
    let result = command(&["scan", &box_.arg()]);
    assert!(result.status.success());
    let output = String::from_utf8_lossy(&result.stdout);
    assert!(output.contains("log：12 B"), "输出：{output}");
}

#[test]
#[ignore = "任务 05：需要学员增加 --name-only 查找模式"]
fn mission_05_find_basename_only() {
    let box_ = Sandbox::new();
    box_.create("logs/a.txt", "hello");
    let result = command(&["find", &box_.arg(), "logs", "--name-only"]);
    assert!(result.status.success());
    let output = String::from_utf8_lossy(&result.stdout);
    assert!(output.contains("共找到 0 个文件"), "输出：{output}");
}

#[test]
#[ignore = "任务 06：需要学员支持报告标题并正确 HTML 转义"]
fn mission_06_custom_report_title_is_escaped() {
    let box_ = Sandbox::new();
    box_.create("readme.md", "hello");
    let output_path = box_.path().join("outside.html");
    let path_arg = output_path.to_string_lossy().into_owned();
    let result = command(&[
        "report",
        &box_.arg(),
        "--output",
        &path_arg,
        "--title",
        "<b>Archive</b>",
    ]);
    assert!(result.status.success());
    let html = fs::read_to_string(output_path).unwrap();
    assert!(html.contains("&lt;b&gt;Archive&lt;/b&gt;"));
    assert!(!html.contains("<b>Archive</b>"));
}
