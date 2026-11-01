//! Local key/value storage for learning custom errors, traits, iterators and shared ownership.
//! The journal implementation is supplied infrastructure; learners implement three small stages.
pub mod journal;
use journal::Journal;
use std::{
    collections::{btree_map, BTreeMap},
    error::Error,
    fmt, io,
    path::PathBuf,
    sync::{Arc, Mutex},
};

pub const MAX_KEY_BYTES: usize = 128;
pub const MAX_VALUE_BYTES: usize = 65_536;
#[derive(Debug)]
pub enum KvError {
    InvalidKey(&'static str),
    ValueTooLarge,
    Missing(String),
    Io(io::Error),
    Corrupt { line: usize, reason: String },
    Busy(PathBuf),
    Poisoned,
}
impl KvError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::InvalidKey(_) => "invalid-key",
            Self::ValueTooLarge => "value-too-large",
            Self::Missing(_) => "not-found",
            Self::Io(_) => "io",
            Self::Corrupt { .. } => "corrupt",
            Self::Busy(_) => "busy",
            Self::Poisoned => "poisoned",
        }
    }
}
impl fmt::Display for KvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidKey(reason) => write!(f, "键不合法：{reason}"),
            Self::ValueTooLarge => write!(f, "值不能超过 {MAX_VALUE_BYTES} 字节"),
            Self::Missing(key) => write!(f, "键不存在：{key}"),
            Self::Io(err) => write!(f, "文件操作失败：{err}"),
            Self::Corrupt { line, reason } => {
                write!(f, "日志第 {line} 行损坏：{reason}；未自动修改原文件")
            }
            Self::Busy(path) => write!(
                f,
                "数据库被占用：{}；确认所有 MiniKV 进程已退出后再检查同名 .lock 文件",
                path.display()
            ),
            Self::Poisoned => write!(
                f,
                "写入或共享锁发生故障，当前会话停止继续操作；请先检查文件"
            ),
        }
    }
}
impl Error for KvError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            _ => None,
        }
    }
}
impl From<io::Error> for KvError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    Put { key: String, value: String },
    Get { key: String },
    Delete { key: String },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Stored { replaced: bool },
    Value(String),
    Removed,
}

/// Stage 1: validate without trimming, silently changing keys, or performing I/O.
pub fn validate_key(key: &str) -> Result<(), KvError> {
        // TODO(1): 只实现当前阶段，讲义在 START.html。
        // LESSON 1 BEGIN
        let _ = key;
    Err(KvError::InvalidKey("尚未完成键验证"))
        // LESSON 1 END
    }

/// Generic over a storage journal: tests can use memory, the CLI uses a real file.
pub struct Database<J: Journal> {
    data: BTreeMap<String, String>,
    journal: J,
}
impl<J: Journal> Database<J> {
    pub fn open(mut journal: J) -> Result<Self, KvError> {
        let entries = journal.read()?;
        let mut data = BTreeMap::new();
        for (index, op) in entries.into_iter().enumerate() {
            let invalid = |reason: String| KvError::Corrupt {
                line: index + 2,
                reason,
            };
            match op {
                Operation::Put { key, value } => {
                    validate_key(&key).map_err(|e| invalid(e.to_string()))?;
                    if value.len() > MAX_VALUE_BYTES {
                        return Err(invalid("值超出容量上限".into()));
                    }
                    data.insert(key, value);
                }
                Operation::Delete { key } => {
                    validate_key(&key).map_err(|e| invalid(e.to_string()))?;
                    if data.remove(&key).is_none() {
                        return Err(invalid("删除了不存在的键".into()));
                    }
                }
                Operation::Get { .. } => return Err(invalid("读取操作不应写入日志".into())),
            }
        }
        Ok(Self { data, journal })
    }
    /// Stage 2: validate first, append a mutation, THEN update memory. Reads don't append.
    pub fn execute(&mut self, operation: Operation) -> Result<Outcome, KvError> {
        // TODO(2): 只实现当前阶段，讲义在 START.html。
        // LESSON 2 BEGIN
        let _ = operation;
        let _ = &self.journal;
        Err(KvError::InvalidKey("尚未完成读写逻辑"))
        // LESSON 2 END
    }
    /// The returned iterator borrows this database; mutation is forbidden while that borrow is live.
    ///
    /// ```compile_fail,E0502
    /// use minikv::{Database, Operation, journal::MemoryJournal};
    /// let mut db = Database::open(MemoryJournal::default()).unwrap();
    /// let mut rows = db.scan("");
    /// db.execute(Operation::Put { key: "new".into(), value: "value".into() }).unwrap();
    /// println!("{:?}", rows.next()); // the immutable borrow is still in use
    /// ```
    ///
    /// Own only the keys you need, end the borrow, then modify the database:
    /// ```
    /// use minikv::{Database, Operation, journal::MemoryJournal};
    /// let mut db = Database::open(MemoryJournal::default()).unwrap();
    /// db.execute(Operation::Put { key: "old/a".into(), value: "value".into() }).unwrap();
    /// let keys: Vec<String> = db.scan("old/").map(|(key, _)| key.to_owned()).collect();
    /// for key in keys { db.execute(Operation::Delete { key }).unwrap(); }
    /// assert!(db.is_empty());
    /// ```
    pub fn scan(&self, prefix: &str) -> PrefixScan<'_> {
        PrefixScan {
            inner: self.data.range(prefix.to_owned()..),
            prefix: prefix.to_owned(),
            finished: false,
        }
    }
    pub fn len(&self) -> usize {
        self.data.len()
    }
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

pub struct PrefixScan<'a> {
    inner: btree_map::Range<'a, String, String>,
    prefix: String,
    finished: bool,
}
impl<'a> Iterator for PrefixScan<'a> {
    type Item = (&'a str, &'a str);
    /// Stage 3: one borrowed item at a time. Sorted keys make a first nonmatch terminal.
    fn next(&mut self) -> Option<Self::Item> {
        // TODO(3): 只实现当前阶段，讲义在 START.html。
        // LESSON 3 BEGIN
        let _ = (&mut self.inner, &self.prefix, self.finished);
        None
        // LESSON 3 END
    }
}
impl std::iter::FusedIterator for PrefixScan<'_> {}

pub type SharedDatabase<J> = Arc<Mutex<Database<J>>>;
pub fn share<J: Journal>(database: Database<J>) -> SharedDatabase<J> {
    Arc::new(Mutex::new(database))
}
