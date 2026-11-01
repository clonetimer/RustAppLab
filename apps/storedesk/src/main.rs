use std::{env, ffi::OsString, fs, path::Path};
use storedesk::{
    apply_changes, history, list_items, open, parse_changes, save_item, sqlite, Change, Item,
};
const HELP: &str = "StoreDesk · SQLite 物品库存台\ninit <新数据库文件>\nadd <数据库> <SKU> <名称> <初始库存>\nlist <数据库>\nadjust <数据库> <SKU> <变化量>\napply <数据库> <变化.tsv>    两列：SKU<Tab>变化量，整批事务\nhistory <数据库>\ndemo <新目录>\ndoctor    显示实际 SQLite 引擎\nhelp\n不会覆盖未知数据库；单次教学查询最多 10000 行。";
fn text(arg: &OsString) -> Result<&str, String> {
    arg.to_str().ok_or_else(|| "参数需要 UTF-8 文本".into())
}
fn existing(path: &Path) -> Result<sqlite::Db, String> {
    if !path.is_file() {
        return Err("数据库不存在；先 init 新文件。路径拼错不会创建新库。".into());
    }
    open(path)
}
fn run() -> Result<(), String> {
    let a: Vec<_> = env::args_os().skip(1).collect();
    match a.first().and_then(|s| s.to_str()).unwrap_or("help") {
        "help" | "--help" if a.len() <= 1 => println!("{HELP}"),
        "doctor" if a.len() == 1 => println!(
            "SQLite={} backend={} schema=1",
            sqlite::version(),
            if cfg!(windows) {
                "winsqlite3"
            } else {
                "system-sqlite3"
            }
        ),
        "init" if a.len() == 2 => {
            let p = Path::new(&a[1]);
            // Reserve a new file before SQLite opens it; never truncate an existing path.
            fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(p)
                .map_err(|e| format!("init 只接受新文件：{e}"))?;
            let _db = open(p)?;
            println!("initialized={} schema=1", p.display());
        }
        "add" if a.len() == 5 => {
            let item = Item {
                sku: text(&a[2])?.into(),
                name: text(&a[3])?.into(),
                stock: text(&a[4])?.parse().map_err(|_| "库存应为整数")?,
            };
            let db = existing(Path::new(&a[1]))?;
            save_item(&db, &item)?;
            println!("added={} stock={}", item.sku, item.stock);
        }
        "list" if a.len() == 2 => {
            let db = existing(Path::new(&a[1]))?;
            println!("SKU\t名称\t库存");
            for item in list_items(&db)? {
                println!("{}\t{}\t{}", item.sku, item.name, item.stock);
            }
        }
        "adjust" if a.len() == 4 => {
            let mut db = existing(Path::new(&a[1]))?;
            let change = Change {
                sku: text(&a[2])?.into(),
                delta: text(&a[3])?.parse().map_err(|_| "变化量应为整数")?,
            };
            apply_changes(&mut db, &[change])?;
            println!("committed=1");
        }
        "apply" if a.len() == 3 => {
            let changes = parse_changes(&fs::read_to_string(&a[2]).map_err(|e| e.to_string())?)?;
            let mut db = existing(Path::new(&a[1]))?;
            apply_changes(&mut db, &changes)?;
            println!("committed={}", changes.len());
        }
        "history" if a.len() == 2 => {
            let db = existing(Path::new(&a[1]))?;
            for (id, sku, delta) in history(&db)? {
                println!("{id}\t{sku}\t{delta}");
            }
        }
        "demo" if a.len() == 2 => {
            let dir = Path::new(&a[1]);
            fs::create_dir(dir).map_err(|e| format!("demo 只创建新目录：{e}"))?;
            let mut db = open(&dir.join("inventory.db"))?;
            save_item(
                &db,
                &Item {
                    sku: "PEN".into(),
                    name: "学习用笔".into(),
                    stock: 10,
                },
            )?;
            save_item(
                &db,
                &Item {
                    sku: "BOOK".into(),
                    name: "Rust 练习本".into(),
                    stock: 3,
                },
            )?;
            let changes = "PEN\t-2\nBOOK\t1\n";
            fs::write(dir.join("changes.tsv"), changes).map_err(|e| e.to_string())?;
            apply_changes(&mut db, &parse_changes(changes)?)?;
            let mut report = format!("StoreDesk · SQLite {}\n", sqlite::version());
            for item in list_items(&db)? {
                report.push_str(&format!("{}\t{}\t{}\n", item.sku, item.name, item.stock));
            }
            report.push_str(&format!("audit_rows={}\n", history(&db)?.len()));
            fs::write(dir.join("INVENTORY_REPORT.txt"), &report).map_err(|e| e.to_string())?;
            println!("{report}数据库：{}", dir.join("inventory.db").display());
        }
        _ => return Err(HELP.into()),
    }
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("StoreDesk 错误：{e}");
        std::process::exit(2);
    }
}
