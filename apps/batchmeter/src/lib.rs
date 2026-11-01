//! 标准库 scoped threads 实战：分片只读文件、消息传递、有序收集。
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    sync::mpsc,
    thread,
};
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileStats {
    pub path: PathBuf,
    pub bytes: u64,
    pub lines: usize,
}
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Totals {
    pub files: usize,
    pub bytes: u64,
    pub lines: usize,
}
/// 教学数据最大每文件 8 MiB，避免初学者误读巨型文件。
pub fn analyze(path: &Path) -> Result<FileStats, String> {
    // LESSON 1 BEGIN
    let mut buffer = Vec::new();
    File::open(path)
        .and_then(|f| f.take(8 * 1024 * 1024 + 1).read_to_end(&mut buffer))
        .map_err(|e| format!("{}：{e}", path.display()))?;
    if buffer.len() > 8 * 1024 * 1024 {
        return Err(format!("{} 超过 8 MiB", path.display()));
    }
    let bytes = buffer.len() as u64;
    let text =
        std::str::from_utf8(&buffer).map_err(|_| format!("{} 不是 UTF-8 文本", path.display()))?;
    Ok(FileStats {
        path: path.to_path_buf(),
        bytes,
        lines: text.lines().count(),
    })
    // LESSON 1 END
}
pub fn aggregate(rows: &[FileStats]) -> Result<Totals, String> {
    // LESSON 2 BEGIN
    let mut totals = Totals {
        files: rows.len(),
        ..Totals::default()
    };
    for row in rows {
        totals.bytes = totals
            .bytes
            .checked_add(row.bytes)
            .ok_or_else(|| "字节总数溢出".to_string())?;
        totals.lines = totals
            .lines
            .checked_add(row.lines)
            .ok_or_else(|| "行数总数溢出".to_string())?;
    }
    Ok(totals)
    // LESSON 2 END
}
pub fn analyze_many(paths: &[PathBuf], workers: usize) -> Result<Vec<FileStats>, String> {
    // LESSON 3 BEGIN
    if !(1..=16).contains(&workers) {
        return Err("工作线程数必须是 1..16".into());
    }
    if paths.len() > 8192 {
        return Err("一次最多处理 8192 个路径".into());
    }
    if paths.is_empty() {
        return Ok(Vec::new());
    }
    let chunk = paths.len().div_ceil(workers);
    let (tx, rx) = mpsc::channel();
    thread::scope(|scope| -> Result<(), String> {
        let mut handles = Vec::new();
        for (batch, items) in paths.chunks(chunk).enumerate() {
            let tx = tx.clone();
            handles.push(scope.spawn(move || {
                for (offset, path) in items.iter().enumerate() {
                    if tx.send((batch * chunk + offset, analyze(path))).is_err() {
                        break;
                    }
                }
            }));
        }
        for handle in handles {
            handle.join().map_err(|_| "工作线程意外退出".to_string())?;
        }
        Ok(())
    })?;
    drop(tx); // 所有 sender 已销毁，receiver 的迭代才会结束。
    let mut indexed: Vec<_> = rx.into_iter().collect();
    indexed.sort_by_key(|(index, _)| *index);
    indexed.into_iter().map(|(_, result)| result).collect()
    // LESSON 3 END
}
