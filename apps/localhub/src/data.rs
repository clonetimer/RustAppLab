//! 聚合两款独立应用的只读快照。浏览器不得提交文件路径。
use filequest::scanner::{self, ScanSummary};
use loglens::analyzer::{self, Summary};
use loglens::parser::Level;
use std::fmt::Write as _;
use std::path::Path;

pub struct Snapshot {
    pub files: ScanSummary,
    pub logs: Option<Summary>,
}

impl Snapshot {
    pub fn load(root: &Path, log: Option<&Path>) -> Result<Self, String> {
        let files = scanner::scan(root, false)
            .map_err(|err| format!("无法扫描指定的工作目录 {}：{err}", root.display()))?;
        let logs = log
            .map(|path| analyzer::analyze_file(path, false))
            .transpose()?;
        Ok(Self { files, logs })
    }

    pub fn files_json(&self, limit: usize) -> String {
        let mut output = String::new();
        let paths = &self.files.files;
        write!(
            output,
            "{{\"root\":{},\"stats\":{{\"files\":{},\"directories\":{},\"ignored_directories\":{},\"skipped_symlinks\":{},\"problems\":{},\"total_bytes\":{}}},\"shown\":{},\"truncated\":{},\"extensions\":[",
            json_string(&self.files.root.to_string_lossy()),
            paths.len(),
            self.files.directories,
            self.files.ignored_directories,
            self.files.skipped_symlinks,
            self.files.problems.len(),
            self.files.total_bytes(),
            paths.len().min(limit),
            paths.len() > limit,
        ).expect("String 写入不会失败");
        for (index, (extension, count)) in self.files.extension_counts().iter().take(12).enumerate()
        {
            if index > 0 {
                output.push(',');
            }
            write!(
                output,
                "{{\"extension\":{},\"count\":{count}}}",
                json_string(extension)
            )
            .expect("String 写入不会失败");
        }
        output.push_str("],\"files\":[");
        for (index, file) in paths.iter().take(limit).enumerate() {
            if index > 0 {
                output.push(',');
            }
            write!(
                output,
                "{{\"path\":{},\"bytes\":{},\"extension\":{}}}",
                json_string(&file.relative_path.to_string_lossy()),
                file.bytes,
                json_string(&file.extension),
            )
            .expect("String 写入不会失败");
        }
        output.push_str("]}");
        output
    }

    pub fn logs_json(&self) -> String {
        let Some(summary) = &self.logs else {
            return "{\"enabled\":false}".to_owned();
        };
        let mut output = String::new();
        write!(
            output,
            "{{\"enabled\":true,\"input\":{},\"lines\":{},\"records\":{},\"skipped\":{},\"errors\":{},\"error_rate\":{:.2},\"levels\":[",
            json_string(&summary.path.to_string_lossy()),
            summary.lines,
            summary.records,
            summary.skipped_lines,
            summary.errors(),
            summary.error_rate(),
        ).expect("String 写入不会失败");
        for (index, level) in Level::ALL.iter().enumerate() {
            if index > 0 {
                output.push(',');
            }
            write!(
                output,
                "{{\"level\":{},\"count\":{}}}",
                json_string(&level.to_string()),
                summary.count(*level)
            )
            .expect("String 写入不会失败");
        }
        output.push_str("],\"top_messages\":[");
        for (index, (message, count)) in summary.top_messages(5).iter().enumerate() {
            if index > 0 {
                output.push(',');
            }
            write!(
                output,
                "{{\"message\":{},\"count\":{count}}}",
                json_string(message)
            )
            .expect("String 写入不会失败");
        }
        output.push_str("]}");
        output
    }
}

/// 最小 JSON 字符串编码器：教学项目不依赖 serde，所有字符串必须通过此函数。
/// JSON 输出只作为 application/json 返回；页面通过 textContent 而非 innerHTML 展示。
pub fn json_string(value: &str) -> String {
    let mut output = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            ch if ch <= '\u{001f}' => {
                write!(output, "\\u{:04x}", ch as u32).expect("String 写入不会失败");
            }
            _ => output.push(ch),
        }
    }
    output.push('"');
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_special_json_characters() {
        assert_eq!(
            json_string("x\"\\\n\u{0000}中文"),
            "\"x\\\"\\\\\\n\\u0000中文\""
        );
        assert_eq!(json_string("<script>"), "\"<script>\"");
    }
}
