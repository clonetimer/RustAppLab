//! 真进程退出（exit 86，无 Drop 清理）下的暂存状态与受控恢复验收。
//! 故障注入仅存在于 debug 构建；这不是断电持久性或攻击对抗测试。
#[cfg(debug_assertions)]
use safesync::ops::BackupStatus;
use safesync::ops::{self, StageStatus};
use std::fs;
#[cfg(debug_assertions)]
use std::path::Path;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let base = std::env::temp_dir().join(format!(
            "sfs-recovery-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&base).unwrap();
        Self(base)
    }
    fn at(&self, part: &str) -> PathBuf {
        self.0.join(part)
    }
    #[cfg(debug_assertions)]
    fn fixture(&self) {
        fs::create_dir(self.at("source")).unwrap();
        fs::write(self.at("source/alpha.txt"), b"alpha\n").unwrap();
        fs::write(self.at("source/beta.bin"), [0, 1, 2, 3, 255]).unwrap();
    }
    fn stages(&self) -> Vec<ops::StageInfo> {
        ops::stages(&self.0).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[cfg(debug_assertions)]
fn with_fault(args: &[&Path], point: &str) -> std::process::Output {
    let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_safesync"));
    for arg in args {
        command.arg(arg);
    }
    command
        .env("RUST_APPLAB_SAFESYNC_FAULT", point)
        .output()
        .unwrap()
}

#[cfg(debug_assertions)]
#[test]
fn verified_stage_can_be_committed_after_abrupt_exit_without_source() {
    let fixture = Fixture::new();
    fixture.fixture();
    let result = with_fault(
        &[
            Path::new("backup"),
            &fixture.at("source"),
            &fixture.at("snapshot"),
        ],
        "backup_after_verify",
    );
    assert_eq!(result.status.code(), Some(86));
    assert!(!fixture.at("snapshot").exists());
    let found = fixture.stages();
    assert_eq!(found.len(), 1);
    assert!(matches!(found[0].status, StageStatus::Ready(_)));
    let stage = found[0].path.clone();
    // 故意证明恢复不需要已不存在的原始数据。
    fs::remove_dir_all(fixture.at("source")).unwrap();
    assert_eq!(
        ops::recover(&stage, &fixture.at("snapshot")).unwrap().files,
        2
    );
    assert!(!stage.exists());
    assert_eq!(ops::verify(&fixture.at("snapshot")).unwrap().files, 2);
    assert_eq!(
        fs::read(fixture.at("snapshot/data/beta.bin")).unwrap(),
        [0, 1, 2, 3, 255]
    );
    assert!(fixture.stages().is_empty());
}

#[cfg(debug_assertions)]
#[test]
fn halfway_copied_stage_is_never_promoted_and_can_be_restarted() {
    let fixture = Fixture::new();
    fixture.fixture();
    let result = with_fault(
        &[
            Path::new("backup"),
            &fixture.at("source"),
            &fixture.at("snapshot"),
        ],
        "backup_after_first_file",
    );
    assert_eq!(result.status.code(), Some(86));
    let found = fixture.stages();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].status, StageStatus::Incomplete);
    let stage = found[0].path.clone();
    assert!(ops::recover(&stage, &fixture.at("snapshot")).is_err());
    assert!(stage.exists(), "校验失败时不得擅自删除暂存数据");
    assert!(!fixture.at("snapshot").exists());
    assert_eq!(
        ops::backup(&fixture.at("source"), &fixture.at("snapshot"))
            .unwrap()
            .0,
        BackupStatus::Created,
    );
    assert!(stage.exists(), "重新备份也不应清理未知遗留物");
    assert_eq!(ops::verify(&fixture.at("snapshot")).unwrap().files, 2);
    assert!(ops::recover(&stage, &fixture.at("snapshot")).is_err());
}

#[cfg(debug_assertions)]
#[test]
fn interrupted_restore_must_restart_and_cannot_be_promoted_as_backup() {
    let fixture = Fixture::new();
    fixture.fixture();
    ops::backup(&fixture.at("source"), &fixture.at("snapshot")).unwrap();
    let result = with_fault(
        &[
            Path::new("restore"),
            &fixture.at("snapshot"),
            &fixture.at("output"),
        ],
        "restore_after_first_file",
    );
    assert_eq!(result.status.code(), Some(86));
    let found = fixture.stages();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].status, StageStatus::RestoreStage);
    assert!(!fixture.at("output").exists());
    assert!(ops::recover(&found[0].path, &fixture.at("output")).is_err());
    assert_eq!(
        ops::restore(&fixture.at("snapshot"), &fixture.at("output"))
            .unwrap()
            .files,
        2
    );
    assert_eq!(
        fs::read(fixture.at("output/alpha.txt")).unwrap(),
        b"alpha\n"
    );
    assert!(found[0].path.exists());
}

