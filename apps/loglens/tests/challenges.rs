//! 教学扩展任务：默认 ignored，学生完成对应产品能力后才逐个启用。
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_loglens"))
        .args(args)
        .output()
        .unwrap()
}

struct Sandbox(PathBuf);
impl Sandbox {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("loglens-challenge-{}-{stamp}", std::process::id()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn write(&self, file: &str, text: &str) -> String {
        let path = self.0.join(file);
        fs::write(&path, text).unwrap();
        path.to_string_lossy().into_owned()
    }
}
impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
#[ignore = "任务 08：为 analyze 添加 --top N"]
fn mission_08_analyze_top_count() {
    let box_ = Sandbox::new();
    let path = box_.write("data.log", "INFO alpha\nINFO beta\nINFO beta\nINFO gamma\n");
    let result = run(&["analyze", &path, "--top", "1"]);
    assert!(result.status.success());
    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(stdout.contains("beta"));
    assert!(!stdout.contains("alpha"));
    assert!(!stdout.contains("gamma"));
}

#[test]
#[ignore = "任务 09：增加 --contains 关键词过滤"]
fn mission_09_filter_message() {
    let box_ = Sandbox::new();
    let path = box_.write(
        "data.log",
        "INFO accepted\nERROR cache timeout\nERROR database unavailable\n",
    );
    let result = run(&["analyze", &path, "--contains", "cache"]);
    assert!(result.status.success());
    let output = String::from_utf8_lossy(&result.stdout);
    assert!(output.contains("已识别：1"), "{output}");
}

#[test]
#[ignore = "任务 10：比较错误率回退时可选择失败退出"]
fn mission_10_fail_on_regression() {
    let box_ = Sandbox::new();
    let before = box_.write("before.log", "INFO ok\nINFO ok\n");
    let after = box_.write("after.log", "INFO ok\nERROR broken\n");
    let result = run(&["compare", &before, &after, "--fail-on-regression"]);
    assert!(!result.status.success());
    let output = String::from_utf8_lossy(&result.stderr);
    assert!(output.contains("错误率上升"), "{output}");
}

#[test]
#[ignore = "任务 11：支持报告自定义标题，并防止 Markdown 注入"]
fn mission_11_report_custom_title() {
    let box_ = Sandbox::new();
    let input = box_.write("data.log", "INFO ok\n");
    let output = box_.0.join("custom.md").to_string_lossy().into_owned();
    let result = run(&["report", &input, "-o", &output, "--title", "<b>build</b>"]);
    assert!(result.status.success());
    let md = fs::read_to_string(output).unwrap();
    assert!(md.contains("&lt;b&gt;build&lt;/b&gt;"));
    assert!(!md.contains("<b>build</b>"));
}
