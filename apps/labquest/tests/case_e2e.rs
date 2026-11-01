//! 课程闭环验收：真实生成案例、构建执行微应用、复用三款产品、提交最终作品。
//! 测试使用全新临时目录，不需要网络或第三方 crate。
use labquest::{board, case, checks, coach, proof, simulation};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NONCE: AtomicU64 = AtomicU64::new(0);
struct Sandbox(PathBuf);
impl Sandbox {
    fn new() -> Self {
        let nonce = NONCE.fetch_add(1, Ordering::Relaxed);
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "rust-applab-labquest-{}-{time}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn case(&self) -> PathBuf {
        self.0.join("case")
    }
}
impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn report_file(root: &Path) {
    let summary = filequest::scanner::scan(&root.join("project"), false).unwrap();
    assert!(summary.problems.is_empty());
    fs::write(
        root.join("deliverables/inventory.html"),
        filequest::report::render(&summary, 400),
    )
    .unwrap();
}
fn report_log(root: &Path, src: &str, dst: &str) {
    let analyzed = loglens::analyzer::analyze_file(&root.join("logs").join(src), true).unwrap();
    fs::write(
        root.join("deliverables").join(dst),
        loglens::report::render(&analyzed, 10),
    )
    .unwrap();
}
fn patched_source(root: &Path) {
    let source = root.join("project/src/main.rs");
    let original = fs::read_to_string(&source).unwrap();
    assert!(original.contains("// TODO(CASE-01)"));
    let revised = original.replace("    // TODO(CASE-01):", "    let errors = LATENCIES.iter().filter(|&&ms| ms > timeout).count();\n    println!(\"SUMMARY requests=12 errors={errors}\");\n    // TODO(CASE-01):");
    fs::write(source, revised).unwrap();
}

#[test]
fn new_case_has_real_inputs_but_no_free_progress() {
    let temp = Sandbox::new();
    let root = temp.case();
    case::start(&root).unwrap();
    assert!(
        case::start(&root).is_err(),
        "start must never replace a case"
    );
    assert_eq!(
        fs::read_to_string(root.join("logs/incident.log")).unwrap(),
        simulation::render_log(150)
    );
    assert_eq!(
        fs::read_to_string(root.join("project/config/service.conf"))
            .unwrap()
            .lines()
            .last(),
        Some("timeout_ms=150")
    );
    let grades = checks::evaluate(&root).unwrap();
    assert_eq!(grades.len(), 6);
    assert!(!checks::all_complete(&grades));
    assert_eq!(case::version(&root).unwrap(), case::CaseVersion::Transfer);
    assert!(coach::next(&root, &grades).contains("文件现场地图"));
    assert!(labquest::finish(&root).is_err());
    assert!(!root.join("deliverables/CASE_REPORT.md").exists());
    let dashboard = board::write(&root, &grades).unwrap();
    let html = fs::read_to_string(dashboard).unwrap();
    assert!(html.contains("0/6"));
    assert!(html.contains("离线静态快照"));
}

