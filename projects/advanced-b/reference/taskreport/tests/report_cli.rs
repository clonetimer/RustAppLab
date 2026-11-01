use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
#[test]
fn markdown_report_is_another_process() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let folder = std::env::temp_dir().join(format!("report_cli-{}-{nonce}", std::process::id()));
    fs::create_dir(&folder).unwrap();
    let db = folder.join("我的任务.tsv");
    fs::write(&db, "2\tDONE\t整理笔记\n1\tOPEN\t学习 Rust\n").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_taskpocket-report"))
        .arg(&db)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let txt = String::from_utf8(out.stdout).unwrap();
    assert!(txt.contains("总数：2，待办：1，完成：1"));
    assert!(txt.contains("学习 Rust") && txt.contains("整理笔记"));
    fs::remove_dir_all(folder).unwrap();
}
