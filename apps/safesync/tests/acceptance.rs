use safesync::ops::{self, BackupStatus};
use safesync::sha256::hex;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let base = std::env::temp_dir().join(format!(
            "sfs-acceptance-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&base).unwrap();
        Self(base)
    }
    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn source(root: &Path) {
    fs::create_dir(root).unwrap();
    fs::create_dir_all(root.join("中文/空子目录")).unwrap();
    fs::create_dir_all(root.join("logs")).unwrap();
    fs::write(root.join("README.txt"), "一个可恢复的项目\n").unwrap();
    fs::write(root.join("中文/带 空格.txt"), [0, 0, 1, 2, 253, 254, 255]).unwrap();
    fs::write(root.join("logs/run.log"), "INFO OK\nWARN timeout\n").unwrap();
}

#[test]
fn real_backup_verify_restore_preserves_bytes_and_empty_dirs() {
    let t = Fixture::new();
    source(&t.path("source"));
    let (status, stats) = ops::backup(&t.path("source"), &t.path("snapshot")).unwrap();
    assert_eq!(status, BackupStatus::Created);
    assert_eq!(stats.files, 3);
    assert_eq!(stats.directories, 3);
    assert_eq!(ops::verify(&t.path("snapshot")).unwrap(), stats);
    assert!(t.path("snapshot/manifest.sfs").is_file());
    assert!(t.path("snapshot/data/中文/空子目录").is_dir());
    fs::remove_dir_all(t.path("source")).unwrap();
    assert_eq!(
        ops::restore(&t.path("snapshot"), &t.path("restored")).unwrap(),
        stats
    );
    assert_eq!(
        fs::read(t.path("restored/中文/带 空格.txt")).unwrap(),
        [0, 0, 1, 2, 253, 254, 255]
    );
    assert!(t.path("restored/中文/空子目录").is_dir());
}

#[test]
fn repeat_backup_is_idempotent_but_source_change_does_not_overwrite_snapshot() {
    let t = Fixture::new();
    source(&t.path("src"));
    ops::backup(&t.path("src"), &t.path("bak")).unwrap();
    assert_eq!(
        ops::backup(&t.path("src"), &t.path("bak")).unwrap().0,
        BackupStatus::Unchanged
    );
    fs::write(t.path("src/README.txt"), "新版").unwrap();
    assert!(ops::backup(&t.path("src"), &t.path("bak")).is_err());
    assert!(ops::verify(&t.path("bak")).is_ok());
    assert_eq!(
        fs::read_to_string(t.path("bak/data/README.txt")).unwrap(),
        "一个可恢复的项目\n"
    );
}

#[test]
fn same_length_bit_flip_is_detected_and_restore_never_starts() {
    let t = Fixture::new();
    source(&t.path("src"));
    ops::backup(&t.path("src"), &t.path("bak")).unwrap();
    let p = t.path("bak/data/中文/带 空格.txt");
    let mut contents = fs::read(&p).unwrap();
    contents[3] ^= 0x01;
    fs::write(&p, contents).unwrap();
    assert!(ops::verify(&t.path("bak")).is_err());
    assert!(ops::restore(&t.path("bak"), &t.path("restored")).is_err());
    assert!(!t.path("restored").exists());
    assert!(!fs::read_dir(&t.0).unwrap().any(|e| e
        .unwrap()
        .file_name()
        .to_string_lossy()
        .starts_with(".sfs-")));
}

#[test]
fn reject_extra_data_files_and_tampered_manifest() {
    let t = Fixture::new();
    source(&t.path("src"));
    ops::backup(&t.path("src"), &t.path("bak")).unwrap();
    fs::write(t.path("bak/data/rogue.txt"), "extra").unwrap();
    assert!(ops::verify(&t.path("bak")).is_err());
    fs::remove_file(t.path("bak/data/rogue.txt")).unwrap();
    fs::write(
        t.path("bak/manifest.sfs"),
        format!("SAFESYNC\t1\nD\t{}\n", hex(b"../../escape")),
    )
    .unwrap();
    assert!(ops::verify(&t.path("bak")).is_err());
    assert!(ops::restore(&t.path("bak"), &t.path("outside")).is_err());
    assert!(!t.path("outside").exists());
}

#[test]
fn never_overwrite_restore_target_or_allow_backup_within_source() {
    let t = Fixture::new();
    source(&t.path("src"));
    assert!(ops::backup(&t.path("src"), &t.path("src/internal")).is_err());
    assert!(!t.path("src/internal").exists());
    ops::backup(&t.path("src"), &t.path("bak")).unwrap();
    fs::create_dir(t.path("restored")).unwrap();
    fs::write(t.path("restored/critical.txt"), "do not overwrite").unwrap();
    assert!(ops::restore(&t.path("bak"), &t.path("restored")).is_err());
    assert_eq!(
        fs::read_to_string(t.path("restored/critical.txt")).unwrap(),
        "do not overwrite"
    );
    assert!(ops::restore(&t.path("bak"), &t.path("bak/data/inside")).is_err());
    assert!(!t.path("bak/data/inside").exists());
    assert!(ops::verify(&t.path("bak")).is_ok());
}

#[test]
fn blank_source_and_hidden_names_are_valid() {
    let t = Fixture::new();
    fs::create_dir(t.path("empty")).unwrap();
    assert_eq!(
        ops::backup(&t.path("empty"), &t.path("bak"))
            .unwrap()
            .1
            .files,
        0
    );
    assert!(
        ops::restore(&t.path("bak"), &t.path("restored"))
            .unwrap()
            .directories
            == 0
    );
    fs::create_dir(t.path("hidden")).unwrap();
    fs::write(t.path("hidden/.env"), b"DATA=not-a-real-secret").unwrap();
    ops::backup(&t.path("hidden"), &t.path("hidden_bak")).unwrap();
    assert!(ops::verify(&t.path("hidden_bak")).is_ok());
}

#[cfg(unix)]
#[test]
fn symlinks_are_refused_at_source_and_backup_and_target() {
    use std::os::unix::fs::symlink;
    let t = Fixture::new();
    source(&t.path("src"));
    symlink(t.path("src/README.txt"), t.path("src/follow_me")).unwrap();
    assert!(ops::backup(&t.path("src"), &t.path("bak")).is_err());
    assert!(!t.path("bak").exists());
    fs::remove_file(t.path("src/follow_me")).unwrap();
    ops::backup(&t.path("src"), &t.path("bak")).unwrap();
    symlink(t.path("src/README.txt"), t.path("bak/data/alias")).unwrap();
    assert!(ops::verify(&t.path("bak")).is_err());
    fs::remove_file(t.path("bak/data/alias")).unwrap();
    symlink(t.path("src"), t.path("restored")).unwrap();
    assert!(ops::restore(&t.path("bak"), &t.path("restored")).is_err());
}

#[cfg(unix)]
#[test]
fn non_utf8_names_are_rejected_explicitly() {
    use std::os::unix::ffi::OsStringExt;
    let t = Fixture::new();
    fs::create_dir(t.path("src")).unwrap();
    let filename = std::ffi::OsString::from_vec(vec![0xff, 0xfe]);
    fs::write(t.path("src").join(filename), b"hi").unwrap();
    assert!(ops::backup(&t.path("src"), &t.path("bak")).is_err());
    assert!(!t.path("bak").exists());
}

#[cfg(unix)]
#[test]
fn special_fifo_file_is_refused() {
    // Unix mkfifo 不依赖第三方 crate；测试只校验特殊文件会被拒绝，不进行读取。
    use std::process::Command;
    let t = Fixture::new();
    fs::create_dir(t.path("src")).unwrap();
    if Command::new("mkfifo")
        .arg(t.path("src/pipe"))
        .status()
        .is_ok_and(|s| s.success())
    {
        assert!(ops::backup(&t.path("src"), &t.path("bak")).is_err());
    }
}

#[test]
fn backup_target_requires_existing_parent_and_no_accidental_files() {
    let t = Fixture::new();
    source(&t.path("src"));
    assert!(ops::backup(&t.path("src"), &t.path("missing-parent/bak")).is_err());
    assert!(!t.path("missing-parent").exists());
    fs::write(t.path("bak"), "existing file").unwrap();
    assert!(ops::backup(&t.path("src"), &t.path("bak")).is_err());
    assert_eq!(fs::read_to_string(t.path("bak")).unwrap(), "existing file");
}
