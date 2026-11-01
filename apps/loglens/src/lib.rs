//! Rust AppLab 第二款产品：读取本地日志、比较变更、输出报告。
pub mod analyzer;
pub mod cli;
pub mod demo;
pub mod guide;
pub mod parser;
pub mod report;

use analyzer::{sanitize, Summary};
use cli::Command;
use parser::Level;
use std::ffi::OsString;
use std::fs;

const HELP: &str = r#"LogLens 0.2.0 · 本地日志分析器

从 RustAppLab 仓库根目录运行：
  cargo run -p loglens -- demo [新目录]
  cargo run -p loglens -- analyze <日志文件> [--strict]
  cargo run -p loglens -- compare <旧日志> <新日志> [--strict]
  cargo run -p loglens -- report <日志文件> [-o 输出.md] [--top 10] [--force] [--strict]
  cargo run -p loglens -- missions [08-11]
  cargo run -p loglens -- help

支持：`LEVEL 消息` 或 `YYYY-MM-DDTHH:MM:SSZ LEVEL 消息`
级别：TRACE / DEBUG / INFO / WARN / ERROR / FATAL（支持 WARNING、ERR、CRITICAL 别名）。
默认跳过无法识别的行，但明确告知数量；--strict 在首条异常行报错。
分析/比较只读，不发送日志。报告默认拒绝覆盖已有文件。
"#;

pub fn run(args: &[OsString]) -> Result<(), String> {
    match cli::parse(args)? {
        Command::Help => print!("{HELP}"),
        Command::Demo { root } => {
            let (baseline, candidate) = demo::create_demo(&root)
                .map_err(|err| format!("无法创建演示目录 {}：{err}", root.display()))?;
            println!(
                "已生成两个练习日志：\n  {}\n  {}",
                baseline.display(),
                candidate.display()
            );
            println!(
                "下一步：cargo run -p loglens -- compare \"{}\" \"{}\"",
                baseline.display(),
                candidate.display()
            );
        }
        Command::Analyze { input, strict } => {
            let summary = analyzer::analyze_file(&input, strict)?;
            print_summary(&summary);
        }
        Command::Compare {
            before,
            after,
            strict,
        } => {
            let previous = analyzer::analyze_file(&before, strict)?;
            let current = analyzer::analyze_file(&after, strict)?;
            println!("LogLens · 两份日志的错误率比较");
            println!("旧日志：{}", previous.path.display());
            println!("新日志：{}", current.path.display());
            println!("             旧日志          新日志");
            println!(
                "识别记录     {:>8}       {:>8}",
                previous.records, current.records
            );
            println!(
                "ERROR+FATAL  {:>8}       {:>8}",
                previous.errors(),
                current.errors()
            );
            println!(
                "错误率       {:>7.2}%       {:>7.2}%",
                previous.error_rate(),
                current.error_rate()
            );
            println!(
                "错误率变化：{:+.2} 个百分点",
                current.error_rate() - previous.error_rate()
            );
            println!(
                "未识别行：旧 {} 行，新 {} 行",
                previous.skipped_lines, current.skipped_lines
            );
            if previous.records == 0 || current.records == 0 {
                eprintln!("注意：至少一份日志没有有效记录，不能据此判断质量改善或退化。");
            }
            if previous.skipped_lines > 0 || current.skipped_lines > 0 {
                eprintln!("注意：存在未识别日志行，以上错误率仅针对已识别记录。");
            }
            println!("注意：仅比较记录级错误占比，不推断故障原因；两份日志需具有可比性。");
        }
        Command::Report {
            input,
            output,
            strict,
            force,
            top,
        } => {
            // 防止 --force 把刚刚分析的原始日志截断成 Markdown 报告。
            if let (Ok(source), Ok(destination)) =
                (fs::canonicalize(&input), fs::canonicalize(&output))
            {
                if source == destination {
                    return Err("报告不能写回输入日志自身，请提供不同的 -o 路径".into());
                }
            }
            let summary = analyzer::analyze_file(&input, strict)?;
            let md = report::render(&summary, top);
            report::write_report(&output, &md, force)
                .map_err(|err| format!("写入报告 {} 失败：{err}", output.display()))?;
            println!(
                "报告已保存：{}（已识别 {} 行；未识别 {} 行）",
                output.display(),
                summary.records,
                summary.skipped_lines
            );
        }
        Command::Missions { id } => guide::show(id.as_deref())?,
    }
    Ok(())
}

fn print_summary(summary: &Summary) {
    println!("LogLens · {}", summary.path.display());
    println!(
        "总行数：{} | 已识别：{} | 空行：{} | 未识别：{}",
        summary.lines, summary.records, summary.empty_lines, summary.skipped_lines
    );
    for level in Level::ALL {
        println!("  {level:<5}：{}", summary.count(level));
    }
    println!(
        "ERROR + FATAL：{} / {}（{:.2}%）",
        summary.errors(),
        summary.records,
        summary.error_rate()
    );
    if let (Some(first), Some(last)) = (&summary.first_timestamp, &summary.last_timestamp) {
        println!("UTC 时间范围：{first} — {last}");
    }
    println!("高频消息 TOP 5：");
    for (message, count) in summary.top_messages(5) {
        println!("  {count:>5} 次 · {}", sanitize(message));
    }
    for problem in &summary.examples {
        eprintln!(
            "未识别第 {} 行：{}；{}",
            problem.line_number, problem.reason, problem.excerpt
        );
    }
    if summary.skipped_lines > 0 {
        eprintln!(
            "注意：有 {} 行无法识别，本结果不代表原文件的全部日志事件。",
            summary.skipped_lines
        );
    }
}
