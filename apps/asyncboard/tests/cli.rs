use asyncboard::{parse_jobs, render_report, Completion};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};
static N: AtomicUsize = AtomicUsize::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let d = std::env::temp_dir().join(format!(
            "applab-asyncboard-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&d).unwrap();
        Self(d)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn demo_and_re_run() {
    let dir = Temp::new();
    let folder = dir.0.join("中文 路径");
    let out = Command::new(env!("CARGO_BIN_EXE_asyncboard"))
        .arg("demo")
        .arg(&folder)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let original = fs::read_to_string(folder.join("RESULTS.tsv")).unwrap();
    assert_eq!(original.lines().count(), 4);
    let ids: Vec<_> = original
        .lines()
        .skip(1)
        .map(|l| l.split('\t').next().unwrap())
        .collect();
    assert_eq!(ids, ["write", "review", "read"]);
    let out2 = Command::new(env!("CARGO_BIN_EXE_asyncboard"))
        .arg("demo")
        .arg(&folder)
        .output()
        .unwrap();
    assert!(!out2.status.success());
    assert_eq!(
        fs::read_to_string(folder.join("RESULTS.tsv")).unwrap(),
        original
    );
    let second = folder.join("SECOND.tsv");
    let out3 = Command::new(env!("CARGO_BIN_EXE_asyncboard"))
        .arg("run")
        .arg(folder.join("jobs.tsv"))
        .arg(&second)
        .output()
        .unwrap();
    assert!(
        out3.status.success(),
        "{}",
        String::from_utf8_lossy(&out3.stderr)
    );
    assert_eq!(fs::read_to_string(second).unwrap(), original);
}
#[test]
fn invalid_manifest_does_not_create_output() {
    let dir = Temp::new();
    let input = dir.0.join("bad.tsv");
    let output = dir.0.join("report.tsv");
    fs::write(&input, "bad\t-10\tinvalid\n").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_asyncboard"))
        .arg("run")
        .arg(&input)
        .arg(&output)
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(!output.exists());
}
#[test]
fn report_unicode_and_boundary() {
    let jobs = parse_jobs("id1\t0\t你好 Rust\n").unwrap();
    assert_eq!(jobs[0].delay_ms, 0);
    assert_eq!(
        render_report(&[Completion {
            id: "id1".into(),
            delay_ms: 0,
            note: "你好 Rust".into()
        }]),
        "id\twait_ms\tnote\nid1\t0\t你好 Rust\n"
    );
}
