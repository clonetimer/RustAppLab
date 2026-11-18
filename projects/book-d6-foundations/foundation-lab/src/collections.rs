//! 向量、UTF-8 字符串、哈希表与可恢复错误练习。
//!
//! # Vec 的读取与可变遍历
//!
//! 对外部输入的下标，用 `get` 处理缺项。直接索引要求下标在范围内。
//!
//! ```
//! use book_d6_foundation_lab::collections::{annotate_labels, select_label};
//!
//! let mut labels = vec![String::from("作业"), String::from("测验")];
//! assert_eq!(&labels[0], "作业");
//! assert_eq!(select_label(&labels, 2), None);
//! for label in &labels {
//!     assert!(!label.is_empty());
//! }
//! annotate_labels(&mut labels, "✓");
//! assert_eq!(select_label(&labels, 1), Some("测验✓"));
//!
//! let mut scores = Vec::new();
//! scores.push(42_u32);
//! for score in &mut scores {
//!     *score += 1;
//! }
//! assert_eq!(scores.pop(), Some(43));
//! assert_eq!(scores.pop(), None);
//! ```
//!
//! 同一个 `Vec<T>` 的元素类型一致。枚举把不同数据形状放进同一个类型。
//!
//! ```
//! enum Cell {
//!     Score(u32),
//!     Label(String),
//! }
//!
//! let cells = vec![Cell::Score(42), Cell::Label(String::from("测验"))];
//! let display: Vec<String> = cells
//!     .iter()
//!     .map(|cell| match cell {
//!         Cell::Score(value) => value.to_string(),
//!         Cell::Label(label) => format!("[{label}]"),
//!     })
//!     .collect();
//! assert_eq!(display, ["42", "[测验]"]);
//! ```
//!
//! # String 的追加、拼接与边界
//!
//! `push_str` 借用一段 UTF-8 文本，`push` 追加一个 `char`。
//! `+` 接收左侧的 String 所有权并借用右侧文本，`format!` 在下例中保留两个输入。
//!
//! ```
//! let mut label = String::from("作业");
//! let suffix = String::from("完成");
//! label.push('：');
//! label.push_str(&suffix);
//! assert_eq!(suffix, "完成");
//!
//! let mark = String::from("✓");
//! let combined = format!("{label}{mark}");
//! assert_eq!(label, "作业：完成");
//! assert_eq!(mark, "✓");
//! let joined = label + mark.as_str();
//! assert_eq!(mark, "✓");
//! assert_eq!(joined, combined);
//! ```
//!
//! ```compile_fail,E0382
//! let left = String::from("作业");
//! let joined = left + "✓";
//! println!("{joined}");
//! println!("{left}");
//! ```
//!
//! Rust 字符串禁止整数索引。下面的 `get` 接收字节范围，并检查两端是 UTF-8 边界。
//!
//! ```compile_fail,E0277
//! let label = String::from("A中");
//! let _ = label[0];
//! ```
//!
//! ```
//! let label = String::from("A中");
//! assert_eq!(label.len(), 4);
//! assert_eq!(label.bytes().collect::<Vec<_>>(), [0x41, 0xE4, 0xB8, 0xAD]);
//! assert_eq!(label.chars().nth(1), Some('中'));
//! assert_eq!(&label[1..4], "中");
//! assert_eq!(label.get(1..4), Some("中"));
//! assert_eq!(label.get(1..2), None);
//! ```
//!
//! 直接索引的前提不成立时会触发 panic。这里用局部捕获观察两种失效条件。
//!
//! ```
//! use std::panic::catch_unwind;
//!
//! let labels = vec![String::from("A中")];
//! let missing = labels.len();
//! assert!(catch_unwind(|| &labels[missing]).is_err());
//!
//! let label = &labels[0];
//! assert!(catch_unwind(|| &label[1..2]).is_err());
//! ```
//!
//! # HashMap 的所有权与更新策略
//!
//! 拥有 `String` 键的 map 可以用 `&str` 查询。`get` 返回 `Option<&V>`。
//! `insert` 返回被替换的旧值。`or_insert` 保留已有值，返回 `&mut V`。
//!
//! ```
//! use std::collections::HashMap;
//!
//! let name = String::from("Lin");
//! let mut owned = HashMap::new();
//! owned.insert(name, 80);
//! let score: Option<&u32> = owned.get("Lin");
//! assert_eq!(score, Some(&80));
//! assert_eq!(owned.get("absent"), None);
//! assert_eq!(owned.insert(String::from("Lin"), 91), Some(80));
//! let score: &mut u32 = owned.entry(String::from("Lin")).or_insert(0);
//! assert_eq!(*score, 91);
//! *score += 1;
//! assert_eq!(owned.get("Lin"), Some(&92));
//!
//! let name = String::from("Kai");
//! let mut borrowed = HashMap::new();
//! borrowed.insert(name.as_str(), 55);
//! assert_eq!(borrowed.get("Kai"), Some(&55));
//! assert_eq!(name, "Kai");
//! ```
//!
//! ```compile_fail,E0382
//! use std::collections::HashMap;
//!
//! let name = String::from("Lin");
//! let mut owned = HashMap::new();
//! owned.insert(name, 80);
//! println!("{name}");
//! ```
//!
//! # match、? 与 From
//!
//! 两版读取函数执行相同的步骤。`?` 遇 `Err` 时用 `From` 转为外层错误并提前返回。
//!
//! ```
//! use book_d6_foundation_lab::collections::{read_score, read_score_explicit, ScoreError};
//!
//! assert_eq!(read_score(&b" 42 \n7"[..]).unwrap(), 42);
//! assert_eq!(read_score_explicit(&b" 42 \n7"[..]).unwrap(), 42);
//! assert!(matches!(read_score(&b"bad\n42"[..]), Err(ScoreError::Parse(_))));
//! assert!(matches!(read_score_explicit(&b"\xff\n"[..]), Err(ScoreError::Io(_))));
//! ```
//!
//! 这里的 `?` 传播 `Result` 的错误，外层函数返回 `()` 会产生 E0277。
//!
//! ```compile_fail,E0277
//! use book_d6_foundation_lab::collections::read_score;
//!
//! fn main() {
//!     let _ = read_score(&b"42\n"[..])?;
//! }
//! ```
//!
//! ```
//! use book_d6_foundation_lab::collections::{read_score, ScoreError};
//!
//! fn main() -> Result<(), ScoreError> {
//!     assert_eq!(read_score(&b"42\n"[..])?, 42);
//!     Ok(())
//! }
//! ```

