//! 可复制到 issue / PR 的 Markdown 分析报告。
use crate::analyzer::{sanitize, Summary};
use crate::parser::Level;
use std::fmt::Write as _;
use std::fs::{self, OpenOptions};
use std::io::{self, Write as _};
use std::path::Path;

pub fn render(summary: &Summary, top: usize) -> String {
    let mut report = String::new();
    writeln!(report, "# LogLens · 日志分析报告\n").expect("写入 String 不会失败");
    writeln!(
        report,
        "日志文件：{}\n",
        escape_md(&summary.path.to_string_lossy())
    )
    .expect("写入 String 不会失败");
    writeln!(report, "| 指标 | 数值 |\n|---|---:|\n| 总行数 | {} |\n| 已识别 | {} |\n| 空行 | {} |\n| 未识别 | {} |\n| ERROR + FATAL | {} |\n| 错误率（以已识别记录为分母） | {:.2}% |\n",
        summary.lines, summary.records, summary.empty_lines, summary.skipped_lines,
        summary.errors(), summary.error_rate()).expect("写入 String 不会失败");
    report.push_str("## 级别分布\n\n| 级别 | 数量 |\n|---|---:|\n");
    for level in Level::ALL {
        writeln!(report, "| {level} | {} |", summary.count(level)).expect("写入 String 不会失败");
    }
    report.push_str("\n## 高频消息\n\n| 消息（相同文本归为一组） | 次数 |\n|---|---:|\n");
    for (message, count) in summary.top_messages(top) {
        writeln!(report, "| {} | {count} |", escape_md(message)).expect("写入 String 不会失败");
    }
    if summary.records == 0 {
        report.push_str("\n未发现可识别的日志记录。\n");
    }
    if let (Some(first), Some(last)) = (&summary.first_timestamp, &summary.last_timestamp) {
        writeln!(
            report,
            "\n时间范围（仅 UTC Z 格式记录）：{first} — {last}\n"
        )
        .expect("写入 String 不会失败");
    }
    if summary.skipped_lines > 0 {
        writeln!(
            report,
            "\n## 未识别行（共 {} 行，最多列出 5 个样例）\n",
            summary.skipped_lines
        )
        .expect("写入 String 不会失败");
        for example in &summary.examples {
            writeln!(
                report,
                "- 第 {} 行：{}；内容：{}",
                example.line_number,
                escape_md(&example.reason),
                escape_md(&example.excerpt)
            )
            .expect("写入 String 不会失败");
        }
        report.push_str("\n**注意**：由于存在未识别行，以上统计不是全量日志语义分析。\n");
    }
    report.push_str("\n---\n\n由 Rust AppLab / LogLens 离线生成。未连接远程服务。\n");
    report
}

/// 避免日志消息被解释成 Markdown 的表格列、HTML 或代码边界。
pub fn escape_md(value: &str) -> String {
    let mut output = String::new();
    for ch in sanitize(value).chars() {
        match ch {
            '\\' => output.push_str("\\\\"),
            '|' => output.push_str("\\|"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '`' => output.push_str("\\`"),
            '&' => output.push_str("&amp;"),
            '[' | ']' | '(' | ')' | '!' | '*' | '_' | '#' | '~' => {
                output.push('\\');
                output.push(ch);
            }
            _ => output.push(ch),
        }
    }
    output
}

/// 首次生成时拒绝覆盖；显式 --force 也拒绝通过符号链接写入。
pub fn write_report(path: &Path, text: &str, force: bool) -> io::Result<()> {
    if force && matches!(fs::symlink_metadata(path), Ok(meta) if !meta.file_type().is_file()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "只能覆盖普通文件，不能覆盖符号链接或目录",
        ));
    }
    let mut options = OpenOptions::new();
    options.write(true);
    if force {
        options.create(true).truncate(true);
    } else {
        options.create_new(true);
    }
    let mut file = options.open(path)?;
    file.write_all(text.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_escapes_table_cells_and_html() {
        assert_eq!(escape_md("a|<b>`x`"), "a\\|&lt;b&gt;\\`x\\`");
        assert_eq!(
            escape_md("![x](https://example.com/a.png)"),
            "\\!\\[x\\]\\(https://example.com/a.png\\)"
        );
    }
}
