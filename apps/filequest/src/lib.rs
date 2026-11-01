//! FileQuest：可实际使用的本地文件概览/检索/离线报告工具。
//! 本仓库同时把它作为 Rust 应用开发课程的第一条产品主线。
pub mod cli;
pub mod demo;
pub mod guide;
pub mod report;
pub mod scanner;

use cli::Command;
use std::ffi::OsString;

const HELP: &str = r#"FileQuest 0.1.0 · Rust AppLab 的第一个真实产品

用法（从 RustAppLab 根目录执行）：
  cargo run -- demo [新目录]                     生成演示项目及 report.html
  cargo run -- scan <目录> [--all]                按文件类型、体积扫描
  cargo run -- find <目录> <关键词> [--all]       按相对路径检索
  cargo run -- report <目录> [-o 输出.html]       生成离线交互式 HTML 报告
       [--limit 400] [--force] [--all]
  cargo run -- missions [00-07]                 查看动手任务
  cargo run -- help                              显示此帮助

安全说明：扫描与检索只读；demo 只创建新目录；报告默认拒绝覆盖已有文件。
默认跳过 .git、target、node_modules 等构建/依赖目录，--all 可扫描。
"#;

pub fn run(args: &[OsString]) -> Result<(), String> {
    match cli::parse(args)? {
        Command::Help => print!("{HELP}"),
        Command::Demo { root } => {
            let (summary, path) = demo::create_demo(&root)
                .map_err(|err| format!("无法创建演示工作区 {}：{err}", root.display()))?;
            println!("演示工作区创建成功：{}", summary.root.display());
            println!("已生成 {} 个样例文件。", summary.files.len());
            println!("离线报告：{}", path.display());
            println!("下一步：cargo run -- scan \"{}\"", root.display());
        }
        Command::Scan { root, all } => {
            let summary = scanner::scan(&root, all)
                .map_err(|err| format!("扫描 {} 失败：{err}", root.display()))?;
            print_summary(&summary);
        }
        Command::Find { root, query, all } => {
            let summary = scanner::scan(&root, all)
                .map_err(|err| format!("扫描 {} 失败：{err}", root.display()))?;
            let matches = summary.find(&query);
            println!("检索范围：{}", summary.root.display());
            println!("关键词：{query}；共找到 {} 个文件", matches.len());
            for item in matches.iter().take(100) {
                println!(
                    "  {} ({})",
                    item.relative_path.display(),
                    scanner::format_size(item.bytes)
                );
            }
            if matches.len() > 100 {
                println!("只展示前 100 项，请使用更精确的关键词。");
            }
            if !summary.problems.is_empty() {
                eprintln!(
                    "注意：{} 个访问异常，匹配结果可能不完整。",
                    summary.problems.len()
                );
            }
        }
        Command::Report {
            root,
            output,
            all,
            force,
            limit,
        } => {
            let summary = scanner::scan(&root, all)
                .map_err(|err| format!("扫描 {} 失败：{err}", root.display()))?;
            let html = report::render(&summary, limit);
            report::write_report(&output, &html, force)
                .map_err(|err| format!("保存报告 {} 失败：{err}", output.display()))?;
            println!("已保存离线报告：{}", output.display());
            println!(
                "扫描 {} 个文件；其中最多展示 {} 个。",
                summary.files.len(),
                limit
            );
            if !summary.problems.is_empty() {
                eprintln!(
                    "注意：{} 个访问异常，请在报告中检查。",
                    summary.problems.len()
                );
            }
        }
        Command::Missions { id } => guide::show(id.as_deref())?,
    }
    Ok(())
}

fn print_summary(summary: &scanner::ScanSummary) {
    println!("FileQuest · 目录扫描完成\n{}", summary.root.display());
    println!(
        "普通文件：{} | 已扫描子目录：{} | 总体积：{}",
        summary.files.len(),
        summary.directories,
        scanner::format_size(summary.total_bytes())
    );
    println!(
        "跳过构建/依赖目录：{} | 跳过符号链接：{} | 访问异常：{}",
        summary.ignored_directories,
        summary.skipped_symlinks,
        summary.problems.len()
    );
    println!("\n文件类型 TOP 8：");
    for (ext, count) in summary.extension_counts().iter().take(8) {
        println!("  {ext:<16} {count:>5} 个");
    }
    println!("\n体积 TOP 8：");
    for file in summary.largest_files(8) {
        println!(
            "  {:>10}  {}",
            scanner::format_size(file.bytes),
            file.relative_path.display()
        );
    }
    for problem in summary.problems.iter().take(12) {
        eprintln!("警告：{problem}");
    }
    if summary.problems.len() > 12 {
        eprintln!("另有 {} 条访问异常未展开。", summary.problems.len() - 12);
    }
}
