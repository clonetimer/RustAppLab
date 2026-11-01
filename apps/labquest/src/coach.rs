//! 教学导航：读取已经验证的步骤，不擅自编译学生代码，也不代替学员完成任务。
use crate::{checks::Step, guide, simulation};
use std::path::Path;

fn quoted(path: &Path) -> String {
    format!("\"{}\"", path.display())
}

pub fn next(root: &Path, steps: &[Step]) -> String {
    let completed = steps.iter().filter(|step| step.passed).count();
    let Some(step) = steps.iter().find(|step| !step.passed) else {
        return format!(
            "CASE 01 · {completed}/6 已完成\n\n你已经交付了文件地图、事故报告、配置修复、Rust 实际运行证据和备份。\n下一步：cargo run -p labquest -- finish {}\n学习迁移：尝试 `labquest predict 400 6`，解释为什么等于 400ms 的请求不算 ERROR。\n",
            quoted(root)
        );
    };
    let quote = quoted(root);
    let (objective, concept, action) = match step.number {
        1 => {
            let output = root.join("deliverables/inventory.html");
            let force = if output.exists() { " --force" } else { "" };
            (
                "用真实文件目录生成一份可重复验收的地图。",
                "Path/PathBuf、目录遍历、结构化数据与 HTML 输出。",
                format!(
                    "cargo run -p filequest -- report {} -o {}{force}",
                    quoted(&root.join("project")),
                    quoted(&output)
                ),
            )
        }
        2 => (
            "先调查故障记录，不直接修改原始日志。",
            "迭代器、字符串解析、统计指标与证据复算。",
            format!(
                "cargo run -p loglens -- report {} -o {}{}",
                quoted(&root.join("logs/incident.log")),
                quoted(&root.join("deliverables/incident.md")),
                if root.join("deliverables/incident.md").exists() {
                    " --force"
                } else {
                    ""
                }
            ),
        ),
        3 => (
            "依据时延判断超时条件；修改配置后重新产生日志。",
            "比较运算符、配置解析、条件分支与边界值。",
            format!(
                "编辑 {}，然后执行 cargo run -p labquest -- simulate {quote}",
                quoted(&root.join("project/config/service.conf"))
            ),
        ),
        4 => (
            "用独立日志工具复核刚才的修复是否改善错误记录。",
            "软件模块复用、独立验证与报告版本一致性。",
            format!(
                "cargo run -p loglens -- report {} -o {}{}",
                quoted(&root.join("logs/current.log")),
                quoted(&root.join("deliverables/current.md")),
                if root.join("deliverables/current.md").exists() {
                    " --force"
                } else {
                    ""
                }
            ),
        ),
        5 => (
            "编写能处理不同输入的 Rust 程序，而不是输出一个固定的正确答案。",
            "借用与迭代器 iter()/filter()/count()、数据驱动的程序行为、Cargo。",
            format!(
                "编辑 {}，然后运行 cargo run -p labquest -- test-app {quote}",
                quoted(&root.join("project/src/main.rs"))
            ),
        ),
        6 => (
            "对当前代码与配置创建可校验的字节级快照，完成可复现交付。",
            "所有权与错误处理的工程延伸：校验、完整性、拒绝覆盖。",
            if root.join("backups/project-snapshot").exists() {
                "已有快照但与当前 project/ 不一致。SafeSync 禁止覆盖。请先人工核对并保留旧快照，再在新的教学案例中重做；不要让工具静默删除已有备份。".to_string()
            } else {
                format!(
                    "cargo run -p safesync -- backup {} {}",
                    quoted(&root.join("project")),
                    quoted(&root.join("backups/project-snapshot"))
                )
            },
        ),
        _ => return "当前案例步骤数据异常，请运行 status。".into(),
    };
    let first_hint = guide::hint(step.number, 1).unwrap_or("先检查输入和输出是否对应。 ");
    let warning = if step.number == 2 && step.feedback.contains("原始事故/基线日志已更改")
    {
        "\n注意：原始证据已变化。不要尝试伪造事故记录；建议新建一个案例目录重新调查。\n"
    } else {
        ""
    };
    format!(
        "CASE 01 · 当前 {completed}/6\n下一步：{}. {}\n\n目标：{objective}\n现状：{}\nRust 知识：{concept}\n推荐操作：{action}\n第一层线索：{first_hint}\n{warning}\n完成后检查：cargo run -p labquest -- status {quote}\n仍不理解时：cargo run -p labquest -- hint {} 2\n",
        step.number, step.name, step.feedback, step.number
    )
}

/// 不记录分数，不修改案例：先预测，再用相同的规则验证概念理解。
pub fn predict(timeout: u32, predicted: usize) -> Result<String, String> {
    if !(1..=5000).contains(&timeout) {
        return Err("timeout_ms 必须位于 1..=5000".into());
    }
    if predicted > simulation::LATENCIES.len() {
        return Err("预测错误记录数必须位于 0..=12".into());
    }
    let actual = simulation::LATENCIES
        .iter()
        .filter(|&&ms| ms > timeout)
        .count();
    if actual == predicted {
        Ok(format!(
            "预测正确：timeout_ms={timeout} 时，ERROR={actual}/12。\n解释：仅当 latency_ms > timeout_ms 才产生 ERROR；等于阈值不算超时。\n迁移问题：阈值改为 400ms 或 750ms 时，你的 Rust 程序仍然能算对吗？"
        ))
    } else {
        Ok(format!(
            "本次预测未命中：你预测 {predicted} 条，实际 {actual} 条（timeout_ms={timeout}）。\n检查 12 个 latency_ms，尤其是刚好等于阈值的记录。判断条件是 >，不是 >=。\n本练习不计入通关成绩，可以调整自己的解释再试。"
        ))
    }
}

pub const PRACTICE: &str = "动手前先预测（不会改变案例或通关进度）：\n  A. timeout_ms=150，12 条记录会有几条 ERROR？\n  B. timeout_ms=400，恰好 400ms 的请求算不算超时？\n  C. timeout_ms=750，最大时延 750ms 的请求算不算超时？\n\n作答：cargo run -p labquest -- predict <阈值> <预测 ERROR 条数>\n例如：cargo run -p labquest -- predict 400 5\n然后修改同一份 Rust 源码，使三种阈值都能自动得到正确结果。\n";

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn prediction_teaches_strict_boundary() {
        assert!(predict(150, 11).unwrap().contains("预测正确"));
        assert!(predict(400, 7).unwrap().contains("实际 6 条"));
        assert!(predict(750, 0).unwrap().contains("预测正确"));
        assert!(predict(0, 0).is_err());
        assert!(predict(150, 13).is_err());
    }
}
