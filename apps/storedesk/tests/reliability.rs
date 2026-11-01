use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use storedesk::{
    self as store,
    sqlite::{self, Cell, Db, Param},
};
static COUNT: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "store-reliability-{}-{}",
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
#[test]
fn sqlite_buffers_and_types() {
    let db = Db::memory().unwrap();
    db.execute("CREATE TABLE notes(n INTEGER,t TEXT)", &[])
        .unwrap();
    let text = "中文 '; DROP TABLE notes; --";
    db.execute(
        "INSERT INTO notes VALUES (?1,?2)",
        &[Param::Int(i64::MAX), Param::Text(text)],
    )
    .unwrap();
    assert_eq!(
        db.query("SELECT n,t,NULL FROM notes", &[]).unwrap(),
        vec![vec![
            Cell::Int(i64::MAX),
            Cell::Text(text.into()),
            Cell::Null
        ]]
    );
    let mut long = String::from("x");
    long.push_str(&"字".repeat(10000));
    assert_eq!(
        db.query(
            "SELECT ?1,?2,?3",
            &[Param::Text(&long), Param::Text(text), Param::Text("")]
        )
        .unwrap()[0][0],
        Cell::Text(long)
    );
    assert!(!sqlite::version().is_empty());
}
#[test]
fn sqlite_bad_sql_and_parameters_are_errors() {
    let db = Db::memory().unwrap();
    assert!(db
        .execute("CREATE TABLE a(x); CREATE TABLE b(y)", &[])
        .is_err());
    assert!(db
        .query("SELECT name FROM sqlite_master WHERE name='a'", &[])
        .unwrap()
        .is_empty());
    assert!(db.execute("bad SQL", &[]).is_err());
    assert!(db.execute("-- comment", &[]).is_err());
    assert!(db.query("SELECT ?1", &[]).is_err());
    assert!(db.query("SELECT 1", &[Param::Int(1)]).is_err());
    assert!(db.query("SELECT ?1", &[Param::Text("a\0b")]).is_err());
    assert!(db.query("SELECT 3.5", &[]).is_err());
    assert!(db.execute("SELECT 1", &[]).is_err());
    assert!(db.query("CREATE TABLE bad(x)", &[]).is_err());
}
#[test]
fn rollback_on_error_and_unwind() {
    let mut db = Db::memory().unwrap();
    db.execute("CREATE TABLE values_(n INTEGER)", &[]).unwrap();
    let result: Result<(), String> = db.transaction(|tx| {
        tx.execute("INSERT INTO values_ VALUES (1)", &[])?;
        Err("test rollback".into())
    });
    assert!(result.is_err());
    assert!(db.query("SELECT n FROM values_", &[]).unwrap().is_empty());
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _: Result<(), String> = db.transaction(|tx| {
            tx.execute("INSERT INTO values_ VALUES (2)", &[])?;
            panic!("forced teaching unwind");
        });
    }));
    assert!(panic.is_err());
    assert!(db.query("SELECT n FROM values_", &[]).unwrap().is_empty());
    db.transaction(|tx| tx.execute("INSERT INTO values_ VALUES (3)", &[]))
        .unwrap();
    assert_eq!(
        db.query("SELECT n FROM values_", &[]).unwrap()[0][0],
        Cell::Int(3)
    );
}
#[test]
fn unrelated_database_is_not_adopted() {
    let t = Temp::new();
    let path = t.0.join("other.db");
    {
        let db = Db::open(&path).unwrap();
        db.execute("CREATE TABLE private_data(x TEXT)", &[])
            .unwrap();
        db.execute("INSERT INTO private_data VALUES ('keep me')", &[])
            .unwrap();
    }
    assert!(store::open(&path).is_err());
    let db = Db::open(&path).unwrap();
    assert_eq!(
        db.query("SELECT x FROM private_data", &[]).unwrap()[0][0],
        Cell::Text("keep me".into())
    );
    assert_eq!(
        db.query("PRAGMA application_id", &[]).unwrap()[0][0],
        Cell::Int(0)
    );
}
#[test]
fn newer_schema_is_rejected_without_downgrade() {
    let t = Temp::new();
    let path = t.0.join("new.db");
    {
        let db = store::open(&path).unwrap();
        db.execute("PRAGMA user_version=2", &[]).unwrap();
    }
    assert!(store::open(&path).is_err());
    let db = Db::open(&path).unwrap();
    assert_eq!(
        db.query("PRAGMA user_version", &[]).unwrap()[0][0],
        Cell::Int(2)
    );
}
#[test]
fn invalid_database_bytes_are_preserved() {
    let t = Temp::new();
    let path = t.0.join("not-db.txt");
    fs::write(&path, "do not destroy 中文").unwrap();
    let before = fs::read(&path).unwrap();
    assert!(store::open(&path).is_err());
    assert_eq!(fs::read(path).unwrap(), before);
}
#[test]
fn changes_validation_and_caps() {
    let mut db = Db::memory().unwrap();
    store::initialize(&mut db).unwrap();
    store::save_item(
        &db,
        &store::Item {
            sku: "A".into(),
            name: "a".into(),
            stock: store::MAX_STOCK,
        },
    )
    .unwrap();
    for changes in [
        vec![],
        vec![store::Change {
            sku: "A".into(),
            delta: i64::MIN,
        }],
        vec![store::Change {
            sku: "A".into(),
            delta: 1,
        }],
    ] {
        assert!(store::apply_changes(&mut db, &changes).is_err());
    }
    assert!(store::parse_changes("A\twrong").is_err());
    assert!(store::parse_changes("A\t1\tmore").is_err());
    assert!(store::history(&db).unwrap().is_empty());
}
#[test]
fn two_connections_observe_committed_values() {
    let t = Temp::new();
    let path = t.0.join("shared.db");
    let mut first = store::open(&path).unwrap();
    let second = store::open(&path).unwrap();
    store::save_item(
        &first,
        &store::Item {
            sku: "A".into(),
            name: "book".into(),
            stock: 2,
        },
    )
    .unwrap();
    store::apply_changes(
        &mut first,
        &[store::Change {
            sku: "A".into(),
            delta: -1,
        }],
    )
    .unwrap();
    assert_eq!(store::list_items(&second).unwrap()[0].stock, 1);
}
#[cfg(unix)]
#[test]
fn symbolic_link_database_is_rejected() {
    let t = Temp::new();
    let original = t.0.join("db");
    let _db = store::open(&original).unwrap();
    let link = t.0.join("link");
    std::os::unix::fs::symlink(original, &link).unwrap();
    assert!(store::open(&link).is_err());
}
