use minikv::{
    journal::{FileJournal, Journal, MemoryJournal},
    *,
};
use std::{fs, io::Write, path::PathBuf};
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "minikv-behavior-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
    fn file(&self) -> PathBuf {
        self.0.join("journal.kv")
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn put(k: &str, v: &str) -> Operation {
    Operation::Put {
        key: k.into(),
        value: v.into(),
    }
}
fn open(p: &std::path::Path) -> Result<Database<FileJournal>, KvError> {
    Database::open(FileJournal::open(p)?)
}
#[test]
fn multiline_empty_values_and_replay_tombstones() {
    let t = Temp::new();
    let p = t.file();
    {
        let mut db = open(&p).unwrap();
        db.execute(put("note/a", "你好\n第二行\t\0")).unwrap();
        db.execute(put("empty", "")).unwrap();
        db.execute(put("gone", "x")).unwrap();
        db.execute(Operation::Delete { key: "gone".into() })
            .unwrap();
    }
    let db = open(&p).unwrap();
    assert_eq!(
        db.scan("").collect::<Vec<_>>(),
        vec![("empty", ""), ("note/a", "你好\n第二行\t\0")]
    );
}
#[test]
fn lock_excludes_second_open_and_normal_drop_releases() {
    let t = Temp::new();
    let p = t.file();
    let first = open(&p).unwrap();
    assert!(matches!(open(&p), Err(KvError::Busy(_))));
    drop(first);
    assert!(open(&p).is_ok());
}
#[test]
fn stale_lock_is_not_removed_automatically() {
    let t = Temp::new();
    let p = t.file();
    let lock = t.0.join("journal.kv.lock");
    fs::write(&lock, "unknown owner").unwrap();
    assert!(matches!(open(&p), Err(KvError::Busy(_))));
    assert_eq!(fs::read_to_string(lock).unwrap(), "unknown owner");
    assert!(!p.exists());
}
#[test]
fn wrong_header_and_truncated_tail_preserve_bytes() {
    for bytes in ["not my database\n", "RUSTAPPLAB_MINIKV_V1\nP\t61\t62"] {
        let t = Temp::new();
        let p = t.file();
        fs::write(&p, bytes).unwrap();
        assert!(matches!(open(&p), Err(KvError::Corrupt { .. })));
        assert_eq!(fs::read_to_string(&p).unwrap(), bytes);
        assert!(!t.0.join("journal.kv.lock").exists());
    }
}
#[test]
fn malformed_hex_utf8_and_replay_delete_are_rejected() {
    for line in [
        "P\t61\tff\n",
        "P\txx\t62\n",
        "P\t6\t62\n",
        "D\t61\n",
        "G\t61\n",
        "P\t\t62\n",
        "\n",
    ] {
        let t = Temp::new();
        let p = t.file();
        let text = format!("RUSTAPPLAB_MINIKV_V1\n{line}");
        fs::write(&p, &text).unwrap();
        assert!(open(&p).is_err(), "{line:?}");
        assert_eq!(fs::read_to_string(p).unwrap(), text);
    }
}
#[test]
fn rejected_write_keeps_original_file_bytes() {
    let t = Temp::new();
    let p = t.file();
    let mut db = open(&p).unwrap();
    db.execute(put("a", "1")).unwrap();
    let before = fs::read(&p).unwrap();
    assert!(db.execute(put(" leading", "2")).is_err());
    assert!(db
        .execute(Operation::Delete { key: "none".into() })
        .is_err());
    assert_eq!(fs::read(p).unwrap(), before);
}
#[test]
fn typed_error_retains_io_source() {
    use std::error::Error;
    let error = KvError::from(std::io::Error::new(
        std::io::ErrorKind::PermissionDenied,
        "denied",
    ));
    assert_eq!(error.kind(), "io");
    assert!(error.source().is_some());
    assert!(KvError::ValueTooLarge.source().is_none());
}
#[test]
fn boxed_journal_uses_same_database_api() {
    let journal: Box<dyn Journal> = Box::<MemoryJournal>::default();
    let mut db = Database::open(journal).unwrap();
    db.execute(put("a", "b")).unwrap();
    assert_eq!(db.scan("").next(), Some(("a", "b")));
}
#[test]
fn shared_threads_do_not_lose_updates() {
    let db = share(Database::open(MemoryJournal::default()).unwrap());
    let workers = (0..4)
        .map(|n| {
            let db = std::sync::Arc::clone(&db);
            std::thread::spawn(move || {
                for i in 0..30 {
                    db.lock()
                        .unwrap()
                        .execute(put(&format!("worker/{n}/{i}"), "ok"))
                        .unwrap();
                }
            })
        })
        .collect::<Vec<_>>();
    for worker in workers {
        worker.join().unwrap();
    }
    assert_eq!(db.lock().unwrap().len(), 120);
}
#[test]
fn journal_capacity_limit_is_not_silent_truncation() {
    let t = Temp::new();
    let p = t.file();
    let mut f = fs::File::create(&p).unwrap();
    f.write_all(b"RUSTAPPLAB_MINIKV_V1\n").unwrap();
    f.set_len(minikv::journal::MAX_LOG_BYTES + 1).unwrap();
    drop(f);
    assert!(open(&p).is_err());
    assert_eq!(
        fs::metadata(p).unwrap().len(),
        minikv::journal::MAX_LOG_BYTES + 1
    );
}
#[test]
fn missing_parent_does_not_create_directories() {
    let t = Temp::new();
    let p = t.0.join("missing/store.kv");
    assert!(open(&p).is_err());
    assert!(!t.0.join("missing").exists());
}
#[cfg(unix)]
#[test]
fn symlink_database_is_not_followed() {
    use std::os::unix::fs::symlink;
    let t = Temp::new();
    let outside = t.0.join("outside");
    fs::write(&outside, "my personal data").unwrap();
    symlink(&outside, t.file()).unwrap();
    assert!(open(&t.file()).is_err());
    assert_eq!(fs::read_to_string(outside).unwrap(), "my personal data");
}

#[test]
fn existing_empty_file_is_not_silently_initialized() {
    let temp = Temp::new();
    let p = temp.file();
    fs::write(&p, []).unwrap();
    assert!(matches!(open(&p), Err(KvError::Corrupt { .. })));
    assert_eq!(fs::metadata(&p).unwrap().len(), 0);
    assert!(!temp.0.join("journal.kv.lock").exists());
}
