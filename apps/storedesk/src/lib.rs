//! Local inventory and transaction teaching application. Always binds user data as parameters.
pub mod sqlite;
use sqlite::{Db, Param, Result};
use std::{fs, path::Path};

pub const MAX_STOCK: i64 = 1_000_000_000;
const APPLICATION_ID: i64 = 0x5354444b; // STDK; never adopt an unrelated database.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub sku: String,
    pub name: String,
    pub stock: i64,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    pub sku: String,
    pub delta: i64,
}

/// Stage 1: validation must happen before the first SQL write.
pub fn validate_item(item: &Item) -> Result<()> {
    if item.sku.is_empty()
        || item.sku.len() > 32
        || !item
            .sku
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c))
    {
        return Err("SKU 应为 1–32 位 ASCII 字母、数字、-、_ 或 .".into());
    }
    if item.name.trim().is_empty()
        || item.name.chars().count() > 80
        || item.name.chars().any(char::is_control)
    {
        return Err("名称应有 1–80 个字符，不含控制字符".into());
    }
    if !(0..=MAX_STOCK).contains(&item.stock) {
        return Err("库存应在 0..=1000000000".into());
    }
    Ok(())
}

/// Stage 2: INSERT with parameters. A duplicate SKU must fail, not replace previous stock.
pub fn save_item(db: &Db, item: &Item) -> Result<()> {
    validate_item(item)?;
    db.execute(
        "INSERT INTO items(sku,name,stock) VALUES (?1,?2,?3)",
        &[
            Param::Text(&item.sku),
            Param::Text(&item.name),
            Param::Int(item.stock),
        ],
    )?;
    Ok(())
}

/// Stage 3: updates and their audit rows form ONE all-or-nothing transaction.
pub fn apply_changes(db: &mut Db, changes: &[Change]) -> Result<()> {
    if changes.is_empty() || changes.len() > 1000 {
        return Err("一次需要 1–1000 条库存变化".into());
    }
    for change in changes {
        if change.delta == 0 || !(-MAX_STOCK..=MAX_STOCK).contains(&change.delta) {
            return Err("变化量不能为零或超过正负 1000000000".into());
        }
        validate_item(&Item {
            sku: change.sku.clone(),
            name: "validation".into(),
            stock: 0,
        })?;
    }
    db.transaction(|tx| {
        for change in changes {
            let affected = tx.execute("UPDATE items SET stock=stock+?1 WHERE sku=?2 AND stock+?1 BETWEEN 0 AND 1000000000", &[Param::Int(change.delta), Param::Text(&change.sku)])?;
            if affected != 1 { return Err(format!("{} 不存在，或调整会导致负库存/超过上限；本批次全部撤销", change.sku)); }
            tx.execute("INSERT INTO movements(sku,delta) VALUES (?1,?2)", &[Param::Text(&change.sku), Param::Int(change.delta)])?;
        }
        Ok(())
    })
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
        if !meta.is_file() || meta.file_type().is_symlink() {
            return Err("数据库路径必须是非链接普通文件".into());
        }
    }
    let mut db = Db::open(path)?;
    initialize(&mut db)?;
    Ok(db)
}
pub fn list_items(db: &Db) -> Result<Vec<Item>> {
    db.query("SELECT sku,name,stock FROM items ORDER BY sku", &[])?
        .into_iter()
        .map(|row| {
            let item = Item {
                sku: row[0].text()?.into(),
                name: row[1].text()?.into(),
                stock: row[2].int()?,
            };
            validate_item(&item)?;
            Ok(item)
        })
        .collect()
}
pub fn history(db: &Db) -> Result<Vec<(i64, String, i64)>> {
    db.query("SELECT id,sku,delta FROM movements ORDER BY id", &[])?
        .into_iter()
        .map(|row| Ok((row[0].int()?, row[1].text()?.into(), row[2].int()?)))
        .collect()
}
pub fn parse_changes(text: &str) -> Result<Vec<Change>> {
    let mut changes = Vec::new();
    for (line, row) in text.lines().enumerate() {
        let fields: Vec<_> = row.split('\t').collect();
        if fields.len() != 2 {
            return Err(format!("第 {} 行需要 SKU<Tab>整数变化量", line + 1));
        }
        changes.push(Change {
            sku: fields[0].into(),
            delta: fields[1]
                .parse()
                .map_err(|_| format!("第 {} 行变化量不是整数", line + 1))?,
        });
    }
    Ok(changes)
}
