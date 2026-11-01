//! 生成不含个人数据的演示工作区，便于第一次开箱体验。
use crate::report;
use crate::scanner::{self, ScanSummary};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const SAMPLE: &[(&str, &str)] = &[
    (
        "src/main.rs",
        "fn main() { println!(\"FileQuest: hello from the demo!\"); }\n",
    ),
    (
        "src/scan.rs",
        "pub fn count_files(items: &[&str]) -> usize { items.len() }\n",
    ),
    (
        "src/config.rs",
        "pub const APP_NAME: &str = \"demo-workspace\";\n",
    ),
    (
        "docs/README.md",
        "# 演示项目\n这是 FileQuest 的安全练习目录。\n",
    ),
    (
        "docs/design.md",
        "# 产品草图\n需求：统计本地文件、按名称检索、生成离线报告。\n",
    ),
    (
        "logs/build.log",
        "INFO compile started\nWARN unused variable: state\nINFO compile completed\n",
    ),
    (
        "logs/app.log",
        "INFO application launched\nERROR cannot open example.tmp\nINFO retry success\n",
    ),
    (
        "data/metrics.csv",
        "minute,requests,errors\n0,8,0\n1,15,1\n2,19,0\n",
    ),
    (
        "config/app.toml",
        "name = \"demo-workspace\"\nlevel = \"info\"\n",
    ),
];

/// 创建全新目录；如果路径已存在则返回错误，不会向其中写入任何文件。
pub fn create_demo(root: &Path) -> io::Result<(ScanSummary, PathBuf)> {
    fs::create_dir(root)?;
    for (relative, content) in SAMPLE {
        let destination = root.join(relative);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(destination, content)?;
    }
    let result = scanner::scan(root, false)?;
    let report_path = root.join("report.html");
    report::write_report(&report_path, &report::render(&result, 400), false)?;
    Ok((result, report_path))
}
