//! 每关实验的失败态：学员选择运行一项，修改功能直到转绿。默认全部跳过。
use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn workspace() -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!(
        "sfs-challenge-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&p).unwrap();
    p
}

#[test]
#[ignore = "第 17 关：学员增加 verify --quiet；成功静默输出，损坏时仍返回非零状态"]
fn mission_17_verify_quiet() {
    let dir = workspace();
    fs::create_dir(dir.join("src")).unwrap();
    fs::write(dir.join("src/a"), b"data").unwrap();
    let bin = env!("CARGO_BIN_EXE_safesync");
    let backup = Command::new(bin)
        .args([
            "backup",
            dir.join("src").to_str().unwrap(),
            dir.join("bak").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(backup.status.success());
    let output = Command::new(bin)
        .args(["verify", dir.join("bak").to_str().unwrap(), "--quiet"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    fs::remove_dir_all(dir).unwrap();
}

#[test]
#[ignore = "第 18 关：学员增加 restore --dry-run；显示计划但不得创建目标"]
fn mission_18_restore_dry_run() {
    let dir = workspace();
    fs::create_dir(dir.join("src")).unwrap();
    fs::write(dir.join("src/a"), b"data").unwrap();
    let bin = env!("CARGO_BIN_EXE_safesync");
    assert!(Command::new(bin)
        .args([
            "backup",
            dir.join("src").to_str().unwrap(),
            dir.join("bak").to_str().unwrap()
        ])
        .status()
        .unwrap()
        .success());
    let dest = dir.join("preview");
    let output = Command::new(bin)
        .args([
            "restore",
            dir.join("bak").to_str().unwrap(),
            dest.to_str().unwrap(),
            "--dry-run",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(!dest.exists());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Dry-run"));
    fs::remove_dir_all(dir).unwrap();
}

#[test]
#[ignore = "第 19 关：学员增加 verify --json，输出可供脚本使用的机器可读计数"]
fn mission_19_verify_json() {
    let dir = workspace();
    fs::create_dir(dir.join("src")).unwrap();
    fs::write(dir.join("src/a"), b"data").unwrap();
    let bin = env!("CARGO_BIN_EXE_safesync");
    assert!(Command::new(bin)
        .args([
            "backup",
            dir.join("src").to_str().unwrap(),
            dir.join("bak").to_str().unwrap()
        ])
        .status()
        .unwrap()
        .success());
    let output = Command::new(bin)
        .args(["verify", dir.join("bak").to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("\"files\":1") && text.contains("\"verified\":true"));
    fs::remove_dir_all(dir).unwrap();
}
