//! TaskPocket 的负向/正向真实编译验收；使用独立临时项目，不触碰学员目录。
use labquest::capstone;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NONCE: AtomicU64 = AtomicU64::new(0);
struct Sandbox(PathBuf);
impl Sandbox {
    fn new() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let suffix = NONCE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "rust-applab-capstone-{}-{nanos}-{suffix}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        Self(root)
    }
    fn student(&self) -> PathBuf {
        self.0.join("student")
    }
}
impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn init_creates_independent_unsolved_project_without_overwriting() {
    let space = Sandbox::new();
    let project = space.student();
    capstone::init(&project).unwrap();
    assert!(project.join("Cargo.toml").is_file());
    assert!(project.join("src/main.rs").is_file());
    assert!(project.join("CONTRACT.md").is_file());
    assert!(fs::read_to_string(project.join("src/main.rs"))
        .unwrap()
        .contains("TODO(01)"));
    assert!(capstone::init(&project).is_err());
}

#[test]
fn starter_compiles_but_has_no_fake_success_and_reports_failures() {
    let space = Sandbox::new();
    let project = space.student();
    capstone::init(&project).unwrap();
    let result = capstone::check(&project).unwrap();
    assert!(!result, "未实现程序不能被评为合格");
    let report = fs::read_to_string(project.join("APP_ASSESSMENT.md")).unwrap();
    assert!(report.contains("FAIL"));
    assert!(report.contains("跨进程") || report.contains("缺失数据库"));
}

#[test]
fn real_independent_implementation_passes_nine_groups_and_can_be_rechecked() {
    let space = Sandbox::new();
    let project = space.student();
    capstone::init(&project).unwrap();
    fs::write(
        project.join("src/main.rs"),
        include_str!("fixtures/taskpocket_reference.rs"),
    )
    .unwrap();
    assert!(capstone::check(&project).unwrap());
    assert!(capstone::check(&project).unwrap());
    let report = fs::read_to_string(project.join("APP_ASSESSMENT.md")).unwrap();
    assert!(report.contains("通过 9/9 项"));
    assert!(!report.contains("## FAIL"));
}

#[test]
fn hardcoding_first_id_cannot_pass_dynamic_and_persistence() {
    let space = Sandbox::new();
    let project = space.student();
    capstone::init(&project).unwrap();
    fs::write(
        project.join("src/main.rs"),
        r#"fn main() { println!("added=1"); }"#,
    )
    .unwrap();
    assert!(!capstone::check(&project).unwrap());
    let report = fs::read_to_string(project.join("APP_ASSESSMENT.md")).unwrap();
    assert!(report.contains("TSV") || report.contains("缺失数据库"));
    assert!(!report.contains("通过 9/9 项"));
}

#[test]
fn report_wont_overwrite_human_written_content() {
    let space = Sandbox::new();
    let project = space.student();
    capstone::init(&project).unwrap();
    fs::write(project.join("APP_ASSESSMENT.md"), "this is my own journal").unwrap();
    assert!(capstone::check(&project).is_err());
    assert_eq!(
        fs::read_to_string(project.join("APP_ASSESSMENT.md")).unwrap(),
        "this is my own journal"
    );
}