#[cfg(debug_assertions)]
#[test]
fn recover_rejects_wrong_name_different_parent_and_existing_target() {
    let fixture = Fixture::new();
    fixture.fixture();
    let result = with_fault(
        &[
            Path::new("backup"),
            &fixture.at("source"),
            &fixture.at("snapshot"),
        ],
        "backup_after_verify",
    );
    assert_eq!(result.status.code(), Some(86));
    let stage = fixture.stages()[0].path.clone();
    fs::create_dir(fixture.at("other-parent")).unwrap();
    assert!(ops::recover(&stage, &fixture.at("other-parent/snapshot")).is_err());
    assert!(ops::recover(&stage, &fixture.at("not-the-snapshot")).is_err());
    fs::create_dir(fixture.at("snapshot")).unwrap();
    fs::write(fixture.at("snapshot/untouched"), "important").unwrap();
    assert!(ops::recover(&stage, &fixture.at("snapshot")).is_err());
    assert_eq!(
        fs::read(fixture.at("snapshot/untouched")).unwrap(),
        b"important"
    );
    assert!(stage.exists());
}

#[cfg(unix)]
#[test]
fn stages_lists_symlink_without_following_it() {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new();
    let outside = fixture.at("other");
    fs::create_dir(&outside).unwrap();
    symlink(&outside, fixture.at(".sfs-foreign-111-222-0.part")).unwrap();
    let found = fixture.stages();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].status, StageStatus::Unsafe);
    assert!(outside.exists());
}

#[cfg(debug_assertions)]
#[test]
fn cli_stages_and_recover_are_executable_user_workflow() {
    let fixture = Fixture::new();
    fixture.fixture();
    let exit = with_fault(
        &[
            Path::new("backup"),
            &fixture.at("source"),
            &fixture.at("snapshot"),
        ],
        "backup_after_verify",
    );
    assert_eq!(exit.status.code(), Some(86));
    let listing = std::process::Command::new(env!("CARGO_BIN_EXE_safesync"))
        .arg("stages")
        .arg(&fixture.0)
        .output()
        .unwrap();
    assert!(listing.status.success());
    assert!(String::from_utf8_lossy(&listing.stdout).contains("READY:"));
    let stage = fixture.stages()[0].path.clone();
    let recovered = std::process::Command::new(env!("CARGO_BIN_EXE_safesync"))
        .arg("recover")
        .arg(&stage)
        .arg(fixture.at("snapshot"))
        .output()
        .unwrap();
    assert!(recovered.status.success(), "{:?}", recovered.stderr);
    assert!(String::from_utf8_lossy(&recovered.stdout).contains("Recover OK"));
    assert_eq!(ops::verify(&fixture.at("snapshot")).unwrap().files, 2);
}

#[cfg(debug_assertions)]
#[test]
fn ordinary_io_error_rolls_back_staging_through_drop() {
    let fixture = Fixture::new();
    fixture.fixture();
    let result = with_fault(
        &[
            Path::new("backup"),
            &fixture.at("source"),
            &fixture.at("snapshot"),
        ],
        "backup_after_first_file_error",
    );
    assert!(!result.status.success());
    assert_ne!(result.status.code(), Some(86));
    assert!(!fixture.at("snapshot").exists());
    assert!(
        fixture.stages().is_empty(),
        "普通 I/O 错误返回时 Drop 应释放临时目录"
    );
    assert_eq!(
        ops::backup(&fixture.at("source"), &fixture.at("snapshot"))
            .unwrap()
            .0,
        BackupStatus::Created
    );
}

#[cfg(debug_assertions)]
#[test]
fn simulated_mid_file_write_error_rolls_back_and_leaves_source_intact() {
    let fixture = Fixture::new();
    fs::create_dir(fixture.at("source")).unwrap();
    let bytes = vec![0xA5u8; 256 * 1024];
    fs::write(fixture.at("source/blob.bin"), &bytes).unwrap();
    let result = with_fault(
        &[
            Path::new("backup"),
            &fixture.at("source"),
            &fixture.at("snapshot"),
        ],
        "backup_mid_file_io_error",
    );
    assert!(!result.status.success());
    assert!(!fixture.at("snapshot").exists());
    assert!(fixture.stages().is_empty(), "错误返回后必须触发 Drop 清理");
    assert_eq!(fs::read(fixture.at("source/blob.bin")).unwrap(), bytes);
    assert_eq!(
        ops::backup(&fixture.at("source"), &fixture.at("snapshot"))
            .unwrap()
            .1
            .files,
        1
    );
}

