//! 递归扫描的职责：找出目录项、拒绝链接和特殊文件、计算真实 SHA-256。
use crate::sha256::file_digest;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const MAX_ENTRIES: usize = 100_000;
pub const MAX_PATH_BYTES: usize = 2048;
const MAX_DEPTH: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    Directory,
    File { bytes: u64, sha256: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub path: String,
    pub kind: Kind,
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

pub fn check_component(part: &str) -> io::Result<()> {
    // 限制名称为 Windows/Linux/macOS 都可移植的常用子集；仍不承诺跨文件系统完全等价。
    let stem = part.split('.').next().unwrap_or("").to_ascii_uppercase();
    let reserved = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (stem.len() == 4
            && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && matches!(stem.as_bytes()[3], b'1'..=b'9'));
    if reserved
        || part.is_empty()
        || part == "."
        || part == ".."
        || part.chars().any(|ch| {
            matches!(
                ch,
                '/' | '\\' | ':' | '\0' | '<' | '>' | '"' | '|' | '?' | '*'
            )
        })
        || part.ends_with(' ')
        || part.ends_with('.')
    {
        return Err(invalid(format!("不支持的路径组件：{part:?}")));
    }
    Ok(())
}

pub fn relative_path(text: &str) -> io::Result<PathBuf> {
    if text.is_empty() || text.len() > MAX_PATH_BYTES {
        return Err(invalid("相对路径为空或过长"));
    }
    let mut result = PathBuf::new();
    for part in text.split('/') {
        check_component(part)?;
        result.push(part);
    }
    Ok(result)
}

pub fn require_real_dir(path: &Path) -> io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("必须提供真实目录（禁止符号链接）：{}", path.display()),
        ));
    }
    Ok(())
}

pub fn scan(root: &Path) -> io::Result<Vec<Entry>> {
    require_real_dir(root)?;
    let mut entries = Vec::new();
    visit(root, "", 0, &mut entries)?;
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(entries)
}

fn visit(directory: &Path, prefix: &str, depth: usize, out: &mut Vec<Entry>) -> io::Result<()> {
    if depth >= MAX_DEPTH {
        return Err(invalid("目录嵌套超过 64 层"));
    }
    let mut items = fs::read_dir(directory)?.collect::<io::Result<Vec<_>>>()?;
    items.sort_by_key(|item| item.file_name());
    for item in items {
        if out.len() >= MAX_ENTRIES {
            return Err(invalid("目录项超过 100000，已停止以限制资源消耗"));
        }
        let name = item
            .file_name()
            .into_string()
            .map_err(|_| invalid("文件名不是 UTF-8；为保证跨平台恢复，无法备份"))?;
        check_component(&name)?;
        let name = if prefix.is_empty() {
            name
        } else {
            format!("{prefix}/{name}")
        };
        if name.len() > MAX_PATH_BYTES {
            return Err(invalid("相对路径超过 2048 字节"));
        }
        let path = item.path();
        let meta = fs::symlink_metadata(&path)?;
        if meta.file_type().is_symlink() {
            return Err(invalid(format!(
                "包含符号链接，拒绝备份：{}",
                path.display()
            )));
        }
        if meta.is_dir() {
            out.push(Entry {
                path: name.clone(),
                kind: Kind::Directory,
            });
            visit(&path, &name, depth + 1, out)?;
        } else if meta.is_file() {
            let (bytes, sha256) = file_digest(&path)?;
            out.push(Entry {
                path: name,
                kind: Kind::File { bytes, sha256 },
            });
        } else {
            return Err(invalid(format!(
                "包含非普通文件，拒绝备份：{}",
                path.display()
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_path_escapes_and_ambiguous_windows_names() {
        for text in [
            "", "/etc", "../a", "a/../b", "a//b", "a\\b", "a:C", "a/.", "a/b.", "a/b ", "CON",
            "con.txt", "LPT1.log", "COM9", "a?b", "a<b",
        ] {
            assert!(relative_path(text).is_err(), "{text}");
        }
        assert_eq!(
            relative_path("中文/a b").unwrap(),
            PathBuf::from("中文/a b")
        );
    }
}
