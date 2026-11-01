//! 实际 CLI 入口与 0.9.0 兼容性测试。
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn cli(parts: &[&std::ffi::OsStr]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_labquest"))
        .args(parts)
        .output()
        .expect("能够运行 labquest")
}

#[test]
fn catalog_and_hint_commands_are_accessible() {
    let out = Command::new(env!("CARGO_BIN_EXE_labquest"))
        .args(["app", "catalog"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let msg = String::from_utf8(out.stdout).unwrap();
    assert!(msg.contains("taskpocket") && msg.contains("pagemint"));
    let hint = Command::new(env!("CARGO_BIN_EXE_labquest"))
        .args(["app", "hint", "pagemint", "1"])
        .output()
        .unwrap();
    assert!(hint.status.success());
    assert!(String::from_utf8(hint.stdout).unwrap().contains("stats"));
    assert!(!Command::new(env!("CARGO_BIN_EXE_labquest"))
        .args(["app", "hint", "pagemint", "4"])
        .status()
        .unwrap()
        .success());
}

#[test]
fn both_legacy_and_new_init_are_supported() {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let base = std::env::temp_dir().join(format!("rustapplab-cli-{}-{nanos}", std::process::id()));
    fs::create_dir(&base).unwrap();
    let pagemint = base.join("web");
    let taskpocket = base.join("tasks");
    let legacy = base.join("legacy");
    let pagemint_s = pagemint.as_os_str();
    let taskpocket_s = taskpocket.as_os_str();
    let legacy_s = legacy.as_os_str();
    let ok = |args: &[&std::ffi::OsStr]| {
        let out = cli(args);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    };
    ok(&[
        "app".as_ref(),
        "init".as_ref(),
        "pagemint".as_ref(),
        pagemint_s,
    ]);
    ok(&[
        "app".as_ref(),
        "init".as_ref(),
        "taskpocket".as_ref(),
        taskpocket_s,
    ]);
    ok(&["app".as_ref(), "init".as_ref(), legacy_s]);
    assert!(pagemint.join("samples/notes/01-first.note").exists());
    assert!(taskpocket.join("CONTRACT.md").exists());
    assert!(legacy.join("CONTRACT.md").exists());
    assert!(!cli(&[
        "app".as_ref(),
        "init".as_ref(),
        "pagemint".as_ref(),
        pagemint_s
    ])
    .status
    .success());
    let _ = fs::remove_dir_all(base);
}

#[test]
fn pagemint_check_failure_creates_report_and_returns_nonzero() {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let base: PathBuf =
        std::env::temp_dir().join(format!("rustapplab-grade-{}-{nanos}", std::process::id()));
    fs::create_dir(&base).unwrap();
    let project = base.join("web");
    assert!(cli(&[
        "app".as_ref(),
        "init".as_ref(),
        "pagemint".as_ref(),
        project.as_os_str()
    ])
    .status
    .success());
    let output = cli(&[
        "app".as_ref(),
        "check".as_ref(),
        "pagemint".as_ref(),
        project.as_os_str(),
    ]);
    assert!(!output.status.success());
    assert!(project.join("APP_ASSESSMENT.md").is_file());
    assert!(fs::read_to_string(project.join("APP_ASSESSMENT.md"))
        .unwrap()
        .contains("FAIL"));
    let _ = fs::remove_dir_all(base);
}
