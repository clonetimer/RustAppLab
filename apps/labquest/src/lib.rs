//! LabQuest：在具象案例中复用四款应用，交付可复算的学习成果。
pub mod advanced;
pub mod advanced_b;
mod advanced_b_json;
pub mod beginner;
pub mod board;
mod book_reading;
pub mod capstone;
pub mod case;
pub mod checks;
pub mod coach;
pub mod ecosystem;
pub mod guide;
pub mod independent;
pub mod learning;
mod learning_cards;
mod process;
pub mod proof;
pub mod simulation;
pub mod studio;
pub mod transfer;

use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

fn path(item: Option<&OsString>, usage: &str) -> Result<PathBuf, String> {
    item.map(PathBuf::from).ok_or_else(|| usage.to_string())
}

pub fn run(args: &[OsString]) -> Result<(), String> {
    match args.first().and_then(|part| part.to_str()) {
        Some("--version" | "version") if args.len() == 1 => {
            println!("Rust AppLab / LabQuest {}", env!("CARGO_PKG_VERSION"))
        }
        Some("advanced") => advanced::run(&args[1..])?,
        Some("advanced-b") => advanced_b::run(&args[1..])?,
        Some("studio") => studio::run(&args[1..])?,
        Some("learn") => learning::run(&args[1..])?,
        Some("ecosystem") => ecosystem::run(&args[1..])?,
        Some("beginner") if args.len() == 3 && args[1] == "start" => {
            let root = path(args.get(2), "用法：beginner start <全新目录>")?;
            beginner::init(&root)?;
        }
        Some("beginner") if args.len() == 3 && args[1] == "check" => {
            let root = path(args.get(2), "用法：beginner check <学习目录>")?;
            if !beginner::check(&root)? {
                return Err(
                    "第一张任务卡还没通过；请修改 src/main.rs 后再试。你可以使用 beginner hint 1。"
                        .into(),
                );
            }
        }
        Some("beginner") if args.len() == 3 && args[1] == "hint" => {
            let tier = args[2]
                .to_str()
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
            println!("{}", beginner::hint(tier)?);
        }

        None | Some("help" | "--help" | "-h") if args.len() <= 1 => {
            print!("{}", guide::HELP);
        }
        Some("app") if args.len() == 2 && args[1] == "catalog" => {
            println!("独立作品任务目录：\n  taskpocket — TSV 任务管理器（状态持久化）\n  pagemint   — 离线笔记网页生成器（多文件读取/安全渲染）\n用法：app init <项目名> <新目录>、app check <项目名> <工程目录>、app hint <项目名> <1|2|3>\n0.9.0 的 app init/check <目录> 继续默认 TaskPocket。");
        }
        Some("app") if args.len() == 5 && args[1] == "checkpoint" => {
            let root = path(
                args.get(3),
                "app checkpoint <taskpocket|pagemint> <目录> <1|2|3>",
            )?;
            let id = args[2]
                .to_str()
                .ok_or("项目名必须是 taskpocket 或 pagemint")?;
            let stage = args[4]
                .to_str()
                .and_then(|s| s.parse::<usize>().ok())
                .ok_or("检查点仅支持 1、2、3")?;
            if !independent::check(&root, id, stage)? {
                return Err("当前独立开发检查点未通过；请读取新报告，修改后重试".into());
            }
        }
        Some("app") if args.len() == 4 && args[1] == "init" => {
            let root = path(args.get(3), "用法：app init <taskpocket|pagemint> <新目录>")?;
            match args[2].to_str() {
                Some("taskpocket") => capstone::init(&root)?,
                Some("pagemint") => transfer::init(&root)?,
                _ => return Err("未知独立项目，请使用 app catalog 查看列表".into()),
            }
            println!(
                "工程已创建：{}；请先打开 START.html、阅读 CONTRACT.md，再设计和编辑自己的代码。",
                root.display()
            );
        }
        Some("app") if args.len() == 4 && args[1] == "check" => {
            let root = path(
                args.get(3),
                "用法：app check <taskpocket|pagemint> <工程目录>",
            )?;
            let passed = match args[2].to_str() {
                Some("taskpocket") => capstone::check(&root)?,
                Some("pagemint") => transfer::check(&root)?,
                _ => return Err("未知独立项目，请使用 app catalog 查看列表".into()),
            };
            if !passed {
                return Err("尚未达标，请按 APP_ASSESSMENT.md 第一条 FAIL 逐项修复".into());
            }
            println!("所有基础验收通过。下一步：用自己的数据使用软件，并填写 PROJECT_JOURNAL.md。");
        }
        Some("app") if args.len() == 4 && args[1] == "hint" => {
            let tier = args[3]
                .to_str()
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
            let clue = match args[2].to_str() {
                Some("taskpocket") => capstone::hint(tier)?,
                Some("pagemint") => transfer::hint(tier)?,
                _ => return Err("未知独立项目，请使用 app catalog 查看列表".into()),
            };
            println!("{clue}");
        }
        Some("app") if args.len() == 3 && args[1] == "init" => {
            let root = path(args.get(2), "用法：app init <全新 TaskPocket 项目目录>")?;
            capstone::init(&root)?;
            println!("独立 Rust 工程已创建：{}", root.display());
            println!("阅读 CONTRACT.md，再编辑 src/main.rs；你可先 cargo run -- help 体验入口。");
        }
        Some("app") if args.len() == 3 && args[1] == "check" => {
            let root = path(args.get(2), "用法：app check <TaskPocket 项目目录>")?;
            if !capstone::check(&root)? {
                return Err(
                    "TaskPocket 尚未全部达标，按 APP_ASSESSMENT.md 的第一条失败反馈修改".into(),
                );
            }
            println!("TaskPocket 9 项黑盒功能验收通过；请填写开发记录并实际使用自己的工具。");
        }
        Some("app") if args.len() == 3 && args[1] == "hint" => {
            let tier = args[2]
                .to_str()
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
            println!("{}", capstone::hint(tier)?);
        }
        Some("start") if args.len() == 2 => {
            let root = path(args.get(1), "用法：start <新案例目录>")?;
            case::start(&root)?;
            println!("CASE 01 已建立：{}", root.display());
            println!(
                "先阅读 {}/BRIEFING.md，再查看第一步提示：labquest hint 1",
                root.display()
            );
            println!(
                "当前进度：0/6；使用 labquest next \"{}\" 读取下一步反馈。",
                root.display()
            );
        }
        Some("simulate") if args.len() == 2 => {
            let root = path(args.get(1), "用法：simulate <案例目录>")?;
            let (timeout, errors) = case::simulate(&root)?;
            println!("已重跑 12 条确定性模拟请求：timeout_ms={timeout}，ERROR={errors}");
            println!(
                "输出：{}/logs/current.log；可以用 LogLens compare 分析变化。",
                root.display()
            );
        }
        Some("next") if args.len() == 2 => {
            let root = path(args.get(1), "用法：next <案例目录>")?;
            let steps = checks::evaluate(&root)?;
            print!("{}", coach::next(&root, &steps));
        }
        Some("practice") if args.len() == 1 => {
            print!("{}", coach::PRACTICE);
        }
        Some("predict") if args.len() == 3 => {
            let timeout = args[1]
                .to_str()
                .and_then(|x| x.parse::<u32>().ok())
                .ok_or("用法：predict <timeout_ms> <预测的 ERROR 条数>")?;
            let predicted = args[2]
                .to_str()
                .and_then(|x| x.parse::<usize>().ok())
                .ok_or("用法：predict <timeout_ms> <预测的 ERROR 条数>")?;
            println!("{}", coach::predict(timeout, predicted)?);
        }
        Some("status") if args.len() == 2 => {
            let root = path(args.get(1), "用法：status <案例目录>")?;
            let results = checks::evaluate(&root)?;
            let passed = results.iter().filter(|step| step.passed).count();
            println!("CASE 01 · 事故调查进度：{passed}/6\n");
            for step in &results {
                let state = if step.passed { "[OK]" } else { "[  ]" };
                println!(
                    "{state} {}. {}\n     {}",
                    step.number, step.name, step.feedback
                );
            }
            if passed == 6 {
                println!(
                    "\n全部验收通过。下一步：labquest finish \"{}\"",
                    root.display()
                );
            } else if let Some(next) = results.iter().find(|step| !step.passed) {
                println!("\n下一条线索：labquest hint {}", next.number);
            }
        }
        Some("test-app") if args.len() == 2 => {
            let root = path(args.get(1), "用法：test-app <案例目录>")?;
            proof::test_app(&root)?;
            println!(
                "独立 Rust 项目编译运行通过：输出 12 条模拟日志和 SUMMARY，并完成新案例的多阈值迁移验收（旧版案例保持单配置兼容）。
已记录 logs/program.log 与 deliverables/PROGRAM_PROOF.txt。
请重新生成 FileQuest 文件地图并在最后使用 SafeSync 备份最新 project/。"
            );
        }
        Some("board") if args.len() == 2 => {
            let root = path(args.get(1), "用法：board <案例目录>")?;
            let steps = checks::evaluate(&root)?;
            let output = board::write(&root, &steps)?;
            println!(
                "离线任务进度板：{}（每次任务完成后重新执行 board 更新）",
                output.display()
            );
        }
        Some("hint") if args.len() == 2 || args.len() == 3 => {
            let step = args[1]
                .to_str()
                .and_then(|x| x.parse::<usize>().ok())
                .unwrap_or(0);
            let tier = args
                .get(2)
                .map_or(Some(1), |arg| {
                    arg.to_str().and_then(|x| x.parse::<usize>().ok())
                })
                .unwrap_or(0);
            println!("线索 {step} / 第 {tier} 层：\n{}", guide::hint(step, tier)?);
        }
        Some("finish") if args.len() == 2 => {
            let root = path(args.get(1), "用法：finish <案例目录>")?;
            let target = finish(&root)?;
            println!(
                "调查结束：六步证据均有效。\n作品已生成：{}",
                target.display()
            );
        }
        _ => return Err("命令或参数数量不正确。请运行 labquest help 查看用法".into()),
    }
    Ok(())
}

