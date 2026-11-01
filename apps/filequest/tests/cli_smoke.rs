//! 端到端烟雾测试：与用户一样执行编译后的程序。
use std::fs;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_filequest"))
        .args(args)
        .output()
        .expect("执行 filequest")
}

#[test]
fn help_and_unknown_commands() {
    let help = run(&["help"]);
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("FileQuest 0.1.0"));
    let invalid = run(&["does-not-exist"]);
    assert!(!invalid.status.success());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("未知命令"));
}

#[test]
fn demo_scan_find_report_is_a_real_user_journey() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let parent = std::env::temp_dir().join(format!("filequest-cli-{}-{stamp}", std::process::id()));
    fs::create_dir(&parent).unwrap();
    let root = parent.join("demo");
    let user_report = parent.join("outside.html");
    let root_string = root.to_string_lossy().to_string();
    let report_string = user_report.to_string_lossy().to_string();

    let demonstration = run(&["demo", &root_string]);
    assert!(
        demonstration.status.success(),
        "{}",
        String::from_utf8_lossy(&demonstration.stderr)
    );
    assert!(root.join("report.html").is_file());
    assert!(String::from_utf8_lossy(&demonstration.stdout).contains("9 个样例文件"));
    let duplicate = run(&["demo", &root_string]);
    assert!(!duplicate.status.success(), "demo 不得覆盖已有目录");

    let scan = run(&["scan", &root_string]);
    assert!(scan.status.success());
    assert!(String::from_utf8_lossy(&scan.stdout).contains("普通文件：10"));

    let find = run(&["find", &root_string, "LOG"]);
    assert!(find.status.success());
    assert!(String::from_utf8_lossy(&find.stdout).contains("共找到 2 个文件"));

    let report = run(&["report", &root_string, "-o", &report_string, "--limit", "5"]);
    assert!(report.status.success());
    let html = fs::read_to_string(&user_report).unwrap();
    assert!(html.contains("显示前 5 / 10 个文件"));

    let repeat = run(&["report", &root_string, "-o", &report_string]);
    assert!(!repeat.status.success());
    let forced = run(&["report", &root_string, "-o", &report_string, "--force"]);
    assert!(forced.status.success());

    fs::remove_dir_all(&parent).unwrap();
}
