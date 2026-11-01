//! 按行流式读取日志。将分析和 CLI/报告分开，便于精确测试。
use crate::parser::{parse_line, Level};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ParseProblem {
    pub line_number: usize,
    pub reason: String,
    pub excerpt: String,
}

#[derive(Debug)]
pub struct Summary {
    pub path: PathBuf,
    pub lines: u64,
    pub records: u64,
    pub empty_lines: u64,
    pub skipped_lines: u64,
    pub levels: BTreeMap<Level, u64>,
    /// 只保存每种不同的消息及出现次数，不把整个日志加载进内存。
    /// 极高基数日志仍会占用大量内存，后续专题引入有界聚合。
    pub messages: BTreeMap<String, u64>,
    pub first_timestamp: Option<String>,
    pub last_timestamp: Option<String>,
    pub examples: Vec<ParseProblem>,
}

impl Summary {
    fn new(path: &Path) -> Self {
        Self {
            path: path.to_owned(),
            lines: 0,
            records: 0,
            empty_lines: 0,
            skipped_lines: 0,
            levels: BTreeMap::new(),
            messages: BTreeMap::new(),
            first_timestamp: None,
            last_timestamp: None,
            examples: Vec::new(),
        }
    }

    pub fn count(&self, level: Level) -> u64 {
        self.levels.get(&level).copied().unwrap_or(0)
    }

    pub fn errors(&self) -> u64 {
        self.count(Level::Error) + self.count(Level::Fatal)
    }

    pub fn error_rate(&self) -> f64 {
        if self.records == 0 {
            return 0.0;
        }
        self.errors() as f64 / self.records as f64 * 100.0
    }

    pub fn top_messages(&self, limit: usize) -> Vec<(&str, u64)> {
        let mut items: Vec<_> = self
            .messages
            .iter()
            .map(|(msg, count)| (msg.as_str(), *count))
            .collect();
        items.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
        items.truncate(limit);
        items
    }

    fn record_timestamp(&mut self, timestamp: String) {
        if match &self.first_timestamp {
            Some(min) => &timestamp < min,
            None => true,
        } {
            self.first_timestamp = Some(timestamp.clone());
        }
        if match &self.last_timestamp {
            Some(max) => &timestamp > max,
            None => true,
        } {
            self.last_timestamp = Some(timestamp);
        }
    }
}

/// `strict=false` 保留识别的记录并报告无法识别的行；
/// `strict=true` 第一条异常行即报错，避免被误判为完整分析。
pub fn analyze_file(path: &Path, strict: bool) -> Result<Summary, String> {
    let file = File::open(path).map_err(|err| format!("无法读取 {}：{err}", path.display()))?;
    let meta = file
        .metadata()
        .map_err(|err| format!("无法获取 {} 元数据：{err}", path.display()))?;
    if !meta.is_file() {
        return Err(format!("输入必须是普通日志文件：{}", path.display()));
    }
    let reader = BufReader::new(file);
    let mut summary = Summary::new(path);
    for (index, item) in reader.lines().enumerate() {
        let line_number = index + 1;
        let line = item.map_err(|err| line_error(path, line_number, &err))?;
        summary.lines += 1;
        match parse_line(&line) {
            Ok(Some(record)) => {
                summary.records += 1;
                *summary.levels.entry(record.level).or_default() += 1;
                *summary.messages.entry(record.message).or_default() += 1;
                if let Some(timestamp) = record.timestamp {
                    summary.record_timestamp(timestamp);
                }
            }
            Ok(None) => summary.empty_lines += 1,
            Err(reason) => {
                if strict {
                    return Err(format!(
                        "{}:{}：{reason}（使用默认模式可跳过并报告无法识别的行）",
                        path.display(),
                        line_number
                    ));
                }
                summary.skipped_lines += 1;
                if summary.examples.len() < 5 {
                    summary.examples.push(ParseProblem {
                        line_number,
                        reason: reason.to_owned(),
                        excerpt: sanitize(&line).chars().take(100).collect(),
                    });
                }
            }
        }
    }
    Ok(summary)
}

fn line_error(path: &Path, line_number: usize, err: &io::Error) -> String {
    format!("读取 {} 第 {line_number} 行失败：{err}", path.display())
}

/// 不让日志正文中的控制字符污染 CLI 输出。
pub fn sanitize(text: &str) -> String {
    text.chars()
        .map(|ch| if ch.is_control() { '�' } else { ch })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_for_empty_log_is_defined() {
        let summary = Summary::new(Path::new("empty.log"));
        assert_eq!(summary.error_rate(), 0.0);
        assert!(summary.top_messages(5).is_empty());
    }

    #[test]
    fn sanitize_replaces_terminal_control_sequences() {
        assert_eq!(sanitize("error\x1b[31m"), "error�[31m");
    }
}