#[test]
fn full_case_requires_actual_code_build_and_evidence_stays_fresh() {
    let temp = Sandbox::new();
    let root = temp.case();
    case::start(&root).unwrap();
    report_log(&root, "incident.log", "incident.md");
    fs::write(root.join("project/config/service.conf"), "timeout_ms=800\n").unwrap();
    assert_eq!(case::simulate(&root).unwrap(), (800, 0));
    report_log(&root, "current.log", "current.md");
    report_file(&root);
    assert!(
        proof::test_app(&root).is_err(),
        "unmodified app must not pass the code exercise"
    );
    assert!(!root.join("deliverables/PROGRAM_PROOF.txt").exists());

    patched_source(&root);
    proof::test_app(&root).unwrap(); // actually executes `cargo run --offline --locked` on the mini project
    assert!(proof::check(&root));
    assert!(root.join("build").exists());
    assert!(
        !root.join("project/target").exists(),
        "compilation must not contaminate the deliverable"
    );
    assert_eq!(
        fs::read_to_string(root.join("logs/program.log"))
            .unwrap()
            .lines()
            .count(),
        13
    );
    assert!(
        !checks::evaluate(&root).unwrap()[0].passed,
        "source edits invalidate old inventory"
    );
    report_file(&root);
    safesync::ops::backup(
        &root.join("project"),
        &root.join("backups/project-snapshot"),
    )
    .unwrap();
    assert!(checks::all_complete(&checks::evaluate(&root).unwrap()));
    let report = labquest::finish(&root).unwrap();
    let text = fs::read_to_string(&report).unwrap();
    assert!(text.contains("91.67%"));
    assert!(text.contains("PROGRAM_PROOF.txt"));
    assert!(
        labquest::finish(&root).is_err(),
        "must not overwrite an existing final report"
    );

    fs::write(root.join("project/config/service.conf"), "timeout_ms=900\n").unwrap();
    assert!(
        !proof::check(&root),
        "old proof must not validate new config"
    );
    let grades = checks::evaluate(&root).unwrap();
    assert!(!grades[4].passed && !grades[5].passed);
    fs::write(root.join("logs/current.log"), "bogus log\n").unwrap();
    assert!(
        !checks::evaluate(&root).unwrap()[2].passed,
        "manually forged current log fails"
    );
}

#[test]
fn case_rejects_unsafe_directories_and_unknown_markers() {
    let temp = Sandbox::new();
    let root = temp.case();
    case::start(&root).unwrap();
    fs::write(root.join(".labquest-case"), "not-a-real-case\n").unwrap();
    assert!(case::ensure_case(&root).is_err());
    assert!(case::simulate(&root).is_err());
    fs::write(root.join(".labquest-case"), "RustAppLab-LabQuest-Case-v1\n").unwrap();
    fs::remove_dir(root.join("build")).unwrap();
    assert!(checks::evaluate(&root).is_err());
}

#[cfg(unix)]
#[test]
fn symlinked_case_outputs_are_rejected() {
    use std::os::unix::fs::symlink;
    let temp = Sandbox::new();
    let root = temp.case();
    case::start(&root).unwrap();
    fs::write(root.join("project/config/service.conf"), "timeout_ms=800\n").unwrap();
    case::simulate(&root).unwrap();
    fs::remove_file(root.join("logs/current.log")).unwrap();
    let innocent = temp.0.join("outside.log");
    fs::write(&innocent, "keep-me").unwrap();
    symlink(&innocent, root.join("logs/current.log")).unwrap();
    assert!(case::simulate(&root).is_err());
    assert_eq!(fs::read_to_string(innocent).unwrap(), "keep-me");

    // Even when the deliverables directory exists under an attacker-controlled path,
    // board/finish/test-app must reject it and cannot write to external targets.
    fs::rename(root.join("deliverables"), root.join("deliverables.real")).unwrap();
    symlink(&temp.0, root.join("deliverables")).unwrap();
    assert!(checks::evaluate(&root).is_err());
    assert!(board::write(&root, &[]).is_err());
}

#[test]
fn cli_helps_navigate_errors_without_destroying_files() {
    let temp = Sandbox::new();
    let root = temp.case();
    let exe = env!("CARGO_BIN_EXE_labquest");
    let first = Command::new(exe).arg("start").arg(&root).output().unwrap();
    assert!(first.status.success());
    let status = Command::new(exe).arg("status").arg(&root).output().unwrap();
    let text = String::from_utf8_lossy(&status.stdout);
    assert!(text.contains("0/6"));
    assert!(text.contains("hint 1"));
    let hint = Command::new(exe).args(["hint", "5", "2"]).output().unwrap();
    assert!(hint.status.success());
    assert!(String::from_utf8_lossy(&hint.stdout).contains("迭代器"));
    let wrong_hint = Command::new(exe).args(["hint", "7"]).output().unwrap();
    assert!(!wrong_hint.status.success());
    let premature = Command::new(exe).arg("finish").arg(&root).output().unwrap();
    assert!(!premature.status.success());
    assert!(!root.join("deliverables/CASE_REPORT.md").exists());
}

