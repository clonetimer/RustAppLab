use minikv::{
    journal::{Journal, MemoryJournal},
    *,
};
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::{Arc, Mutex},
};
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "minikv-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn put(key: &str, value: &str) -> Operation {
    Operation::Put {
        key: key.into(),
        value: value.into(),
    }
}
fn cli(args: &[&std::ffi::OsStr]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_minikv"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn step1_accepts_real_keys_and_exact_byte_boundary() {
    for key in ["course/next", "书签/Rust", "contains space", "x", "😀"] {
        validate_key(key).unwrap();
    }
    validate_key(&"x".repeat(128)).unwrap();
    validate_key(&"好".repeat(42)).unwrap();
}
#[test]
fn step1_rejects_without_normalizing() {
    for key in [
        "",
        " ",
        " leading",
        "trailing ",
        "line\nkey",
        "a\tb",
        "\u{1b}[2J",
        "\0",
    ] {
        assert!(
            matches!(validate_key(key), Err(KvError::InvalidKey(_))),
            "{key:?}"
        );
    }
    assert!(validate_key(&"x".repeat(129)).is_err());
    assert!(validate_key(&"好".repeat(43)).is_err());
}
#[test]
fn step2_write_read_replace_remove_and_missing() {
    let mut db = Database::open(MemoryJournal::default()).unwrap();
    assert_eq!(
        db.execute(put("book/rust", "借用")).unwrap(),
        Outcome::Stored { replaced: false }
    );
    assert_eq!(
        db.execute(put("book/rust", "迭代器")).unwrap(),
        Outcome::Stored { replaced: true }
    );
    assert_eq!(
        db.execute(Operation::Get {
            key: "book/rust".into()
        })
        .unwrap(),
        Outcome::Value("迭代器".into())
    );
    assert_eq!(db.len(), 1);
    assert_eq!(
        db.execute(Operation::Delete {
            key: "book/rust".into()
        })
        .unwrap(),
        Outcome::Removed
    );
    assert!(matches!(
        db.execute(Operation::Get {
            key: "book/rust".into()
        }),
        Err(KvError::Missing(_))
    ));
    assert!(matches!(
        db.execute(Operation::Delete {
            key: "unknown".into()
        }),
        Err(KvError::Missing(_))
    ));
    assert!(db.is_empty());
}
#[derive(Clone, Default)]
struct Recorder(Arc<Mutex<Vec<Operation>>>);
impl Journal for Recorder {
    fn read(&mut self) -> Result<Vec<Operation>, KvError> {
        Ok(self.0.lock().unwrap().clone())
    }
    fn append(&mut self, op: &Operation) -> Result<(), KvError> {
        self.0.lock().unwrap().push(op.clone());
        Ok(())
    }
}
#[test]
fn step2_append_order_readonly_and_invalid_data() {
    let journal = Recorder::default();
    let history = journal.0.clone();
    let mut db = Database::open(journal).unwrap();
    db.execute(put("a", "1")).unwrap();
    db.execute(Operation::Get { key: "a".into() }).unwrap();
    assert!(db.execute(put("bad\nkey", "2")).is_err());
    assert!(db
        .execute(put("a", &"x".repeat(MAX_VALUE_BYTES + 1)))
        .is_err());
    assert!(db
        .execute(Operation::Delete {
            key: "missing".into()
        })
        .is_err());
    assert_eq!(*history.lock().unwrap(), vec![put("a", "1")]);
    let mut failing = Database::open(MemoryJournal {
        operations: vec![put("a", "old")],
        fail_next: true,
    })
    .unwrap();
    assert!(failing.execute(put("a", "new")).is_err());
    assert_eq!(
        failing.execute(Operation::Get { key: "a".into() }).unwrap(),
        Outcome::Value("old".into())
    );
}
#[test]
fn step3_sorted_borrowed_prefix_and_fused_end() {
    let mut db = Database::open(MemoryJournal::default()).unwrap();
    for (key, value) in [
        ("z", "last"),
        ("book/b", "B"),
        ("book/a", "A"),
        ("bookmark", "M"),
        ("中文/a", "中"),
    ] {
        db.execute(put(key, value)).unwrap();
    }
    let mut iter = db.scan("book/");
    assert_eq!(iter.next(), Some(("book/a", "A")));
    assert_eq!(iter.next(), Some(("book/b", "B")));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), None);
    assert_eq!(db.scan("中文/").collect::<Vec<_>>(), vec![("中文/a", "中")]);
    assert_eq!(db.scan("absent/").next(), None);
}
#[test]
fn step3_composes_with_take_filter_and_empty_prefix() {
    let mut db = Database::open(MemoryJournal::default()).unwrap();
    for n in 0..4 {
        db.execute(put(&format!("note/{n}"), &n.to_string()))
            .unwrap();
    }
    assert_eq!(db.scan("").count(), 4);
    let first = db.scan("note/").take(2).map(|(k, _)| k).collect::<Vec<_>>();
    assert_eq!(first, vec!["note/0", "note/1"]);
    assert_eq!(db.scan("").filter(|(_, v)| *v == "2").count(), 1);
    let empty = Database::open(MemoryJournal::default()).unwrap();
    assert!(empty.scan("").next().is_none());
}
#[test]
fn step3_cli_cross_process_persistence_unicode_and_export() {
    let t = Temp::new();
    let db = t.0.join("我的 书签.kv");
    let out = t.0.join("作品.tsv");
    for (key, value) in [("course/next", "自己的程序"), ("book/rust", "中文与空格")] {
        let result = cli(&["put".as_ref(), db.as_os_str(), key.as_ref(), value.as_ref()]);
        assert!(result.status.success(), "{:?}", result);
    }
    let result = cli(&["get".as_ref(), db.as_os_str(), "course/next".as_ref()]);
    assert!(result.status.success());
    assert!(String::from_utf8_lossy(&result.stdout).contains("自己的程序"));
    let result = cli(&["scan".as_ref(), db.as_os_str(), "book/".as_ref()]);
    assert!(result.status.success());
    assert!(String::from_utf8_lossy(&result.stdout).contains("book/rust\t中文与空格"));
    assert!(cli(&["export".as_ref(), db.as_os_str(), out.as_os_str()])
        .status
        .success());
    assert!(fs::read_to_string(out)
        .unwrap()
        .contains("course/next\t自己的程序"));
}
#[test]
fn step3_cli_rejects_missing_and_never_clobbers_output() {
    let t = Temp::new();
    let db = t.0.join("store.kv");
    let report = t.0.join("keep.tsv");
    assert!(!cli(&["get".as_ref(), db.as_os_str(), "unknown".as_ref()])
        .status
        .success());
    assert!(!db.exists());
    assert!(!cli(&[
        "put".as_ref(),
        db.as_os_str(),
        " bad".as_ref(),
        "value".as_ref()
    ])
    .status
    .success());
    assert!(!db.exists());
    assert!(cli(&[
        "put".as_ref(),
        db.as_os_str(),
        "key".as_ref(),
        "data".as_ref()
    ])
    .status
    .success());
    fs::write(&report, "keep mine").unwrap();
    assert!(
        !cli(&["export".as_ref(), db.as_os_str(), report.as_os_str()])
            .status
            .success()
    );
    assert_eq!(fs::read_to_string(report).unwrap(), "keep mine");
}
