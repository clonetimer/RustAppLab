//! 标准库 scoped threads 实战：分片只读文件、消息传递、有序收集。
#[allow(unused_imports)] // 在第 1 步实现后这些 import 会被真正使用。
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
    // TODO(1): 读取真实文件并明确失败。先读 LESSON.md 对应步骤。
    let _ = path;
    let _buffer = Vec::<u8>::new();
    Err("请完成第 1 步：读取单文件".into())
    // LESSON 1 END
}
pub fn aggregate(rows: &[FileStats]) -> Result<Totals, String> {
    // LESSON 2 BEGIN
    // TODO(2): 用结构体累计批处理结果。先读 LESSON.md 对应步骤。
    let _ = rows;
    Err("请完成第 2 步：汇总结果".into())
    // LESSON 2 END
}
pub fn analyze_many(paths: &[PathBuf], workers: usize) -> Result<Vec<FileStats>, String> {
    // LESSON 3 BEGIN
    // TODO(3): 用作用域线程与通道保持结果有序。先读 LESSON.md 对应步骤。
    let _ = (paths, workers);
    let _ = mpsc::channel::<usize>();
    let _ = thread::current();
    Err("请完成第 3 步：并行处理".into())
    // LESSON 3 END
}
