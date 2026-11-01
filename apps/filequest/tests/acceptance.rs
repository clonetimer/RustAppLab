//! 产品层验收：使用真实文件系统创建测试输入，并检查可观察结果。
use filequest::{report, scanner};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static SEQUENCE: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("系统时间应晚于 1970 年")
            .as_nanos();
        let id = SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "filequest-test-{}-{nanos}-{id}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("创建临时测试目录");
        Self(root)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn write(&self, relative: &str, content: &str) {
        let path = self.path().join(relative);
        fs::create_dir_all(path.parent().expect("路径须有父目录")).unwrap();
        fs::write(path, content).unwrap();
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn scans_real_directories_with_explicit_ignore_policy() {
    let fixture = Fixture::new();
    fixture.write("src/main.RS", "fn main() {}\n");
    fixture.write("src/工具.rs", "pub fn ok() {}\n");
    fixture.write("docs/README", "hello\n");
    fixture.write("target/debug/cache", "skip\n");
    fixture.write(".git/config", "skip\n");

    let result = scanner::scan(fixture.path(), false).unwrap();
    assert_eq!(result.files.len(), 3);
    assert_eq!(result.ignored_directories, 2);
    assert_eq!(result.directories, 2);
    assert_eq!(result.extension_counts()[0], ("rs".to_string(), 2));
    assert_eq!(result.find("工具").len(), 1);
    assert_eq!(result.find("MAIN.rs").len(), 1);

    let all = scanner::scan(fixture.path(), true).unwrap();
    assert_eq!(all.files.len(), 5);
    assert_eq!(all.ignored_directories, 0);
}

#[test]
fn does_not_traverse_symlinks() {
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let fixture = Fixture::new();
        fixture.write("nested/main.rs", "hello");
        symlink(
            fixture.path().join("nested"),
            fixture.path().join("shortcut"),
        )
        .unwrap();
        let result = scanner::scan(fixture.path(), true).unwrap();
        assert_eq!(result.files.len(), 1);
        assert_eq!(result.skipped_symlinks, 1);
        assert!(scanner::scan(&fixture.path().join("shortcut"), true).is_err());
    }
}

#[test]
fn handles_empty_directories() {
    let fixture = Fixture::new();
    let summary = scanner::scan(fixture.path(), false).unwrap();
    assert!(summary.files.is_empty());
    assert_eq!(summary.total_bytes(), 0);
    assert!(summary.extension_counts().is_empty());
    assert!(summary.find("anything").is_empty());
    let report = report::render(&summary, 400);
    assert!(report.contains("该目录没有可展示的普通文件"));
}

#[cfg(unix)]
#[test]
fn accepts_non_utf8_file_names_without_panicking() {
    use std::os::unix::ffi::OsStringExt;
    let fixture = Fixture::new();
    let name = std::ffi::OsString::from_vec(b"strange-\xff.rs".to_vec());
    fs::write(fixture.path().join(name), "x").unwrap();
    let summary = scanner::scan(fixture.path(), false).unwrap();
    assert_eq!(summary.files.len(), 1);
    assert_eq!(summary.extension_counts(), vec![("rs".to_string(), 1)]);
    let html = report::render(&summary, 10);
    assert!(html.contains("strange-"));
}

#[test]
fn errors_on_invalid_scan_roots() {
    let fixture = Fixture::new();
    fixture.write("one.txt", "data");
    assert!(scanner::scan(&fixture.path().join("one.txt"), false).is_err());
    assert!(scanner::scan(&fixture.path().join("missing"), false).is_err());
}

#[test]
fn report_escapes_file_names_and_refuses_unintended_overwrites() {
    let fixture = Fixture::new();
    // Windows 文件名不允许 < > 等符号；分别验证实际文件路径转义和纯函数转义。
    #[cfg(windows)]
    fixture.write("name&A'postrophe.rs", "hello");
    #[cfg(not(windows))]
    fixture.write("<img onerror='x'>.rs", "hello");
    let summary = scanner::scan(fixture.path(), false).unwrap();
    let html = report::render(&summary, 100);
    #[cfg(windows)]
    assert!(html.contains("name&amp;A&#39;postrophe.rs"));
    #[cfg(not(windows))]
    {
        assert!(html.contains("&lt;img onerror=&#39;x&#39;&gt;.rs"));
        assert!(!html.contains("<img onerror='x'>"));
    }
    assert!(html.contains("本地文件雷达"));
    let path = fixture.path().join("report.html");
    report::write_report(&path, &html, false).unwrap();
    assert!(report::write_report(&path, "other", false).is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), html);
    report::write_report(&path, "new content", true).unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), "new content");
}

#[test]
fn report_rejects_overwriting_symlink_even_with_force() {
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let fixture = Fixture::new();
        fixture.write("real.txt", "untouched");
        let target = fixture.path().join("real.txt");
        let shortcut = fixture.path().join("alias.txt");
        symlink(&target, &shortcut).unwrap();
        assert!(report::write_report(&shortcut, "evil", true).is_err());
        assert_eq!(fs::read_to_string(target).unwrap(), "untouched");
    }
}
