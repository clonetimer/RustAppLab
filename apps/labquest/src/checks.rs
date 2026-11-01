//! 可解释的结果验收：从四款产品的真实领域逻辑重新计算预期，不依赖手工打卡。
use crate::{case, proof, simulation};
use std::fs;
use std::path::Path;

pub struct Step {
    pub number: usize,
    pub name: &'static str,
    pub passed: bool,
    pub feedback: String,
}

fn report_matches(path: &Path, expected: &str, ignore_path_header: bool) -> bool {
    let Ok(actual) = fs::read_to_string(path) else {
        return false;
    };
    if !ignore_path_header {
        return actual == expected;
    }
    let marker = "| 指标 | 数值 |";
    match (actual.split_once(marker), expected.split_once(marker)) {
        (Some((_, left)), Some((_, right))) => left == right,
        _ => false,
    }
}

fn file_report(root: &Path) -> Result<bool, String> {
    let summary = filequest::scanner::scan(&root.join("project"), false)
        .map_err(|err| format!("扫描项目失败：{err}"))?;
    if !summary.problems.is_empty() || summary.files.is_empty() {
        return Ok(false);
    }
    let html = filequest::report::render(&summary, 400);
    Ok(report_matches(
        &root.join("deliverables/inventory.html"),
        &html,
        false,
    ))
}

fn log_report(root: &Path, filename: &str, output: &str) -> bool {
    let Ok(summary) = loglens::analyzer::analyze_file(&root.join("logs").join(filename), true)
    else {
        return false;
    };
    report_matches(
        &root.join("deliverables").join(output),
        &loglens::report::render(&summary, 10),
        true,
    )
}

fn fixtures_intact(root: &Path) -> bool {
    fs::read_to_string(root.join("logs/baseline.log"))
        .is_ok_and(|text| text == simulation::render_log(simulation::HEALTHY_TIMEOUT))
        && fs::read_to_string(root.join("logs/incident.log"))
            .is_ok_and(|text| text == simulation::render_log(simulation::INCIDENT_TIMEOUT))
}

fn repair_check(root: &Path) -> bool {
    let Ok(timeout) = simulation::read_timeout(&root.join("project")) else {
        return false;
    };
    let path = root.join("logs/current.log");
    let Ok(actual) = fs::read_to_string(&path) else {
        return false;
    };
    if actual != simulation::render_log(timeout) {
        return false;
    }
    let Ok(summary) = loglens::analyzer::analyze_file(&path, true) else {
        return false;
    };
    summary.records == 12 && summary.errors() == 0
}

fn backup_check(root: &Path) -> bool {
    let backup = root.join("backups/project-snapshot");
    if safesync::ops::verify(&backup).is_err() {
        return false;
    }
    let Ok(source) = safesync::tree::scan(&root.join("project")) else {
        return false;
    };
    let Ok(snapshot) = safesync::tree::scan(&backup.join("data")) else {
        return false;
    };
    source == snapshot
}

pub fn evaluate(root: &Path) -> Result<Vec<Step>, String> {
    case::ensure_case(root)?;
    let intact = fixtures_intact(root);
    let inventory = file_report(root).unwrap_or(false);
    let incident_report = intact && log_report(root, "incident.log", "incident.md");
    let repaired = intact && repair_check(root);
    let current_report = repaired && log_report(root, "current.log", "current.md");
    let built_app = repaired && proof::check(root);
    let snapshot = built_app && backup_check(root);
    Ok(vec![
        Step { number: 1, name: "文件现场地图", passed: inventory,
            feedback: if inventory { "HTML 报告与当前 project/ 文件结构一致" } else { "用 FileQuest 生成或更新 inventory.html；修改配置或源码后要重新生成" }.into() },
        Step { number: 2, name: "事故日志取证", passed: incident_report,
            feedback: if incident_report { "事故日志和 Markdown 统计报告可复算" } else if !intact { "原始事故/基线日志已更改，请重新创建新案例" } else { "用 LogLens 为 incident.log 生成 incident.md（--top 默认 10）" }.into() },
        Step { number: 3, name: "配置修复与复跑", passed: repaired,
            feedback: if repaired { "模拟当前日志与配置一致；12 条请求均未超时" } else { "根据日志时延检查 service.conf，再运行 labquest simulate；只改日志不能通关" }.into() },
        Step { number: 4, name: "修复后独立复验", passed: current_report,
            feedback: if current_report { "当前日志报告与修复结果一致" } else { "修复完成后，用 LogLens 为 current.log 生成或更新 current.md" }.into() },
        Step { number: 5, name: "亲手改 Rust 并运行", passed: built_app,
            feedback: if built_app { "独立 Rust 程序已真实运行；摘要、日志与当前 project/ 文件指纹一致" } else { "在 project/src/main.rs 实现动态 SUMMARY，再执行 labquest test-app；源码或配置变化后须重跑" }.into() },
        Step { number: 6, name: "安全封存现场", passed: snapshot,
            feedback: if snapshot { "SafeSync 快照校验通过，内容与当前 project/ 一致" } else { "先通过 Rust 代码关卡，再用 SafeSync 备份最新 project/ 到 backups/project-snapshot" }.into() },
    ])
}

pub fn all_complete(steps: &[Step]) -> bool {
    steps.iter().all(|step| step.passed)
}
