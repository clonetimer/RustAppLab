//! B4 needs the explicitly fetched, locked Serde graph. Base tests stay registry-free.
use labquest::advanced_b;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
const CORE: &str = include_str!("../../../projects/advanced-b/reference/taskcore/src/lib.rs");
const JSON: &str = include_str!("../../../projects/advanced-b/reference/json-export/src/lib.rs");
const CLI: &str = include_str!("../../../projects/advanced-b/reference/json-export/src/main.rs");
static NONCE: AtomicU64 = AtomicU64::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "rustapplab-b4-{}-{time}-{}",
            std::process::id(),
            NONCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn project(&self, complete: bool) -> PathBuf {
        let p = self.0.join("my JSON 学习");
        advanced_b::init(&p).unwrap();
        fs::write(p.join("taskcore/src/lib.rs"), CORE).unwrap();
        if complete {
            fs::write(p.join("json-export/src/lib.rs"), JSON).unwrap();
            fs::write(p.join("json-export/src/main.rs"), CLI).unwrap();
        }
        p
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn command(action: &str, project: &Path) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_labquest"));
    c.args(["advanced-b", action]).arg(project);
    c
}
fn assert_success(out: &Output) {
    assert!(
        out.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}
fn report(project: &Path) -> String {
    let mut paths: Vec<_> = fs::read_dir(project.join(".advanced-b-reports"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    paths.sort();
    fs::read_to_string(paths.last().unwrap()).unwrap()
}

#[test]
fn todo_red_then_fixed_reference_green_and_source_scoped_navigation() {
    let tmp = Scratch::new();
    let project = tmp.project(false);
    assert!(!advanced_b::check(&project, 4).unwrap());
    assert!(report(&project).contains("passed=3\n"));
    fs::write(project.join("json-export/src/lib.rs"), JSON).unwrap();
    fs::write(project.join("json-export/src/main.rs"), CLI).unwrap();
    // These files must never control the platform's fixed assessment.
    fs::write(
        project.join("json-export/Cargo.toml"),
        "deliberately invalid manifest",
    )
    .unwrap();
    fs::write(project.join("json-export/Cargo.lock"), "not a lockfile").unwrap();
    for name in ["b4_api.rs", "b4_cli.rs"] {
        fs::write(
            project.join("json-export/tests").join(name),
            "compile_error!(\"untrusted test executed\");",
        )
        .unwrap();
    }
    fs::write(
        project.join("json-export/build.rs"),
        "fn main(){panic!(\"untrusted build executed\");}",
    )
    .unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_labquest"))
        .args(["learn", "check"])
        .arg(&project)
        .arg("all")
        .output()
        .unwrap();
    assert_success(&out);
    let complete = report(&project);
    assert!(complete.starts_with("RustAppLab Advanced B Report v3\n"));
    assert!(complete.contains("passed=4\n") && complete.contains("json_source_sha256="));
    assert!(advanced_b::next(&project).unwrap().contains("4/4"));
    fs::write(
        project.join("json-export/src/lib.rs"),
        format!("{JSON}\n// personal change\n"),
    )
    .unwrap();
    let next = advanced_b::next(&project).unwrap();
    assert!(next.contains("B4") && next.contains("过期"), "{next}");
    fs::write(project.join("json-export/src/lib.rs"), JSON).unwrap();
    // The latest explicit B3 report keeps B4 pending, even after a previous B4 pass.
    assert!(advanced_b::check(&project, 3).unwrap());
    assert!(!advanced_b::next(&project).unwrap().contains("4/4"));
    assert!(advanced_b::next(&project).unwrap().contains("B4"));
    fs::write(project.join("json-export/src/lib.rs"), "invalid Rust").unwrap();
    assert!(!advanced_b::next(&project).unwrap().contains("过期"));
}

#[test]
fn b4_rejects_hardcoded_json_wrong_field_types_and_broken_cli() {
    let wrong_type = JSON
        .replace("done: bool,", "done: u8,")
        .replace("done: task.done,", "done: u8::from(task.done),");
    assert_ne!(wrong_type, JSON);
    for (lib, cli) in [
        (
            "pub fn to_json(_: &[taskcore::Task]) -> Result<String,String> { Ok(\"[]\".into()) }",
            CLI,
        ),
        (wrong_type.as_str(), CLI),
        (JSON, "fn main() { println!(\"[]\"); }"),
    ] {
        let tmp = Scratch::new();
        let project = tmp.project(true);
        fs::write(project.join("json-export/src/lib.rs"), lib).unwrap();
        fs::write(project.join("json-export/src/main.rs"), cli).unwrap();
        assert!(!advanced_b::check(&project, 4).unwrap());
        let text = report(&project);
        assert!(
            text.contains("passed=3\n") && text.contains("B4-default"),
            "{text}"
        );
    }
}

#[test]
fn b4_requires_strict_and_doctests_after_default_tests_pass() {
    let strict_only = JSON.replace(
        "    let mut ids = HashSet::new();",
        "    if cfg!(feature = \"strict-titles\") { return Err(\"strict-only regression\".into()); }\n    let mut ids = HashSet::new();",
    );
    assert_ne!(strict_only, JSON);
    let doc_only = format!("{JSON}\n/// ```\n/// panic!(\"doctest-only regression\");\n/// ```\npub struct BrokenExample;\n");
    for (source, failure) in [(strict_only, "B4-strict"), (doc_only, "B4-doc-default")] {
        let tmp = Scratch::new();
        let project = tmp.project(true);
        fs::write(project.join("json-export/src/lib.rs"), source).unwrap();
        assert!(!advanced_b::check(&project, 4).unwrap());
        let text = report(&project);
        assert!(text.contains("passed=3\n"), "{text}");
        assert!(text.contains(&format!("=== {failure} ===")), "{text}");
        assert!(text.contains("=== B4-default ==="), "{text}");
        assert!(!text.contains("=== B4-doc-strict ==="), "{text}");
    }
}

#[test]
fn b4_regresses_earlier_stages_before_reading_json_source() {
    let tmp = Scratch::new();
    let project = tmp.project(true);
    fs::write(
        project.join("taskcore/src/lib.rs"),
        CORE.replace("total: tasks.len(),", "total: 0,"),
    )
    .unwrap();
    fs::remove_dir_all(project.join("json-export")).unwrap();
    assert!(!advanced_b::check(&project, 4).unwrap());
    let text = report(&project);
    assert!(text.contains("passed=0\n"), "{text}");
    assert!(!text.contains("=== B4"), "{text}");
}

#[test]
fn empty_cache_is_an_environment_failure_and_prepare_failure_is_persisted() {
    let tmp = Scratch::new();
    let project = tmp.project(true);
    let cache = tmp.0.join("empty-cargo-cache");
    fs::create_dir(&cache).unwrap();
    let out = command("check", &project)
        .arg("4")
        .env("CARGO_HOME", &cache)
        .env("CARGO_NET_OFFLINE", "true")
        .output()
        .unwrap();
    assert!(!out.status.success());
    let text = report(&project);
    assert!(
        text.contains("passed=3\n") && text.contains("prepare-json"),
        "{text}"
    );
    assert!(!text.contains("=== B4-default ==="), "{text}");
    let out = command("prepare-json", &project)
        .env("CARGO_HOME", &cache)
        .env("CARGO_NET_OFFLINE", "true")
        .output()
        .unwrap();
    assert!(!out.status.success());
    let status = fs::read_to_string(project.join(".advanced-b-dependencies/STATUS.md")).unwrap();
    assert!(
        status.contains("fetch: FAIL") && status.contains("状态：失败"),
        "{status}"
    );
    assert!(!status.contains("状态：PASS"));
    assert!(project
        .join(".advanced-b-dependencies/logs/fetch.txt")
        .is_file());
}

#[test]
fn preparation_executes_fixed_reference_only_and_is_not_a_grade() {
    let tmp = Scratch::new();
    let project = tmp.project(false);
    let core = "compile_error!(\"learner code must not run during prepare\");";
    fs::write(project.join("taskcore/src/lib.rs"), core).unwrap();
    fs::write(
        project.join("taskcore/Cargo.toml"),
        "invalid learner manifest",
    )
    .unwrap();
    fs::write(
        project.join("json-export/build.rs"),
        "fn main(){panic!(\"learner build script executed\");}",
    )
    .unwrap();
    fs::write(
        project.join("json-export/Cargo.toml"),
        "invalid learner manifest",
    )
    .unwrap();
    fs::create_dir(project.join(".cargo")).unwrap();
    fs::write(
        project.join(".cargo/config.toml"),
        "invalid learner Cargo configuration",
    )
    .unwrap();
    let before = fs::read(project.join("json-export/src/lib.rs")).unwrap();
    let out = command("prepare-json", &project)
        .env("CARGO_NET_OFFLINE", "true")
        .output()
        .unwrap();
    assert_success(&out);
    let status = fs::read_to_string(project.join(".advanced-b-dependencies/STATUS.md")).unwrap();
    assert!(
        status.contains("状态：PASS") && status.contains("不是 B4 通过记录"),
        "{status}"
    );
    assert!(!project.join(".advanced-b-reports").exists());
    assert_eq!(
        fs::read_to_string(project.join("taskcore/src/lib.rs")).unwrap(),
        core
    );
    assert_eq!(
        fs::read(project.join("json-export/src/lib.rs")).unwrap(),
        before
    );
    assert!(advanced_b::next(&project).unwrap().contains("从 B1 开始"));
    // Owned state is necessary before a retry may overwrite preparation logs.
    fs::write(
        project.join(".advanced-b-dependencies/.rustapplab-json-dependencies"),
        "foreign directory",
    )
    .unwrap();
    assert!(!command("prepare-json", &project)
        .env("CARGO_NET_OFFLINE", "true")
        .output()
        .unwrap()
        .status
        .success());
    assert_eq!(
        fs::read_to_string(project.join(".advanced-b-dependencies/STATUS.md")).unwrap(),
        status
    );
}

#[test]
fn missing_json_extension_preserves_b3_and_explains_explicit_migration() {
    let tmp = Scratch::new();
    let project = tmp.project(true);
    fs::remove_dir_all(project.join("json-export")).unwrap();
    let out = command("check", &project).arg("all").output().unwrap();
    assert!(!out.status.success());
    let text = report(&project);
    assert!(
        text.contains("passed=3\n") && text.contains("add-json"),
        "{text}"
    );
    assert!(!project.join("json-export").exists());
    assert!(advanced_b::next(&project).unwrap().contains("B1..B3"));
}
