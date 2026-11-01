//! PageMint 的真实 Cargo + CLI + HTML 文件验收回归：完成版、空项目和作弊行为。
use labquest::transfer;
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
        let index = NONCE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "rust-applab-pagemint-e2e-{}-{nanos}-{index}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        Self(root)
    }
    fn project(&self) -> PathBuf {
        self.0.join("pagemint-student")
    }
}
impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn starter_is_independent_and_cannot_pass_without_implementation() {
    let space = Sandbox::new();
    let root = space.project();
    transfer::init(&root).unwrap();
    assert!(root.join("samples/notes/01-first.note").is_file());
    assert!(root.join("Cargo.toml").is_file());
    assert!(fs::read_to_string(root.join("src/main.rs"))
        .unwrap()
        .contains("TODO(04)"));
    assert!(transfer::init(&root).is_err());
    assert!(!transfer::check(&root).unwrap());
    let report = fs::read_to_string(root.join("APP_ASSESSMENT.md")).unwrap();
    assert!(report.contains("FAIL"));
}

#[test]
fn correct_independent_solution_passes_real_html_and_robustness_checks() {
    let space = Sandbox::new();
    let root = space.project();
    transfer::init(&root).unwrap();
    fs::write(
        root.join("src/main.rs"),
        include_str!("fixtures/pagemint_reference.rs"),
    )
    .unwrap();
    assert!(transfer::check(&root).unwrap());
    assert!(transfer::check(&root).unwrap());
    let report = fs::read_to_string(root.join("APP_ASSESSMENT.md")).unwrap();
    #[cfg(unix)]
    assert!(report.contains("通过；") || report.contains("9/9 项通过"));
    #[cfg(not(unix))]
    assert!(report.contains("8/8 项通过"));
    assert!(!report.contains("## FAIL"));
}

#[test]
fn hardcoded_success_output_does_not_pass() {
    let space = Sandbox::new();
    let root = space.project();
    transfer::init(&root).unwrap();
    fs::write(
        root.join("src/main.rs"),
        "fn main() { println!(\"built=1\"); }",
    )
    .unwrap();
    assert!(!transfer::check(&root).unwrap());
    assert!(fs::read_to_string(root.join("APP_ASSESSMENT.md"))
        .unwrap()
        .contains("FAIL"));
}

#[test]
fn html_without_escaping_is_rejected() {
    let space = Sandbox::new();
    let root = space.project();
    transfer::init(&root).unwrap();
    let source = include_str!("fixtures/pagemint_reference.rs").replace(
        "'&' => result.push_str(\"&amp;\"),",
        "'&' => result.push('&'),",
    );
    fs::write(root.join("src/main.rs"), source).unwrap();
    assert!(!transfer::check(&root).unwrap());
    let report = fs::read_to_string(root.join("APP_ASSESSMENT.md")).unwrap();
    assert!(report.contains("Unicode 与 HTML 转义"));
    assert!(report.contains("## FAIL"));
}

#[test]
fn assessment_will_not_overwrite_personal_report() {
    let space = Sandbox::new();
    let root = space.project();
    transfer::init(&root).unwrap();
    fs::write(root.join("APP_ASSESSMENT.md"), "我的个人开发笔记").unwrap();
    assert!(transfer::check(&root).is_err());
    assert_eq!(
        fs::read_to_string(root.join("APP_ASSESSMENT.md")).unwrap(),
        "我的个人开发笔记"
    );
}