pub fn finish(root: &Path) -> Result<PathBuf, String> {
    let results = checks::evaluate(root)?;
    if !checks::all_complete(&results) {
        let unmet: Vec<_> = results
            .iter()
            .filter(|s| !s.passed)
            .map(|s| s.number.to_string())
            .collect();
        return Err(format!(
            "仍有未完成的验收步骤：{}。先运行 status 查看反馈",
            unmet.join(", ")
        ));
    }
    let previous = loglens::analyzer::analyze_file(&root.join("logs/incident.log"), true)?;
    let current = loglens::analyzer::analyze_file(&root.join("logs/current.log"), true)?;
    let baseline = loglens::analyzer::analyze_file(&root.join("logs/baseline.log"), true)?;
    let timeout = simulation::read_timeout(&root.join("project"))?;
    let inventory = filequest::scanner::scan(&root.join("project"), false)
        .map_err(|err| format!("无法复算项目文件：{err}"))?;
    let backup = safesync::ops::verify(&root.join("backups/project-snapshot"))
        .map_err(|err| format!("无法复验备份：{err}"))?;
    let report = format!("# CASE 01 · 网关超时事故调查包\n\n> Rust AppLab / LabQuest 0.8.0 · **确定性教学模拟，不代表真实生产事故**\n\n## 最终结论\n\n变更后模拟日志的错误记录占比为 **{:.2}%**（{}/{}），在将配置中的 timeout_ms 调整为 **{}** 后，当前日志错误记录占比为 **{:.2}%**（{}/{}），下降 **{:.2} 个百分点**。\n\n| 阶段 | 可识别记录 | ERROR+FATAL | 错误记录占比 |\n|---|---:|---:|---:|\n| 基线 | {} | {} | {:.2}% |\n| 故障 | {} | {} | {:.2}% |\n| 修复后 | {} | {} | {:.2}% |\n\n## 可独立检查的交付证据\n\n- [FileQuest 文件地图](inventory.html)：当前项目 {} 个普通文件，总字节 {}。\n- [LogLens 故障分析](incident.md)：保留修复前的可复算记录。\n- [LogLens 修复后分析](current.md)：对应 current.log 的统计。\n- [Rust 程序真实运行证据](PROGRAM_PROOF.txt)：在显式 test-app 后由 Cargo 编译执行，结果与最新源码指纹及配置一致；原始 12 条请求输出见 `../logs/program.log`。\n- `../backups/project-snapshot/`：SafeSync 校验通过，{} 个文件、{} 字节，且快照与当前 project/ 内容匹配。\n- `../project/`：独立 Rust 小应用源码及服务配置（可另行用 Cargo 构建）。\n\n## 判定边界\n\n本案 12 条请求时延是固定样例。错误记录占比不是实际服务可用率；修改超时阈值只说明此模拟下错误减少，并不能证明真实系统已解决性能根因。本报告仅在生成时复验文件，后续如再次改动项目或日志，必须重新运行 status。\n",
        previous.error_rate(), previous.errors(), previous.records, timeout,
        current.error_rate(), current.errors(), current.records,
        previous.error_rate() - current.error_rate(),
        baseline.records, baseline.errors(), baseline.error_rate(),
        previous.records, previous.errors(), previous.error_rate(),
        current.records, current.errors(), current.error_rate(),
        inventory.files.len(), inventory.total_bytes(), backup.files, backup.bytes
    );
    let target = root.join("deliverables/CASE_REPORT.md");
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&target)
        .map_err(|err| format!("拒绝覆盖已有调查报告 {}：{err}", target.display()))?;
    output
        .write_all(report.as_bytes())
        .map_err(|err| format!("写入调查报告失败：{err}"))?;
    Ok(target)
}
