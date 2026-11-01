//! Supplied infrastructure. The format is inspectable, not encrypted or authenticated.
//! Operations are serialized by Database. Cross-process lock files are conservative.
use crate::{KvError, Operation, MAX_VALUE_BYTES};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};
const HEADER: &str = "RUSTAPPLAB_MINIKV_V1\n";
pub const MAX_LOG_BYTES: u64 = 16 * 1024 * 1024;

pub trait Journal: Send {
    fn read(&mut self) -> Result<Vec<Operation>, KvError>;
    fn append(&mut self, operation: &Operation) -> Result<(), KvError>;
}
impl<T: Journal + ?Sized> Journal for Box<T> {
    fn read(&mut self) -> Result<Vec<Operation>, KvError> {
        (**self).read()
    }
    fn append(&mut self, op: &Operation) -> Result<(), KvError> {
        (**self).append(op)
    }
}
#[derive(Default)]
pub struct MemoryJournal {
    pub operations: Vec<Operation>,
    pub fail_next: bool,
}
impl Journal for MemoryJournal {
    fn read(&mut self) -> Result<Vec<Operation>, KvError> {
        Ok(self.operations.clone())
    }
    fn append(&mut self, op: &Operation) -> Result<(), KvError> {
        if self.fail_next {
            self.fail_next = false;
            return Err(std::io::Error::other("模拟日志写入失败").into());
        }
        self.operations.push(op.clone());
        Ok(())
    }
}
// Existence guard; we close the file at acquisition so Windows can remove it during normal Drop.
struct Lease {
    path: PathBuf,
}
impl Drop for Lease {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
pub struct FileJournal {
    file: File,
    _lease: Lease,
    poisoned: bool,
}
fn ordinary_or_missing(path: &Path) -> Result<(), KvError> {
    match fs::symlink_metadata(path) {
        Ok(m) if m.file_type().is_symlink() || !m.is_file() => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "数据库路径必须是普通文件，不允许目录或符号链接",
        )
        .into()),
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}
impl FileJournal {
    pub fn open(path: &Path) -> Result<Self, KvError> {
        ordinary_or_missing(path)?;
        let mut lock_name = path.as_os_str().to_owned();
        lock_name.push(".lock");
        let lock_path = PathBuf::from(lock_name);
        let mut lock = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&lock_path)
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::AlreadyExists {
                    KvError::Busy(path.to_owned())
                } else {
                    e.into()
                }
            })?;
        let lease = Lease { path: lock_path };
        let written = writeln!(lock, "MiniKV owner pid={}", std::process::id());
        drop(lock);
        written?;
        ordinary_or_missing(path)?;
        let (mut file, created) = match OpenOptions::new()
            .create_new(true)
            .append(true)
            .read(true)
            .open(path)
        {
            Ok(file) => (file, true),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => (
                OpenOptions::new().append(true).read(true).open(path)?,
                false,
            ),
            Err(error) => return Err(error.into()),
        };
        if created {
            file.write_all(HEADER.as_bytes())?;
            file.sync_data()?;
        }
        Ok(Self {
            file,
            _lease: lease,
            poisoned: false,
        })
    }
}
impl Journal for FileJournal {
    fn read(&mut self) -> Result<Vec<Operation>, KvError> {
        let corrupt = |line, reason: &str| KvError::Corrupt {
            line,
            reason: reason.into(),
        };
        if self.file.metadata()?.len() > MAX_LOG_BYTES {
            return Err(corrupt(0, "日志超过 16 MiB；教学版不自动压缩"));
        }
        self.file.seek(SeekFrom::Start(0))?;
        let mut bytes = Vec::new();
        (&mut self.file)
            .take(MAX_LOG_BYTES + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX_LOG_BYTES {
            return Err(corrupt(0, "读取时日志超过容量上限"));
        }
        let text = std::str::from_utf8(&bytes).map_err(|_| corrupt(1, "日志不是 UTF-8"))?;
        if !text.starts_with(HEADER) {
            return Err(corrupt(1, "不是本应用的 V1 日志"));
        }
        if !text.ends_with('\n') {
            return Err(corrupt(text.lines().count(), "末条记录不完整"));
        }
        text[HEADER.len()..]
            .lines()
            .enumerate()
            .map(|(index, line)| decode(line, index + 2))
            .collect()
    }
    fn append(&mut self, op: &Operation) -> Result<(), KvError> {
        if self.poisoned {
            return Err(KvError::Poisoned);
        }
        let encoded = encode(op)?;
        if self.file.metadata()?.len() + encoded.len() as u64 > MAX_LOG_BYTES {
            return Err(std::io::Error::other("日志已达 16 MiB 教学上限，请另建数据库").into());
        }
        self.poisoned = true;
        self.file.write_all(encoded.as_bytes())?;
        self.file.sync_data()?;
        self.poisoned = false;
        Ok(())
    }
}
fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(DIGITS[(b >> 4) as usize] as char);
        out.push(DIGITS[(b & 15) as usize] as char);
    }
    out
}
fn unhex(text: &str) -> Result<String, &'static str> {
    if text.len() % 2 != 0 || text.len() > MAX_VALUE_BYTES * 2 {
        return Err("十六进制长度不合法");
    }
    fn digit(b: u8) -> Option<u8> {
        match b {
            b'0'..=b'9' => Some(b - b'0'),
            b'a'..=b'f' => Some(b - b'a' + 10),
            _ => None,
        }
    }
    let bytes = text
        .as_bytes()
        .chunks_exact(2)
        .map(|p| Ok((digit(p[0]).ok_or("非法十六进制")? << 4) | digit(p[1]).ok_or("非法十六进制")?))
        .collect::<Result<Vec<u8>, &str>>()?;
    String::from_utf8(bytes).map_err(|_| "记录包含非法 UTF-8")
}
fn encode(op: &Operation) -> Result<String, KvError> {
    match op {
        Operation::Put { key, value } => Ok(format!(
            "P\t{}\t{}\n",
            hex(key.as_bytes()),
            hex(value.as_bytes())
        )),
        Operation::Delete { key } => Ok(format!("D\t{}\n", hex(key.as_bytes()))),
        Operation::Get { .. } => {
            Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "不能持久化读取命令").into())
        }
    }
}
fn decode(line: &str, number: usize) -> Result<Operation, KvError> {
    let parts: Vec<_> = line.split('\t').collect();
    let invalid = |reason: &str| KvError::Corrupt {
        line: number,
        reason: reason.into(),
    };
    match parts.as_slice() {
        ["P", key, value] => Ok(Operation::Put {
            key: unhex(key).map_err(invalid)?,
            value: unhex(value).map_err(invalid)?,
        }),
        ["D", key] => Ok(Operation::Delete {
            key: unhex(key).map_err(invalid)?,
        }),
        _ => Err(invalid("记录字段或操作类型不合法")),
    }
}
