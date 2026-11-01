use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let folder = std::env::temp_dir().join(format!(
            "sfs-cli-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&folder).unwrap();
        Self(folder)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_safesync"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn help_missions_and_bad_arguments() {
    let help = run(&["help"]);
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("SHA-256"));
    assert!(String::from_utf8_lossy(&run(&["missions"]).stdout).contains("16"));
    assert!(!run(&["restore", "a", "b", "--force"]).status.success());
    assert!(!run(&["backup", "a"]).status.success());
}

#[test]
fn demo_verify_tamper_and_restore_is_real_cli_flow() {
    let root = Fixture::new();
    let folder = root.0.join("demo");
    let as_text = folder.to_str().unwrap();
    let output = run(&["demo", as_text]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("已校验"));
    assert!(run(&["verify", folder.join("backup").to_str().unwrap()])
        .status
        .success());
    assert!(run(&[
        "backup",
        folder.join("source").to_str().unwrap(),
        folder.join("backup").to_str().unwrap()
    ])
    .status
    .success());
    assert!(!run(&["demo", as_text]).status.success());
    let restored = folder.join("second-restore");
    assert!(run(&[
        "restore",
        folder.join("backup").to_str().unwrap(),
        restored.to_str().unwrap()
    ])
    .status
    .success());
    assert_eq!(
        fs::read(restored.join("binary.dat")).unwrap(),
        [0, 1, 2, 0, 255, 0, 17]
    );
    fs::write(
        folder.join("backup/data/binary.dat"),
        [2, 1, 2, 0, 255, 0, 17],
    )
    .unwrap();
    assert!(!run(&["verify", folder.join("backup").to_str().unwrap()])
        .status
        .success());
    let fail_target = folder.join("should-not-appear");
    assert!(!run(&[
        "restore",
        folder.join("backup").to_str().unwrap(),
        fail_target.to_str().unwrap()
    ])
    .status
    .success());
    assert!(!fail_target.exists());
}
