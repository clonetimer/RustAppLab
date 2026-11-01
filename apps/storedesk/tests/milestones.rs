use storedesk::*;

use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};
static COUNT: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let p = std::env::temp_dir().join(format!(
            "storedesk-test-{}-{stamp}-{}",
            std::process::id(),
            COUNT.fetch_add(1, Ordering::Relaxed)
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
fn cli(args: &[&std::ffi::OsStr]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_storedesk"))
        .args(args)
        .output()
        .unwrap()
}

fn item(sku: &str, stock: i64) -> Item {
    Item {
        sku: sku.into(),
        name: "钢笔 ' 与 中文".into(),
        stock,
    }
}
#[test]
fn step1_valid_items_and_boundaries() {
    for n in [0, 1, MAX_STOCK] {
        assert!(validate_item(&item("PEN_01-A", n)).is_ok());
    }
    let mut i = item("X", 1);
    i.name = "'); DROP TABLE items; --".into();
    assert!(
        validate_item(&i).is_ok(),
        "数据可以含引号，安全性由参数绑定保证"
    );
}
#[test]
fn step1_bad_items() {
    for sku in ["", "a b", "中文", "A'", "A\tB"] {
        assert!(validate_item(&item(sku, 1)).is_err());
    }
    for n in [-1, MAX_STOCK + 1, i64::MAX] {
        assert!(validate_item(&item("A", n)).is_err());
    }
    for name in ["", " ", "bad\nname", "bad\u{1b}name"] {
        let mut i = item("A", 1);
        i.name = name.into();
        assert!(validate_item(&i).is_err());
    }
    let mut i = item("A", 1);
    i.name = "字".repeat(81);
    assert!(validate_item(&i).is_err());
}
#[test]
fn step2_real_sqlite_persistence_and_parameter_binding() {
    let t = Temp::new();
    let path = t.0.join("中文 库存.db");
    let mut i = item("B", 12);
    i.name = "x'); DROP TABLE items; --".into();
    {
        let db = open(&path).unwrap();
        save_item(&db, &i).unwrap();
        save_item(&db, &item("A", 3)).unwrap();
    }
    assert!(fs::read(&path).unwrap().starts_with(b"SQLite format 3\0"));
    let db = open(&path).unwrap();
    let rows = list_items(&db).unwrap();
    assert_eq!(
        rows.iter().map(|i| i.sku.as_str()).collect::<Vec<_>>(),
        vec!["A", "B"]
    );
    assert_eq!(rows[1], i);
}
#[test]
fn step2_duplicate_and_invalid_never_replace() {
    let t = Temp::new();
    let db = open(&t.0.join("data.db")).unwrap();
    save_item(&db, &item("PEN", 5)).unwrap();
    assert!(save_item(&db, &item("PEN", 99)).is_err());
    assert!(save_item(&db, &item("BAD", -1)).is_err());
    assert_eq!(list_items(&db).unwrap(), vec![item("PEN", 5)]);
}
#[test]
fn step3_atomic_commit_with_audit() {
    let t = Temp::new();
    let mut db = open(&t.0.join("data.db")).unwrap();
    save_item(&db, &item("A", 10)).unwrap();
    save_item(&db, &item("B", 2)).unwrap();
    apply_changes(
        &mut db,
        &[
            Change {
                sku: "A".into(),
                delta: -3,
            },
            Change {
                sku: "B".into(),
                delta: 3,
            },
            Change {
                sku: "A".into(),
                delta: 1,
            },
        ],
    )
    .unwrap();
    assert_eq!(
        list_items(&db)
            .unwrap()
            .iter()
            .map(|i| i.stock)
            .collect::<Vec<_>>(),
        vec![8, 5]
    );
    assert_eq!(
        history(&db)
            .unwrap()
            .iter()
            .map(|r| r.2)
            .collect::<Vec<_>>(),
        vec![-3, 3, 1]
    );
}
#[test]
fn step3_failed_batch_rolls_back_updates_and_audit() {
    let t = Temp::new();
    let mut db = open(&t.0.join("data.db")).unwrap();
    save_item(&db, &item("A", 10)).unwrap();
    for changes in [
        vec![
            Change {
                sku: "A".into(),
                delta: -2,
            },
            Change {
                sku: "A".into(),
                delta: -9,
            },
        ],
        vec![
            Change {
                sku: "A".into(),
                delta: 3,
            },
            Change {
                sku: "missing".into(),
                delta: 1,
            },
        ],
        vec![Change {
            sku: "A".into(),
            delta: 0,
        }],
    ] {
        assert!(apply_changes(&mut db, &changes).is_err());
        assert_eq!(list_items(&db).unwrap()[0].stock, 10);
        assert!(history(&db).unwrap().is_empty());
    }
}
#[test]
fn step3_cli_cross_process_persistence() {
    let t = Temp::new();
    let db = t.0.join("练习 数据.db");
    assert!(cli(&["init".as_ref(), db.as_os_str()]).status.success());
    assert!(cli(&[
        "add".as_ref(),
        db.as_os_str(),
        "PEN".as_ref(),
        "我的笔".as_ref(),
        "10".as_ref()
    ])
    .status
    .success());
    assert!(cli(&[
        "adjust".as_ref(),
        db.as_os_str(),
        "PEN".as_ref(),
        "-2".as_ref()
    ])
    .status
    .success());
    let list = cli(&["list".as_ref(), db.as_os_str()]);
    assert!(list.status.success());
    assert!(String::from_utf8_lossy(&list.stdout).contains("PEN\t我的笔\t8"));
    assert!(
        String::from_utf8_lossy(&cli(&["history".as_ref(), db.as_os_str()]).stdout)
            .contains("PEN\t-2")
    );
}
#[test]
fn step3_cli_bad_batch_is_nonzero_and_unchanged() {
    let t = Temp::new();
    let path = t.0.join("inventory.db");
    let file = t.0.join("bad.tsv");
    {
        let db = open(&path).unwrap();
        save_item(&db, &item("PEN", 10)).unwrap();
    }
    fs::write(&file, "PEN\t-2\nPEN\t-99\n").unwrap();
    let out = cli(&["apply".as_ref(), path.as_os_str(), file.as_os_str()]);
    assert!(!out.status.success());
    let db = open(&path).unwrap();
    assert_eq!(list_items(&db).unwrap()[0].stock, 10);
    assert!(history(&db).unwrap().is_empty());
}
