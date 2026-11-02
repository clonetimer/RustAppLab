use labquest::advanced_b;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
static NONCE: AtomicU64 = AtomicU64::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "rustapplab-b-{}-{t}-{}",
            std::process::id(),
            NONCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        Self(root)
    }
    fn project(&self) -> PathBuf {
        self.0.join("my-taskpocket-pro")
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn initial_red_then_reference_green_and_stale() {
    let tmp = Scratch::new();
    let p = tmp.project();
    advanced_b::init(&p).unwrap();
    assert!(advanced_b::is_project(&p));
    assert!(advanced_b::init(&p).is_err());
    assert!(!advanced_b::check(&p, 1).unwrap());
    let next = advanced_b::next(&p).unwrap();
    assert!(next.contains("B1"));
    assert!(!next.contains("B1..B0"), "{next}");
    fs::write(
        p.join("taskcore/src/lib.rs"),
        include_str!("../../../projects/advanced-b/reference/taskcore/src/lib.rs"),
    )
    .unwrap();
    assert!(advanced_b::check(&p, 3).unwrap());
    assert!(advanced_b::next(&p).unwrap().contains("3/3"));
    let mut src = fs::read_to_string(p.join("taskcore/src/lib.rs")).unwrap();
    src.push_str("\n// personal regression test\n");
    fs::write(p.join("taskcore/src/lib.rs"), src).unwrap();
    assert!(advanced_b::next(&p).unwrap().contains("过期"));
}
#[test]
fn editing_visible_student_tests_does_not_bypass_fixed_contract() {
    let tmp = Scratch::new();
    let p = tmp.project();
    advanced_b::init(&p).unwrap();
    fs::write(
        p.join("taskcore/tests/b1.rs"),
        "#[test]fn fake(){assert!(true);}\n",
    )
    .unwrap();
    assert!(!advanced_b::check(&p, 1).unwrap());
}
#[cfg(unix)]
#[test]
fn symlink_source_rejected_before_running_cargo() {
    use std::os::unix::fs::symlink;
    let tmp = Scratch::new();
    let p = tmp.project();
    advanced_b::init(&p).unwrap();
    fs::remove_file(p.join("taskcore/src/lib.rs")).unwrap();
    symlink("/etc/hosts", p.join("taskcore/src/lib.rs")).unwrap();
    assert!(advanced_b::check(&p, 1).is_err());
}

#[test]
fn stage_one_does_not_require_unfinished_business_methods() {
    let tmp = Scratch::new();
    let p = tmp.project();
    advanced_b::init(&p).unwrap();
    let refcode = include_str!("../../../projects/advanced-b/reference/taskcore/src/lib.rs");
    let begin = refcode.find("pub fn add(").unwrap();
    let end = refcode.find("pub fn load(").unwrap();
    let partial = format!("{}{}{}", &refcode[..begin],
      "pub fn add(_tasks:&mut Vec<Task>,_title:&str)->Result<u64,String>{Err(\"not implemented\".into())}\npub fn complete(_tasks:&mut [Task],_id:u64)->Result<(),String>{Err(\"not implemented\".into())}\n",
      &refcode[end..]);
    fs::write(p.join("taskcore/src/lib.rs"), partial).unwrap();
    assert!(advanced_b::check(&p, 1).unwrap());
    assert!(!advanced_b::check(&p, 2).unwrap());
}

fn reference_project(tmp: &Scratch) -> PathBuf {
    let project = tmp.project();
    advanced_b::init(&project).unwrap();
    fs::write(
        project.join("taskcore/src/lib.rs"),
        include_str!("../../../projects/advanced-b/reference/taskcore/src/lib.rs"),
    )
    .unwrap();
    project
}

fn latest_report(project: &std::path::Path) -> String {
    let mut reports: Vec<_> = fs::read_dir(project.join(".advanced-b-reports"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    reports.sort();
    fs::read_to_string(reports.last().unwrap()).unwrap()
}

#[test]
fn b3_rejects_a_cli_with_broken_list_output() {
    let tmp = Scratch::new();
    let project = reference_project(&tmp);
    let cli = project.join("taskcli/src/main.rs");
    let source = fs::read_to_string(&cli).unwrap();
    assert!(source.contains("print!(\"{}\", taskcore::to_tsv(&tasks));"));
    fs::write(
        cli,
        source.replace(
            "print!(\"{}\", taskcore::to_tsv(&tasks));",
            "println!(\"broken list\");",
        ),
    )
    .unwrap();
    assert!(!advanced_b::check(&project, 3).unwrap());
    assert!(latest_report(&project).contains("passed=2\n"));
}

#[test]
fn b3_rechecks_b1_and_b2_before_running_integration() {
    for (needle, replacement, passed) in [
        ("total: tasks.len(),", "total: 0,", 0),
        (".checked_add(1)", ".checked_add(2)", 1),
    ] {
        let tmp = Scratch::new();
        let project = reference_project(&tmp);
        let core = project.join("taskcore/src/lib.rs");
        let source = fs::read_to_string(&core).unwrap();
        assert!(source.contains(needle));
        fs::write(core, source.replace(needle, replacement)).unwrap();
        assert!(!advanced_b::check(&project, 3).unwrap());
        let report = latest_report(&project);
        assert!(report.contains(&format!("passed={passed}\n")), "{report}");
        assert!(!report.contains("=== B3 ==="), "{report}");
    }
}

#[test]
fn b3_requires_the_report_executable_to_succeed() {
    let tmp = Scratch::new();
    let project = reference_project(&tmp);
    fs::write(
        project.join("taskreport/src/main.rs"),
        "fn main() { std::process::exit(2); }\n",
    )
    .unwrap();
    assert!(!advanced_b::check(&project, 3).unwrap());
    let report = latest_report(&project);
    assert!(report.contains("=== B3-report ==="), "{report}");
    assert!(report.contains("passed=2\n"), "{report}");
}

#[test]
fn b3_requires_both_title_feature_configurations() {
    let strict_check = "    #[cfg(feature = \"strict-titles\")]\n    if t.chars().count() > 80 {\n        return Err(\"strict-titles: 标题超过80个字符\".into());\n    }\n";
    for (removed, failed_check) in [
        (strict_check, "B3-strict-titles"),
        (
            "    #[cfg(feature = \"strict-titles\")]\n",
            "B3-default-titles",
        ),
    ] {
        let tmp = Scratch::new();
        let project = reference_project(&tmp);
        let core = project.join("taskcore/src/lib.rs");
        let source = fs::read_to_string(&core).unwrap();
        assert!(source.contains(removed));
        fs::write(core, source.replace(removed, "")).unwrap();
        assert!(!advanced_b::check(&project, 3).unwrap());
        let report = latest_report(&project);
        assert!(report.contains("passed=2\n"), "{report}");
        assert!(
            report.contains(&format!("=== {failed_check} ===")),
            "{report}"
        );
    }
}

#[test]
fn old_contract_report_requires_read_only_recheck_for_unchanged_sources() {
    let tmp = Scratch::new();
    let project = reference_project(&tmp);
    let cli = project.join("taskcli/src/main.rs");
    let source = fs::read_to_string(&cli).unwrap();
    fs::write(
        &cli,
        source.replace(
            "print!(\"{}\", taskcore::to_tsv(&tasks));",
            "println!(\"broken list\");",
        ),
    )
    .unwrap();
    assert!(advanced_b::check(&project, 1).unwrap());
    let report_path = fs::read_dir(project.join(".advanced-b-reports"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let report = fs::read_to_string(&report_path).unwrap();
    let source_hash = report
        .lines()
        .find(|line| line.starts_with("source_sha256="))
        .unwrap();
    let legacy = format!("RustAppLab Advanced B Report v1\n{source_hash}\npassed=3\nrequested=3\n");
    fs::write(&report_path, &legacy).unwrap();
    let marker = fs::read(project.join(".applab-advanced-b")).unwrap();
    let cli_before = fs::read(&cli).unwrap();
    let core_before = fs::read(project.join("taskcore/src/lib.rs")).unwrap();

    let next = advanced_b::next(&project).unwrap();
    assert!(next.contains("合同") && next.contains("更新"), "{next}");
    assert!(
        next.contains("advanced-b check") && next.contains('3'),
        "{next}"
    );
    assert!(!next.contains("3/3"), "{next}");
    assert_eq!(fs::read_to_string(&report_path).unwrap(), legacy);
    assert_eq!(
        fs::read_dir(project.join(".advanced-b-reports"))
            .unwrap()
            .count(),
        1
    );
    assert_eq!(
        fs::read(project.join(".applab-advanced-b")).unwrap(),
        marker
    );
    assert_eq!(fs::read(&cli).unwrap(), cli_before);
    assert_eq!(
        fs::read(project.join("taskcore/src/lib.rs")).unwrap(),
        core_before
    );

    assert!(!advanced_b::check(&project, 3).unwrap());
    let current_report = latest_report(&project);
    assert!(current_report.starts_with("RustAppLab Advanced B Report v2\n"));
    assert!(current_report.contains("passed=2\n"));
    assert!(advanced_b::next(&project)
        .unwrap()
        .contains("已通过 B1..B2"));
}
