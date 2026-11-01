//! 将真实日志文件作为输入验证，而不是只测试内部类型。
use loglens::{analyzer, demo, parser::Level, report};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Sandbox(PathBuf);
impl Sandbox {
    fn new() -> Self {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("loglens-test-{}-{time}-{id}", std::process::id()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn path(&self) -> &Path {
        &self.0
    }
    fn write(&self, name: &str, text: &str) -> PathBuf {
        let path = self.path().join(name);
        fs::write(&path, text).unwrap();
        path
    }
}
impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn can_compare_a_real_demo_release_regression() {
    let sandbox = Sandbox::new();
    let (before, after) = demo::create_demo(&sandbox.path().join("practice")).unwrap();
    let baseline = analyzer::analyze_file(&before, false).unwrap();
    let current = analyzer::analyze_file(&after, false).unwrap();
    assert_eq!(baseline.records, 8);
    assert_eq!(baseline.errors(), 1);
    assert_eq!(current.records, 8);
    assert_eq!(current.errors(), 3);
    assert_eq!(current.count(Level::Warn), 2);
    assert_eq!(current.skipped_lines, 1);
    assert_eq!(current.examples[0].line_number, 9);
    assert_eq!(
        current.first_timestamp.as_deref(),
        Some("2026-10-28T10:00:00Z")
    );
    assert_eq!(
        current.last_timestamp.as_deref(),
        Some("2026-10-28T10:00:07Z")
    );
    assert!((current.error_rate() - baseline.error_rate() - 25.0).abs() < 1e-9);
    assert_eq!(current.top_messages(1), vec![("slow query", 2)]);
    assert!(analyzer::analyze_file(&after, true)
        .unwrap_err()
        .contains(":9"));
    assert!(demo::create_demo(&sandbox.path().join("practice")).is_err());
}

#[test]
fn explicit_invalid_inputs_and_empty_files() {
    let sandbox = Sandbox::new();
    let path = sandbox.write("empty.log", "\n   \n");
    let result = analyzer::analyze_file(&path, false).unwrap();
    assert_eq!(result.lines, 2);
    assert_eq!(result.empty_lines, 2);
    assert_eq!(result.error_rate(), 0.0);
    assert!(result.messages.is_empty());
    let invalid = sandbox.write("broken.log", "INFO ok\nnot a log\nERROR failed\n");
    let partial = analyzer::analyze_file(&invalid, false).unwrap();
    assert_eq!(partial.records, 2);
    assert_eq!(partial.skipped_lines, 1);
    assert!(analyzer::analyze_file(&invalid, true)
        .unwrap_err()
        .contains(":2"));
    assert!(analyzer::analyze_file(sandbox.path(), false).is_err());
    assert!(analyzer::analyze_file(&sandbox.path().join("missing.log"), false).is_err());
}

#[test]
fn exports_safe_markdown_without_overwriting_by_default() {
    let sandbox = Sandbox::new();
    let input = sandbox.write(
        "in.log",
        "ERROR <img src=x onerror=y>|ouch\nWARN slow\nINFO ![x](https://invalid.example/img.png)\n",
    );
    let summary = analyzer::analyze_file(&input, false).unwrap();
    let md = report::render(&summary, 5);
    assert!(md.contains("&lt;img src=x onerror=y&gt;\\|ouch"));
    assert!(!md.contains("<img src=x"));
    assert!(!md.contains("![x](https://invalid.example"));
    let output = sandbox.path().join("out.md");
    report::write_report(&output, &md, false).unwrap();
    assert!(report::write_report(&output, "unexpected", false).is_err());
    assert_eq!(fs::read_to_string(&output).unwrap(), md);
    report::write_report(&output, "updated", true).unwrap();
    assert_eq!(fs::read_to_string(&output).unwrap(), "updated");
}

#[cfg(unix)]
#[test]
fn report_force_does_not_follow_symlink() {
    use std::os::unix::fs::symlink;
    let sandbox = Sandbox::new();
    let target = sandbox.write("important.txt", "keep");
    let link = sandbox.path().join("shortcut.md");
    symlink(&target, &link).unwrap();
    assert!(report::write_report(&link, "overwrite", true).is_err());
    assert_eq!(fs::read_to_string(target).unwrap(), "keep");
}

#[cfg(unix)]
#[test]
fn invalid_utf8_is_reported_instead_of_silently_replaced() {
    let sandbox = Sandbox::new();
    let input = sandbox.path().join("bad-encoding.log");
    fs::write(&input, b"INFO ok\nERROR \xff\n").unwrap();
    let error = analyzer::analyze_file(&input, false).unwrap_err();
    assert!(error.contains("第 2 行"), "{error}");
}
