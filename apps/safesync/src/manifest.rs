//! Manifest 使用行格式 + UTF-8 相对路径的十六进制编码；路径无制表符注入风险。
//! 清单没有密钥或签名：提供事故/比特损坏检测，不负责抵御主动恶意篡改。
use crate::sha256::{from_hex, hex};
use crate::tree::{relative_path, Entry, Kind, MAX_ENTRIES};
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;

pub const MANIFEST: &str = "manifest.sfs";
const HEADER: &str = "SAFESYNC\t1\n";
const MAX_MANIFEST_BYTES: u64 = 8 * 1024 * 1024;

fn malformed(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, format!("清单无效：{message}"))
}

pub fn save(path: &Path, entries: &[Entry]) -> io::Result<()> {
    let mut file = File::options().write(true).create_new(true).open(path)?;
    file.write_all(HEADER.as_bytes())?;
    for entry in entries {
        let encoded_path = hex(entry.path.as_bytes());
        match &entry.kind {
            Kind::Directory => writeln!(file, "D\t{encoded_path}")?,
            Kind::File { bytes, sha256 } => {
                writeln!(file, "F\t{encoded_path}\t{bytes}\t{sha256}")?;
            }
        }
    }
    file.sync_all()?;
    Ok(())
}

pub fn load(path: &Path) -> io::Result<Vec<Entry>> {
    if fs::symlink_metadata(path)?.file_type().is_symlink() {
        return Err(malformed("不允许符号链接清单"));
    }
    let mut file = File::open(path)?;
    if !file.metadata()?.is_file() {
        return Err(malformed("清单必须是普通文件"));
    }
    let mut bytes = Vec::new();
    std::io::Read::by_ref(&mut file)
        .take(MAX_MANIFEST_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_MANIFEST_BYTES {
        return Err(malformed("清单超过 8 MiB 限制"));
    }
    let content = std::str::from_utf8(&bytes).map_err(|_| malformed("不是 UTF-8"))?;
    let records = content
        .strip_prefix(HEADER)
        .ok_or_else(|| malformed("版本标识不匹配"))?;
    if !records.is_empty() && !records.ends_with('\n') {
        return Err(malformed("记录末尾缺少换行"));
    }
    let mut result = Vec::new();
    let mut seen = HashSet::new();
    for line in records.lines() {
        if result.len() >= MAX_ENTRIES {
            return Err(malformed("记录过多"));
        }
        let fields: Vec<_> = line.split('\t').collect();
        let encoded = fields.get(1).ok_or_else(|| malformed("缺少路径"))?;
        let path = from_hex(encoded)
            .and_then(|raw| String::from_utf8(raw).ok())
            .ok_or_else(|| malformed("路径十六进制编码错误"))?;
        relative_path(&path).map_err(|_| malformed("危险或无效的相对路径"))?;
        if !seen.insert(path.clone()) {
            return Err(malformed("重复路径"));
        }
        let kind = match fields.as_slice() {
            ["D", _] => Kind::Directory,
            ["F", _, size, digest] => {
                let bytes = size
                    .parse::<u64>()
                    .map_err(|_| malformed("文件大小格式错误"))?;
                if digest.len() != 64
                    || !digest
                        .bytes()
                        .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
                {
                    return Err(malformed("SHA-256 格式错误"));
                }
                Kind::File {
                    bytes,
                    sha256: (*digest).to_owned(),
                }
            }
            _ => return Err(malformed("记录字段数量不正确")),
        };
        result.push(Entry { path, kind });
    }
    if result
        .windows(2)
        .any(|window| window[0].path >= window[1].path)
    {
        return Err(malformed("文件清单未按路径排序"));
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sha256::hex;
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn rejects_traversal_and_duplicates() {
        let file = env::temp_dir().join(format!(
            "sfs-manifest-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        for data in [
            format!("{HEADER}D\t{}\n", hex(b"../../etc")),
            format!("{HEADER}D\t{}\nD\t{}\n", hex(b"x"), hex(b"x")),
            format!("{HEADER}F\t{}\t1\tbad\n", hex(b"x")),
            format!("{HEADER}D\t{}", hex(b"x")),
        ] {
            fs::write(&file, data).unwrap();
            assert!(load(&file).is_err());
        }
        fs::remove_file(file).unwrap();
    }
}