#[cfg(debug_assertions)]
#[test]
fn complete_restore_stage_is_not_a_recoverable_backup_even_if_layout_matches() {
    let fixture = Fixture::new();
    fixture.fixture();
    ops::backup(&fixture.at("source"), &fixture.at("inner")).unwrap();

    // 制造一个源目录，其内容碰巧就是另一份合法备份的 data/ + manifest.sfs。
    // 0.5.0 的无类型暂存若在此时中断，会因通过 verify 而被误标为 READY。
    fs::create_dir_all(fixture.at("backup-shaped-source/data")).unwrap();
    fs::copy(
        fixture.at("inner/manifest.sfs"),
        fixture.at("backup-shaped-source/manifest.sfs"),
    )
    .unwrap();
    fs::copy(
        fixture.at("inner/data/alpha.txt"),
        fixture.at("backup-shaped-source/data/alpha.txt"),
    )
    .unwrap();
    fs::copy(
        fixture.at("inner/data/beta.bin"),
        fixture.at("backup-shaped-source/data/beta.bin"),
    )
    .unwrap();
    ops::backup(&fixture.at("backup-shaped-source"), &fixture.at("outer")).unwrap();

    let result = with_fault(
        &[
            Path::new("restore"),
            &fixture.at("outer"),
            &fixture.at("output"),
        ],
        "restore_after_verify",
    );
    assert_eq!(result.status.code(), Some(86));
    let found = fixture.stages();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].status, StageStatus::RestoreStage);
    assert_eq!(
        ops::verify(&found[0].path).unwrap().files,
        2,
        "内容本身恰巧是一份合法备份"
    );
    assert!(ops::recover(&found[0].path, &fixture.at("output")).is_err());
    assert!(ops::recover_legacy(&found[0].path, &fixture.at("output")).is_err());
    assert!(!fixture.at("output").exists());
    assert!(found[0].path.exists());
}

#[cfg(debug_assertions)]
#[test]
fn older_untyped_stage_requires_explicit_legacy_recovery() {
    let fixture = Fixture::new();
    fixture.fixture();
    let result = with_fault(
        &[
            Path::new("backup"),
            &fixture.at("source"),
            &fixture.at("snapshot"),
        ],
        "backup_after_verify",
    );
    assert_eq!(result.status.code(), Some(86));
    let current = fixture.stages()[0].path.clone();
    let legacy = fixture.at(".sfs-snapshot-100-123456-0.part");
    fs::rename(&current, &legacy).unwrap();
    assert!(matches!(
        fixture.stages()[0].status,
        StageStatus::LegacyReady(_)
    ));
    assert!(ops::recover(&legacy, &fixture.at("snapshot")).is_err());
    assert!(legacy.exists());
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_safesync"))
        .arg("recover-legacy")
        .arg(&legacy)
        .arg(fixture.at("snapshot"))
        .output()
        .unwrap();
    assert!(result.status.success(), "{:?}", result.stderr);
    assert!(String::from_utf8_lossy(&result.stdout).contains("Recover legacy OK"));
    assert_eq!(ops::verify(&fixture.at("snapshot")).unwrap().files, 2);
}

#[cfg(debug_assertions)]
#[test]
fn malformed_typed_stage_name_is_never_ready_even_with_valid_contents() {
    let fixture = Fixture::new();
    fixture.fixture();
    let result = with_fault(
        &[
            Path::new("backup"),
            &fixture.at("source"),
            &fixture.at("snapshot"),
        ],
        "backup_after_verify",
    );
    assert_eq!(result.status.code(), Some(86));
    let original = fixture.stages()[0].path.clone();
    let malformed = fixture.at(".sfs~B~snapshot~bad-pid~123~0.part");
    fs::rename(&original, &malformed).unwrap();
    assert_eq!(fixture.stages()[0].status, StageStatus::Unsafe);
    assert!(ops::verify(&malformed).is_ok());
    assert!(ops::recover(&malformed, &fixture.at("snapshot")).is_err());
    assert!(malformed.exists());
}

#[test]
fn stages_does_not_promote_arbitrary_files() {
    let fixture = Fixture::new();
    fs::write(
        fixture.at(".sfs-snapshot-100-200-0.part"),
        b"not-a-directory",
    )
    .unwrap();
    let found = fixture.stages();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].status, StageStatus::Unsafe);
    assert!(fs::read(&found[0].path).is_ok(), "仅检查，不删除或更改内容");
}
