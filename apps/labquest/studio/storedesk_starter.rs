//! Local inventory and transaction teaching application. Always binds user data as parameters.
pub mod sqlite;
use sqlite::{Db, Result};
// 第 2 步需要时增加：use sqlite::Param;
use std::{fs, path::Path};

pub const MAX_STOCK: i64 = 1_000_000_000;
const APPLICATION_ID: i64 = 0x5354444b; // STDK; never adopt an unrelated database.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item { pub sku: String, pub name: String, pub stock: i64 }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change { pub sku: String, pub delta: i64 }

/// Stage 1: validation must happen before the first SQL write.
pub fn validate_item(item: &Item) -> Result<()> {
    // TODO(1): 按 LESSON.md 第 1 步完成；其它辅助代码暂时不改。
    // LESSON 1 BEGIN
    let _ = item;
    Err("尚未完成物品校验".into())
    // LESSON 1 END
}

/// Stage 2: INSERT with parameters. A duplicate SKU must fail, not replace previous stock.
pub fn save_item(db: &Db, item: &Item) -> Result<()> {
    // TODO(2): 按 LESSON.md 第 2 步完成；其它辅助代码暂时不改。
    // LESSON 2 BEGIN
    let _ = (db, item);
    Err("尚未完成参数化写入".into())
    // LESSON 2 END
}

/// Stage 3: updates and their audit rows form ONE all-or-nothing transaction.
pub fn apply_changes(db: &mut Db, changes: &[Change]) -> Result<()> {
    // TODO(3): 按 LESSON.md 第 3 步完成；其它辅助代码暂时不改。
    // LESSON 3 BEGIN
    let _ = (db, changes);
    Err("尚未完成批量事务".into())
    // LESSON 3 END
}

/// Supplied schema setup. Schema v1 only; a newer version is rejected, not overwritten.
pub fn initialize(db: &mut Db) -> Result<()> {
    db.execute("PRAGMA foreign_keys=ON", &[])?;
    db.transaction(|tx| {
        let app = tx.query("PRAGMA application_id", &[])?[0][0].int()?;
        let version = tx.query("PRAGMA user_version", &[])?[0][0].int()?;
        let tables = tx.query("SELECT name FROM sqlite_master WHERE name NOT LIKE 'sqlite_%'", &[])?;
        if app == APPLICATION_ID && version == 1 {
            tx.query("SELECT sku,name,stock FROM items LIMIT 0", &[])?;
            tx.query("SELECT id,sku,delta FROM movements LIMIT 0", &[])?;
            return Ok(());
        }
        if app != 0 || version != 0 || !tables.is_empty() { return Err("这不是 StoreDesk v1 数据库，或版本较新；不会自动改写".into()); }
        tx.execute("CREATE TABLE items(sku TEXT PRIMARY KEY NOT NULL,name TEXT NOT NULL,stock INTEGER NOT NULL CHECK(typeof(stock)='integer' AND stock BETWEEN 0 AND 1000000000))", &[])?;
        tx.execute("CREATE TABLE movements(id INTEGER PRIMARY KEY,sku TEXT NOT NULL REFERENCES items(sku),delta INTEGER NOT NULL CHECK(typeof(delta)='integer' AND delta<>0))", &[])?;
        tx.execute("PRAGMA application_id=1398031435", &[])?;
        tx.execute("PRAGMA user_version=1", &[])?;
        Ok(())
    })
}
pub fn open(path: &Path) -> Result<Db> {
    if let Ok(meta) = fs::symlink_metadata(path) {
        if !meta.is_file() || meta.file_type().is_symlink() { return Err("数据库路径必须是非链接普通文件".into()); }
    }
    let mut db = Db::open(path)?;
    initialize(&mut db)?;
    Ok(db)
}
pub fn list_items(db: &Db) -> Result<Vec<Item>> {
    db.query("SELECT sku,name,stock FROM items ORDER BY sku", &[])?.into_iter().map(|row| {
        let item = Item { sku: row[0].text()?.into(), name: row[1].text()?.into(), stock: row[2].int()? };
        validate_item(&item)?;
        Ok(item)
    }).collect()
}
pub fn history(db: &Db) -> Result<Vec<(i64, String, i64)>> {
    db.query("SELECT id,sku,delta FROM movements ORDER BY id", &[])?.into_iter().map(|row| Ok((row[0].int()?, row[1].text()?.into(), row[2].int()?))).collect()
}
pub fn parse_changes(text: &str) -> Result<Vec<Change>> {
    let mut changes = Vec::new();
    for (line, row) in text.lines().enumerate() {
        let fields: Vec<_> = row.split('\t').collect();
        if fields.len() != 2 { return Err(format!("第 {} 行需要 SKU<Tab>整数变化量", line + 1)); }
        changes.push(Change { sku: fields[0].into(), delta: fields[1].parse().map_err(|_| format!("第 {} 行变化量不是整数", line + 1))? });
    }
    Ok(changes)
}