#[test]
fn hardcoded_success_for_one_configuration_fails_transfer_verification() {
    let temp = Sandbox::new();
    let root = temp.case();
    case::start(&root).unwrap();
    // 固定 0 会通过当前 800ms 的错误数，但不能处理其他输入。
    fs::write(root.join("project/config/service.conf"), "timeout_ms=800\n").unwrap();
    let path = root.join("project/src/main.rs");
    let source = fs::read_to_string(&path).unwrap();
    fs::write(
        &path,
        source.replace(
            "    // TODO(CASE-01):",
            "    println!(\"SUMMARY requests=12 errors=0\");\n    // TODO(CASE-01):",
        ),
    )
    .unwrap();
    let error = proof::test_app(&root).unwrap_err();
    assert!(error.contains("timeout_ms=150"), "{error}");
    assert!(error.contains("不能写死"), "{error}");
    assert!(!root.join("deliverables/PROGRAM_PROOF.txt").exists());
    assert!(!root.join("logs/program.log").exists());
    // 测试没有写入 service.conf，也没有触碰 logs/incident.log。
    assert_eq!(
        fs::read_to_string(root.join("project/config/service.conf")).unwrap(),
        "timeout_ms=800\n"
    );
}

#[test]
fn greater_equal_bug_is_caught_on_the_400ms_boundary() {
    let temp = Sandbox::new();
    let root = temp.case();
    case::start(&root).unwrap();
    fs::write(root.join("project/config/service.conf"), "timeout_ms=800\n").unwrap();
    patched_source(&root);
    let path = root.join("project/src/main.rs");
    let wrong = fs::read_to_string(&path)
        .unwrap()
        .replace("ms > timeout", "ms >= timeout");
    fs::write(&path, wrong).unwrap();
    let error = proof::test_app(&root).unwrap_err();
    assert!(error.contains("timeout_ms=400"), "{error}");
    assert!(!proof::check(&root));
}

#[test]
fn prior_case_marker_still_supports_legacy_evidence() {
    let temp = Sandbox::new();
    let root = temp.case();
    case::start(&root).unwrap();
    fs::write(root.join(".labquest-case"), "RustAppLab-LabQuest-Case-v1\n").unwrap();
    assert_eq!(case::version(&root).unwrap(), case::CaseVersion::Legacy);
    fs::write(root.join("project/config/service.conf"), "timeout_ms=800\n").unwrap();
    // 旧案例的单配置验证保持兼容，并明确展示 v1 证明标签。
    patched_source(&root);
    proof::test_app(&root).unwrap();
    assert!(proof::check(&root));
    assert!(
        fs::read_to_string(root.join("deliverables/PROGRAM_PROOF.txt"))
            .unwrap()
            .contains("LabQuest-Program-Proof-v1")
    );
}

#[test]
fn next_and_predict_are_read_only_cli_learning_aids() {
    let temp = Sandbox::new();
    let root = temp.case();
    case::start(&root).unwrap();
    let exe = env!("CARGO_BIN_EXE_labquest");
    let before = fs::read_to_string(root.join("project/config/service.conf")).unwrap();
    let next = Command::new(exe).arg("next").arg(&root).output().unwrap();
    let text = String::from_utf8_lossy(&next.stdout);
    assert!(next.status.success());
    assert!(text.contains("Rust 知识"));
    assert!(text.contains("filequest"));
    let practice = Command::new(exe).arg("practice").output().unwrap();
    assert!(String::from_utf8_lossy(&practice.stdout).contains("先预测"));
    let predict = Command::new(exe)
        .args(["predict", "400", "6"])
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&predict.stdout).contains("预测正确"));
    let invalid = Command::new(exe)
        .args(["predict", "0", "0"])
        .output()
        .unwrap();
    assert!(!invalid.status.success());
    assert_eq!(
        fs::read_to_string(root.join("project/config/service.conf")).unwrap(),
        before
    );
    assert!(!root.join("deliverables/inventory.html").exists());
}
