use minikv::{
    journal::{FileJournal, Journal, MemoryJournal},
    Database, KvError, Operation, Outcome,
};
use std::{
    ffi::OsString,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::Path,
};

fn text(value: &OsString) -> Result<String, String> {
    value
        .to_str()
        .map(str::to_owned)
        .ok_or("键和值必须是有效 UTF-8".into())
}
fn display(value: &str) -> String {
    value.chars().flat_map(char::escape_debug).collect()
}
fn show(outcome: Outcome) {
    match outcome {
        Outcome::Stored { replaced } => println!("stored replaced={replaced}"),
        Outcome::Value(value) => println!("{}", display(&value)),
        Outcome::Removed => println!("removed"),
    }
}
fn open(path: &Path, create: bool) -> Result<Database<FileJournal>, KvError> {
    if !create && !path.is_file() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "数据库不存在；先用 put 创建").into());
    }
    Database::open(FileJournal::open(path)?)
}
fn execute<J: Journal>(db: &mut Database<J>, op: Operation) -> Result<(), String> {
    show(db.execute(op).map_err(|e| format!("[{}] {e}", e.kind()))?);
    Ok(())
}
fn export(db: &Database<FileJournal>, path: &Path) -> Result<(), String> {
    let body = std::iter::once("key\tvalue\n".to_string())
        .chain(
            db.scan("")
                .map(|(k, v)| format!("{}\t{}\n", display(k), display(v))),
        )
        .collect::<String>();
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|e| format!("不覆盖已有导出文件：{e}"))?;
    file.write_all(body.as_bytes()).map_err(|e| e.to_string())?;
    println!("exported={} path={}", db.len(), path.display());
    Ok(())
}
fn demo(root: &Path) -> Result<(), String> {
    fs::create_dir(root).map_err(|e| format!("演示需要全新目录：{e}"))?;
    let path = root.join("my-bookmarks.kv");
    {
        let mut db = open(&path, true).map_err(|e| e.to_string())?;
        for (key, value) in [
            ("course/next", "理解借用迭代器"),
            ("bookmark/rust", "https://doc.rust-lang.org/book/"),
            ("bookmark/project", "我的 Rust 作品集"),
            ("scratch/remove", "临时记录"),
        ] {
            execute(
                &mut db,
                Operation::Put {
                    key: key.into(),
                    value: value.into(),
                },
            )?;
        }
        execute(
            &mut db,
            Operation::Delete {
                key: "scratch/remove".into(),
            },
        )?;
    }
    let db = open(&path, false).map_err(|e| e.to_string())?;
    export(&db, &root.join("bookmarks.tsv"))?;
    fs::write(root.join("MY_PRODUCT.md"),format!("# 我的 MiniKV 作品\n\n数据库：my-bookmarks.kv\n重新打开后的键数：{}\n\n修改自己的键和值，再运行 get、scan 和 export。日志不加密，请勿保存密码。\n",db.len())).map_err(|e|e.to_string())?;
    println!("reopened={} demo={}", db.len(), root.display());
    Ok(())
}
fn share_demo(path: &Path) -> Result<(), String> {
    if path.exists() {
        return Err("共享实验请使用全新数据库路径，不修改已有资料".into());
    }
    let shared = minikv::share(open(path, true).map_err(|e| e.to_string())?);
    let handles = (0..4)
        .map(|worker| {
            let db = std::sync::Arc::clone(&shared);
            std::thread::spawn(move || -> Result<(), KvError> {
                for item in 0..10 {
                    let mut guard = db.lock().map_err(|_| KvError::Poisoned)?;
                    guard.execute(Operation::Put {
                        key: format!("worker/{worker}/{item:02}"),
                        value: format!("工作线程 {worker}"),
                    })?;
                }
                Ok(())
            })
        })
        .collect::<Vec<_>>();
    for handle in handles {
        handle
            .join()
            .map_err(|_| "工作线程异常退出")?
            .map_err(|e| e.to_string())?;
    }
    let count = shared.lock().map_err(|_| "共享锁中毒")?.len();
    println!("workers=4 records={count} expected=40");
    if count != 40 {
        return Err("并发结果数量不符".into());
    }
    Ok(())
}
fn help() {
    println!("MiniKV · 本地配置与书签仓库\n\n  minikv demo <新目录>\n  minikv put <数据库.kv> <键> <值>\n  minikv get <数据库.kv> <键>\n  minikv delete <数据库.kv> <键>\n  minikv scan <数据库.kv> [前缀]\n  minikv export <数据库.kv> <新报告.tsv>\n  minikv share-demo <新数据库.kv>\n  minikv memory-demo\n\n键按 UTF-8 字节序输出。读取/删除不会创建缺失数据库。\n日志为教学格式，非加密存储；不要保存密码。不支持多键事务或自动修复。\n正常退出释放 .lock；异常终止需先确认没有运行进程，再人工处理锁文件。\n");
}
fn run(args: Vec<OsString>) -> Result<(), String> {
    let cmd = args.first().and_then(|a| a.to_str()).unwrap_or("help");
    match (cmd, args.len()) {
        ("help" | "--help" | "-h", _) => {
            help();
            Ok(())
        }
        ("demo", 2) => demo(Path::new(&args[1])),
        ("share-demo", 2) => share_demo(Path::new(&args[1])),
        ("memory-demo", 1) => {
            let mut db = Database::open(MemoryJournal::default()).map_err(|e| e.to_string())?;
            execute(
                &mut db,
                Operation::Put {
                    key: "session/goal".into(),
                    value: "内存后端不创建文件".into(),
                },
            )?;
            execute(
                &mut db,
                Operation::Get {
                    key: "session/goal".into(),
                },
            )
        }
        ("put", 4) | ("get" | "delete", 3) => {
            let key = text(&args[2])?;
            // Reject invalid input BEFORE opening/creating any file.
            minikv::validate_key(&key).map_err(|e| format!("[{}] {e}", e.kind()))?;
            let op = match cmd {
                "put" => {
                    let value = text(&args[3])?;
                    if value.len() > minikv::MAX_VALUE_BYTES {
                        return Err("value-too-large".into());
                    }
                    Operation::Put { key, value }
                }
                "get" => Operation::Get { key },
                _ => Operation::Delete { key },
            };
            let mut db = open(Path::new(&args[1]), cmd == "put")
                .map_err(|e| format!("[{}] {e}", e.kind()))?;
            execute(&mut db, op)
        }
        ("scan", 2 | 3) => {
            let db = open(Path::new(&args[1]), false).map_err(|e| format!("[{}] {e}", e.kind()))?;
            let prefix = args.get(2).map(text).transpose()?.unwrap_or_default();
            for (k, v) in db.scan(&prefix) {
                println!("{}\t{}", display(k), display(v));
            }
            Ok(())
        }
        ("export", 3) => {
            let db = open(Path::new(&args[1]), false).map_err(|e| e.to_string())?;
            export(&db, Path::new(&args[2]))
        }
        _ => Err("参数不匹配。运行 minikv help；带空格的键和值需用引号包住".into()),
    }
}
fn main() {
    if let Err(err) = run(std::env::args_os().skip(1).collect()) {
        eprintln!("MiniKV 错误：{err}");
        std::process::exit(2);
    }
}
