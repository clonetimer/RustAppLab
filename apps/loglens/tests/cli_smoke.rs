use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_loglens"))
        .args(args)
        .output()
        .expect("运行 LogLens")
}

#[test]
fn demo_analyze_compare_report_user_journey() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let parent = std::env::temp_dir().join(format!("loglens-cli-{}-{stamp}", std::process::id()));
    fs::create_dir(&parent).unwrap();
    let root = parent.join("demo");
    let before = root.join("baseline.log");
    let after = root.join("candidate.log");
    let report = parent.join("analysis.md");
    let to_arg = |path: &PathBuf| path.to_string_lossy().into_owned();
    assert!(run(&["help"]).status.success());
    assert!(!run(&["no-such-command"]).status.success());
    assert!(run(&["demo", &to_arg(&root)]).status.success());
    assert!(run(&["demo", &to_arg(&root)]).status.code().unwrap_or(0) != 0);
    let analyze = run(&["analyze", &to_arg(&after)]);
    assert!(analyze.status.success());
    assert!(String::from_utf8_lossy(&analyze.stdout).contains("未识别：1"));
    let strict = run(&["analyze", &to_arg(&after), "--strict"]);
    assert!(!strict.status.success());
    let compare = run(&["compare", &to_arg(&before), &to_arg(&after)]);
    assert!(compare.status.success());
    assert!(String::from_utf8_lossy(&compare.stdout).contains("+25.00 个百分点"));
    let original_log = fs::read(&after).unwrap();
    let same_file = run(&["report", &to_arg(&after), "-o", &to_arg(&after), "--force"]);
    assert!(!same_file.status.success(), "不能用报告覆盖输入日志本身");
    assert_eq!(fs::read(&after).unwrap(), original_log);
    let generated = run(&[
        "report",
        &to_arg(&after),
        "-o",
        &to_arg(&report),
        "--top",
        "1",
    ]);
    assert!(generated.status.success());
    let md = fs::read_to_string(&report).unwrap();
    assert!(md.contains("## 高频消息"));
    assert!(md.contains("未识别行（共 1 行"));
    assert!(!run(&["report", &to_arg(&after), "-o", &to_arg(&report)])
        .status
        .success());
    assert!(
        run(&["report", &to_arg(&after), "-o", &to_arg(&report), "--force"])
            .status
            .success()
    );
    fs::remove_dir_all(parent).unwrap();
}
