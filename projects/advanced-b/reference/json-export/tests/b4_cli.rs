use serde_json::{json, Value};
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "taskpocket-json-{}-{unique}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn path(&self) -> PathBuf {
        self.0.join("中文 tasks.tsv")
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn run(args: &[&OsStr]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_taskpocket-json"))
        .args(args)
        .output()
        .unwrap()
}
fn export(path: &Path) -> Output {
    run(&[OsStr::new("export-json"), path.as_os_str()])
}
fn success(output: &Output) -> Value {
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    // Accept the host's line ending, but require exactly one terminating newline.
    let line = stdout
        .strip_suffix("\r\n")
        .or_else(|| stdout.strip_suffix('\n'))
        .expect("one trailing newline");
    assert!(!line.ends_with(['\r', '\n']));
    assert!(!line.contains(['\r', '\n']));
    serde_json::from_str(line).unwrap()
}
fn failure(output: &Output) {
    assert!(!output.status.success());
    assert!(output.stdout.is_empty(), "stdout: {:?}", output.stdout);
    assert!(!output.stderr.is_empty());
}

#[test]
fn real_binary_exports_sorted_typed_json_without_changing_disk() {
    let f = Fixture::new();
    let path = f.path();
    let data = b"9\tOPEN\tsecond\n2\tDONE\tfirst\n";
    fs::write(&path, data).unwrap();
    assert_eq!(
        success(&export(&path)),
        json!([
            {"id":2,"done":true,"title":"first"}, {"id":9,"done":false,"title":"second"}
        ])
    );
    assert_eq!(fs::read(&path).unwrap(), data);
    assert_eq!(fs::read_dir(&f.0).unwrap().count(), 1);
}

#[test]
fn cli_preserves_the_existing_tsv_title_normalization() {
    let f = Fixture::new();
    let path = f.path();
    let data = "1\tOPEN\t  中文  \r\n";
    fs::write(&path, data).unwrap();
    assert_eq!(
        success(&export(&path)),
        json!([{"id":1,"done":false,"title":"中文"}])
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), data);
}

#[test]
fn cli_escapes_legal_controls_quotes_and_backslashes() {
    let f = Fixture::new();
    let path = f.path();
    let data = "18446744073709551615\tDONE\t中\0间\u{0008}字\"\\\n";
    fs::write(&path, data).unwrap();
    assert_eq!(
        success(&export(&path)),
        json!([{"id":u64::MAX,"done":true,"title":"中\0间\u{0008}字\"\\"}])
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), data);
}

#[test]
fn missing_database_exports_empty_without_creating_any_file() {
    let f = Fixture::new();
    let path = f.path();
    assert_eq!(success(&export(&path)), json!([]));
    assert!(!path.exists());
    assert_eq!(fs::read_dir(&f.0).unwrap().count(), 0);
}

#[test]
fn empty_existing_database_stays_empty() {
    let f = Fixture::new();
    let path = f.path();
    fs::write(&path, "").unwrap();
    assert_eq!(success(&export(&path)), json!([]));
    assert!(fs::read(&path).unwrap().is_empty());
}

#[test]
fn malformed_database_fails_without_writes() {
    let f = Fixture::new();
    let path = f.path();
    for data in [
        "0\tOPEN\tzero\n",
        "1\tOPEN\tx\n1\tDONE\ty\n",
        "1\tOTHER\tx\n",
        "1\tOPEN\tx\textra\n",
        "1\tOPEN\tbad\rtitle\n",
        "1\tOPEN\t\n",
        "not tsv",
    ] {
        fs::write(&path, data).unwrap();
        failure(&export(&path));
        assert_eq!(fs::read_to_string(&path).unwrap(), data);
    }
}

#[test]
fn directory_read_error_has_no_stdout_and_preserves_directory() {
    let f = Fixture::new();
    failure(&export(&f.0));
    assert!(f.0.is_dir());
    assert_eq!(fs::read_dir(&f.0).unwrap().count(), 0);
}

#[test]
fn missing_unknown_and_extra_arguments_fail_without_side_effects() {
    let f = Fixture::new();
    let path = f.path();
    for args in [
        vec![],
        vec![OsStr::new("export-json")],
        vec![OsStr::new("export"), path.as_os_str()],
        vec![
            OsStr::new("export-json"),
            path.as_os_str(),
            OsStr::new("extra"),
        ],
    ] {
        failure(&run(&args));
    }
    assert_eq!(fs::read_dir(&f.0).unwrap().count(), 0);
}

#[test]
fn cli_feature_enforces_unicode_character_boundary() {
    let f = Fixture::new();
    let path = f.path();
    let eighty = "学".repeat(80);
    fs::write(&path, format!("1\tOPEN\t{eighty}\n")).unwrap();
    assert_eq!(success(&export(&path))[0]["title"], eighty);
    let data = format!("1\tOPEN\t{}\n", "学".repeat(81));
    fs::write(&path, &data).unwrap();
    let output = export(&path);
    #[cfg(feature = "strict-titles")]
    failure(&output);
    #[cfg(not(feature = "strict-titles"))]
    assert_eq!(success(&output)[0]["title"], "学".repeat(81));
    assert_eq!(fs::read_to_string(&path).unwrap(), data);
}