use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::io::{self, BufRead};
use std::num::ParseIntError;

/// 按下标借用标签，越界时返回 `None`。
pub fn select_label(labels: &[String], index: usize) -> Option<&str> {
    labels.get(index).map(String::as_str)
}

/// 原地向每个标签追加后缀。
pub fn annotate_labels(labels: &mut [String], suffix: &str) {
    for label in labels {
        label.push_str(suffix);
    }
}

/// 为每个姓名保存分数，重复姓名以最后一项为准。
pub fn scores_by_name(entries: &[(&str, u32)]) -> HashMap<String, u32> {
    let mut scores = HashMap::new();
    for &(name, score) in entries {
        scores.insert(name.to_owned(), score);
    }
    scores
}

/// 区分读取失败和整数解析失败，并保留底层错误。
#[derive(Debug)]
pub enum ScoreError {
    Io(io::Error),
    Parse(ParseIntError),
}

impl fmt::Display for ScoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "could not read score: {error}"),
            Self::Parse(error) => write!(f, "could not parse score: {error}"),
        }
    }
}

impl Error for ScoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Parse(error) => Some(error),
        }
    }
}

impl From<io::Error> for ScoreError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<ParseIntError> for ScoreError {
    fn from(error: ParseIntError) -> Self {
        Self::Parse(error)
    }
}

/// 读取第一行，修剪首尾空白后解析为 `u32`。EOF 按空字符串解析。
pub fn read_score<R: BufRead>(mut reader: R) -> Result<u32, ScoreError> {
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let score = line.trim().parse::<u32>()?;
    Ok(score)
}

/// 用 `match` 和提早返回表达与 [`read_score`] 相同的行为。
pub fn read_score_explicit<R: BufRead>(mut reader: R) -> Result<u32, ScoreError> {
    let mut line = String::new();
    match reader.read_line(&mut line) {
        Ok(_) => {}
        Err(error) => return Err(ScoreError::from(error)),
    }
    match line.trim().parse::<u32>() {
        Ok(score) => Ok(score),
        Err(error) => Err(ScoreError::from(error)),
    }
}
