//! Real independent Cargo builds; fixtures validate the teaching grader, not learner outcomes.
use labquest::{capstone, independent, learning, transfer};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
static COUNT: AtomicU64 = AtomicU64::new(0);
struct Space(PathBuf);
impl Space {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "independent-dev6-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            COUNT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Space {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
const TASK: &str = include_str!("fixtures/taskpocket_reference.rs");
const PAGE: &str = include_str!("fixtures/pagemint_reference.rs");
fn start(s: &Space, id: &str) -> PathBuf {
    let p = s.0.join(format!("我的 {id}"));
    if id == "taskpocket" {
        capstone::init(&p).unwrap();
    } else {
        transfer::init(&p).unwrap();
    }
    p
}
#[test]
fn taskpocket_readonly_then_lifecycle_then_full_contract() {
    let s = Space::new();
    let p = start(&s, "taskpocket");
    assert!(!independent::check(&p, "taskpocket", 1).unwrap());
    let partial=TASK.replace("let mut tasks = load(path)?;", "let mut tasks = load(path)?; if cmd == \"add\" || cmd == \"done\" { return Err(\"not implemented yet\".into()); }");
    fs::write(p.join("src/main.rs"), partial).unwrap();
    assert!(learning::check_current(&p, Some(1)).unwrap());
    assert!(learning::next(&p).unwrap().contains("下一检查点 2"));
    assert!(!learning::check_current(&p, Some(2)).unwrap());
    assert!(!p.join("APP_ASSESSMENT.md").exists());
    fs::write(p.join("src/main.rs"), TASK).unwrap();
    assert!(independent::check(&p, "taskpocket", 2).unwrap());
    assert!(learning::next(&p).unwrap().contains("下一检查点 3"));
    assert!(capstone::check(&p).unwrap());
    assert!(fs::read_to_string(p.join("APP_ASSESSMENT.md"))
        .unwrap()
        .contains("通过 9/9 项"));
    let manifest = p.join("Cargo.toml");
    fs::write(
        &manifest,
        fs::read_to_string(&manifest).unwrap() + "\n# my edit\n",
    )
    .unwrap();
    assert!(learning::next(&p).unwrap().contains("过期"));
}
#[test]
fn pagemint_stats_does_not_require_build_then_full_works() {
    let s = Space::new();
    let p = start(&s, "pagemint");
    assert!(!independent::check(&p, "pagemint", 1).unwrap());
    let partial = PAGE.replace("if cmd == \"build\" =>", "if cmd == \"build-later\" =>");
    assert_ne!(partial, PAGE);
    fs::write(p.join("src/main.rs"), partial).unwrap();
    assert!(learning::check_current(&p, Some(1)).unwrap());
    assert!(!learning::check_current(&p, Some(2)).unwrap());
    assert!(!p.join("APP_ASSESSMENT.md").exists());
    fs::write(p.join("src/main.rs"), PAGE).unwrap();
    assert!(independent::check(&p, "pagemint", 2).unwrap());
    assert!(transfer::check(&p).unwrap());
    assert!(learning::overview(&s.0)
        .unwrap()
        .contains("最终合同已有通过记录"));
    let t = p.join("tests/self_check.rs");
    fs::write(
        &t,
        fs::read_to_string(&t).unwrap() + "\n// own test change\n",
    )
    .unwrap();
    assert!(learning::next(&p).unwrap().contains("过期"));
}
#[test]
fn stage_two_success_does_not_hide_data_protection_bugs() {
    let s = Space::new();
    let t = start(&s, "taskpocket");
    let buggy = TASK.replace("if title.is_empty()", "if false");
    assert_ne!(buggy, TASK);
    fs::write(t.join("src/main.rs"), buggy).unwrap();
    assert!(independent::check(&t, "taskpocket", 2).unwrap());
    assert!(!independent::check(&t, "taskpocket", 3).unwrap());
    assert!(learning::next(&t).unwrap().contains("检查点 3 未通过"));
    let p = start(&s, "pagemint");
    let buggy = PAGE.replace(".create_new(true)", ".create(true).truncate(true)");
    assert_ne!(buggy, PAGE);
    fs::write(p.join("src/main.rs"), buggy).unwrap();
    assert!(independent::check(&p, "pagemint", 2).unwrap());
    assert!(!independent::check(&p, "pagemint", 3).unwrap());
}
#[test]
fn cli_guidance_is_readonly_and_rejects_wrong_type_and_scope() {
    let s = Space::new();
    let p = start(&s, "taskpocket");
    let orig = fs::read(p.join("src/main.rs")).unwrap();
    for command in ["next", "lesson"] {
        let o = Command::new(env!("CARGO_BIN_EXE_labquest"))
            .args(["learn", command])
            .arg(&p)
            .output()
            .unwrap();
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        assert!(String::from_utf8_lossy(&o.stdout).contains("只读"));
    }
    let o = Command::new(env!("CARGO_BIN_EXE_labquest"))
        .args(["learn", "hint"])
        .arg(&p)
        .args(["1", "2"])
        .output()
        .unwrap();
    assert!(o.status.success());
    assert_eq!(orig, fs::read(p.join("src/main.rs")).unwrap());
    assert!(!p.join(".app-reports").exists());
    let o = Command::new(env!("CARGO_BIN_EXE_labquest"))
        .args(["app", "checkpoint", "taskpocket"])
        .arg(&p)
        .arg("0")
        .output()
        .unwrap();
    assert!(!o.status.success());
    assert!(independent::check(&p, "pagemint", 1).is_err());
    assert!(!p.join(".applab-target").exists());
}
#[test]
fn generated_self_tests_expose_unfinished_work_instead_of_fake_completion() {
    let s = Space::new();
    for id in ["taskpocket", "pagemint"] {
        let p = start(&s, id);
        let base = |ignored: bool| {
            let mut c = Command::new("cargo");
            c.args(["test", "--offline", "--manifest-path"])
                .arg(p.join("Cargo.toml"))
                .arg("--target-dir")
                .arg(p.join(".self-target"))
                .args(["--test", "self_check"]);
            if ignored {
                c.args(["my_regression", "--", "--ignored"]);
            }
            c.env_remove("CARGO_BUILD_TARGET");
            c.output().unwrap()
        };
        let a = base(false);
        assert!(a.status.success(), "{}", String::from_utf8_lossy(&a.stderr));
        let b = base(true);
        assert!(!b.status.success());
        assert!(String::from_utf8_lossy(&b.stdout).contains("定义输入"));
    }
}
#[test]
fn old_projects_without_new_guides_still_get_guidance_without_overwriting() {
    let s = Space::new();
    let p = start(&s, "pagemint");
    for f in [
        "START.html",
        "DEVELOPMENT_PLAN.md",
        "TEST_PLAN.md",
        "RELEASE_CHECKLIST.md",
    ] {
        fs::remove_file(p.join(f)).unwrap();
    }
    fs::remove_dir_all(p.join("tests")).unwrap();
    assert!(learning::lesson(&p, None).unwrap().contains("旧工程"));
    assert!(!p.join("START.html").exists());
    fs::write(p.join("APP_ASSESSMENT.md"), "my own report").unwrap();
    assert!(transfer::check(&p).is_err());
    assert!(!p.join(".applab-target").exists());
    assert_eq!(
        fs::read_to_string(p.join("APP_ASSESSMENT.md")).unwrap(),
        "my own report"
    );
}
