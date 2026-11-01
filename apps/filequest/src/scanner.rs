//! 文件扫描核心：遍历、归类和检索。本模块不修改被扫描的目录。
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const IGNORED: &[&str] = &[
    ".git",
    ".venv",
    ".next",
    "target",
    "node_modules",
    "__pycache__",
    "dist",
    "build",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileRecord {
    pub relative_path: PathBuf,
    pub bytes: u64,
    pub extension: String,
}

#[derive(Debug)]
pub struct ScanSummary {
    pub root: PathBuf,
    pub files: Vec<FileRecord>,
    /// 已进入的子目录数量，不含根目录。
    pub directories: usize,
    pub ignored_directories: usize,
    pub skipped_symlinks: usize,
    /// 无权访问的路径、已消失的文件等，属于非致命错误。
    pub problems: Vec<String>,
}

impl ScanSummary {
    pub fn total_bytes(&self) -> u64 {
        self.files
            .iter()
            .fold(0_u64, |sum, file| sum.saturating_add(file.bytes))
    }

    pub fn extension_counts(&self) -> Vec<(String, usize)> {
        let mut counts = BTreeMap::<String, usize>::new();
        for file in &self.files {
            *counts.entry(file.extension.clone()).or_default() += 1;
        }
        let mut result: Vec<_> = counts.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        result
    }

    pub fn largest_files(&self, limit: usize) -> Vec<&FileRecord> {
        let mut sorted: Vec<_> = self.files.iter().collect();
        sorted.sort_by(|a, b| {
            b.bytes
                .cmp(&a.bytes)
                .then_with(|| a.relative_path.cmp(&b.relative_path))
        });
        sorted.truncate(limit);
        sorted
    }

    /// 按相对路径（不仅是末尾文件名）进行不区分大小写的包含匹配。
    pub fn find(&self, query: &str) -> Vec<&FileRecord> {
        let query = query.to_lowercase();
        self.files
            .iter()
            .filter(|item| {
                item.relative_path
                    .to_string_lossy()
                    .to_lowercase()
                    .contains(&query)
            })
            .collect()
    }
}

/// 对真实目录进行只读扫描。为避免循环和越界访问，始终不跟随符号链接。
/// 默认跳过常见依赖和构建目录，`include_ignored=true` 时可纳入它们。
pub fn scan(root: &Path, include_ignored: bool) -> io::Result<ScanSummary> {
    if fs::symlink_metadata(root)?.file_type().is_symlink() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "根路径不能是符号链接，请指定真实目录",
        ));
    }
    let root = fs::canonicalize(root)?;
    if !fs::metadata(&root)?.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "扫描对象必须是目录，而不是文件",
        ));
    }

    let mut result = ScanSummary {
        root: root.clone(),
        files: Vec::new(),
        directories: 0,
        ignored_directories: 0,
        skipped_symlinks: 0,
        problems: Vec::new(),
    };
    let mut pending = vec![root.clone()];
    while let Some(dir) = pending.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(err) if dir == root => return Err(err),
            Err(err) => {
                result.problems.push(format!("{}：{err}", dir.display()));
                continue;
            }
        };
        let mut available = Vec::new();
        for entry in entries {
            match entry {
                Ok(item) => available.push(item),
                Err(err) => result.problems.push(format!("{}：{err}", dir.display())),
            }
        }
        available.sort_by_key(|item| item.file_name());

        for entry in available {
            let path = entry.path();
            let kind = match entry.file_type() {
                Ok(kind) => kind,
                Err(err) => {
                    result.problems.push(format!("{}：{err}", path.display()));
                    continue;
                }
            };
            if kind.is_symlink() {
                result.skipped_symlinks += 1;
                continue;
            }
            if kind.is_dir() {
                let dir_name = entry.file_name();
                if !include_ignored && IGNORED.iter().any(|name| dir_name.to_str() == Some(*name)) {
                    result.ignored_directories += 1;
                } else {
                    result.directories += 1;
                    pending.push(path);
                }
            } else if kind.is_file() {
                match entry.metadata() {
                    Ok(meta) => {
                        let relative_path = path
                            .strip_prefix(&root)
                            .expect("遍历获得的文件应位于根目录下")
                            .to_path_buf();
                        result.files.push(FileRecord {
                            extension: extension_of(&relative_path),
                            relative_path,
                            bytes: meta.len(),
                        });
                    }
                    Err(err) => result.problems.push(format!("{}：{err}", path.display())),
                }
            }
        }
    }
    result
        .files
        .sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    Ok(result)
}

pub fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    let mut value = bytes as f64;
    let mut unit = "B";
    for next in ["KiB", "MiB", "GiB", "TiB", "PiB", "EiB"] {
        if value < 1024.0 {
            break;
        }
        value /= 1024.0;
        unit = next;
    }
    format!("{value:.1} {unit}")
}

fn extension_of(path: &Path) -> String {
    path.extension()
        .and_then(|item| item.to_str())
        .filter(|item| !item.is_empty())
        .map(str::to_lowercase)
        .unwrap_or_else(|| "(无扩展名)".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human_readable_bytes() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(1024), "1.0 KiB");
        assert_eq!(format_size(1536), "1.5 KiB");
    }

    #[test]
    fn extension_is_normalized() {
        assert_eq!(extension_of(Path::new("src/MAIN.RS")), "rs");
        assert_eq!(extension_of(Path::new("README")), "(无扩展名)");
    }
}
