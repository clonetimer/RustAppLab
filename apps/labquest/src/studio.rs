//! Studio: twelve progressive, independently runnable learner projects.
//! Copies the learner's src into a fresh evaluation workspace; tests and manifest are trusted assets.
//! Explicitly runs user code: this is a teaching checker, NOT a security sandbox.
use crate::capstone::{run_bounded, short, TempWorkspace};
use safesync::sha256::{hex, Sha256};
use std::{
    ffi::OsString,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub struct Product {
    pub id: &'static str,
    pub title: &'static str,
    pub steps: [&'static str; 3],
    starter: &'static str,
    main: &'static str,
    tests: &'static str,
    lesson: &'static str,
    page: &'static str,
    hints: &'static str,
    samples: &'static [(&'static str, &'static str)],
    support: &'static [(&'static str, &'static str)],
}

pub static PRODUCTS: &[Product] = &[
    Product {
        id: "unitlab",
        title: "UnitLab · 随身单位换算器",
        steps: [
            "写出温度转换公式",
            "用 enum 和 match 区分单位",
            "连接 parse、Result 与格式化输出",
        ],
        starter: include_str!("../studio/unitlab_starter.rs"),
        main: include_str!("../../unitlab/src/main.rs"),
        tests: include_str!("../../unitlab/tests/milestones.rs"),
        lesson: include_str!("../../../course/studio/unitlab.md"),
        page: include_str!("../studio/unitlab_lesson.html"),
        hints: include_str!("../studio/unitlab_hints.md"),
        support: &[],
        samples: &[],
    },
    Product {
        id: "quizdeck",
        title: "QuizDeck · 终端知识闯关",
        steps: [
            "比较答案并处理空白",
            "把文本解析为拥有数据的结构体",
            "用切片、迭代器生成复习清单",
        ],
        starter: include_str!("../studio/quizdeck_starter.rs"),
        main: include_str!("../../quizdeck/src/main.rs"),
        tests: include_str!("../../quizdeck/tests/milestones.rs"),
        lesson: include_str!("../../../course/studio/quizdeck.md"),
        page: include_str!("../studio/quizdeck_lesson.html"),
        hints: include_str!("../studio/quizdeck_hints.md"),
        support: &[],
        samples: &[
            ("deck.tsv", include_str!("../../quizdeck/samples/deck.tsv")),
            (
                "answers.txt",
                include_str!("../../quizdeck/samples/answers.txt"),
            ),
        ],
    },
    Product {
        id: "textscope",
        title: "TextScope · 文本观察镜",
        steps: [
            "分清字节、字符、行和词元",
            "统计词频并建立确定性排序",
            "返回借用原文的搜索结果",
        ],
        starter: include_str!("../studio/textscope_starter.rs"),
        main: include_str!("../../textscope/src/main.rs"),
        tests: include_str!("../../textscope/tests/milestones.rs"),
        lesson: include_str!("../../../course/studio/textscope.md"),
        page: include_str!("../studio/textscope_lesson.html"),
        hints: include_str!("../studio/textscope_hints.md"),
        support: &[],
        samples: &[(
            "notes.txt",
            include_str!("../../textscope/samples/notes.txt"),
        )],
    },
    Product {
        id: "ledgerlite",
        title: "LedgerLite · 本地生活账本",
        steps: [
            "用整数分精确表示金额",
            "解析并验证可持久化记录",
            "借用记录并按分类累计",
        ],
        starter: include_str!("../studio/ledgerlite_starter.rs"),
        main: include_str!("../../ledgerlite/src/main.rs"),
        tests: include_str!("../../ledgerlite/tests/milestones.rs"),
        lesson: include_str!("../../../course/studio/ledgerlite.md"),
        page: include_str!("../studio/ledgerlite_lesson.html"),
        hints: include_str!("../studio/ledgerlite_hints.md"),
        support: &[],
        samples: &[(
            "expenses.tsv",
            include_str!("../../ledgerlite/samples/expenses.tsv"),
        )],
    },
    Product {
        id: "badgeforge",
        title: "BadgeForge · SVG 作品卡生成器",
        steps: [
            "正确转义文本而不破坏中文",
            "为作品卡实现 Element trait",
            "用泛型组合独立 SVG 文档",
        ],
        starter: include_str!("../studio/badgeforge_starter.rs"),
        main: include_str!("../../badgeforge/src/main.rs"),
        tests: include_str!("../../badgeforge/tests/milestones.rs"),
        lesson: include_str!("../../../course/studio/badgeforge.md"),
        page: include_str!("../studio/badgeforge_lesson.html"),
        hints: include_str!("../studio/badgeforge_hints.md"),
        support: &[],
        samples: &[],
    },
    Product {
        id: "batchmeter",
        title: "BatchMeter · 多文件并行统计台",
        steps: [
            "读取真实文件并明确失败",
            "用结构体累计批处理结果",
            "用作用域线程与通道保持结果有序",
        ],
        starter: include_str!("../studio/batchmeter_starter.rs"),
        main: include_str!("../../batchmeter/src/main.rs"),
        tests: include_str!("../../batchmeter/tests/milestones.rs"),
        lesson: include_str!("../../../course/studio/batchmeter.md"),
        page: include_str!("../studio/batchmeter_lesson.html"),
        hints: include_str!("../studio/batchmeter_hints.md"),
        support: &[],
        samples: &[
            ("one.txt", include_str!("../../batchmeter/samples/one.txt")),
            ("two.txt", include_str!("../../batchmeter/samples/two.txt")),
        ],
    },
    Product {
        id: "termgarden",
        title: "TermGarden · 终端任务花园",
        steps: [
            "把用户命令解析成事件",
            "让任务状态与光标正确变化",
            "借用状态绘制界面并保存作品",
        ],
        starter: include_str!("../studio/termgarden_starter.rs"),
        main: include_str!("../../termgarden/src/main.rs"),
        tests: include_str!("../../termgarden/tests/milestones.rs"),
        lesson: include_str!("../../../course/studio/termgarden.md"),
        page: include_str!("../studio/termgarden_lesson.html"),
        hints: include_str!("../studio/termgarden_hints.md"),
        support: &[(
            "storage.rs",
            include_str!("../../termgarden/src/storage.rs"),
        )],
        samples: &[(
            "session.txt",
            include_str!("../../termgarden/samples/session.txt"),
        )],
    },
    Product {
        id: "storedesk",
        title: "StoreDesk · SQLite 物品库存台",
        steps: [
            "验证物品而不写入数据库",
            "用参数化 SQL 保存与查询",
            "让批量调整与流水一起提交或回滚",
        ],
        starter: include_str!("../studio/storedesk_starter.rs"),
        main: include_str!("../../storedesk/src/main.rs"),
        tests: include_str!("../../storedesk/tests/milestones.rs"),
        lesson: include_str!("../../../course/studio/storedesk.md"),
        page: include_str!("../studio/storedesk_lesson.html"),
        hints: include_str!("../studio/storedesk_hints.md"),
        support: &[("sqlite.rs", include_str!("../../storedesk/src/sqlite.rs"))],
        samples: &[(
            "changes.tsv",
            include_str!("../../storedesk/samples/changes.tsv"),
        )],
    },
    Product {
        id: "netfetch",
        title: "NetFetch · 本机 HTTP 下载器",
        steps: [
            "校验 URL，只允许安全的本机 HTTP 地址",
            "读懂 HTTP 状态、头部和响应长度",
            "连接真实 TCP 服务，安全保存文件",
        ],
        starter: include_str!("../studio/netfetch_starter.rs"),
        main: include_str!("../../netfetch/src/main.rs"),
        tests: include_str!("../../netfetch/tests/milestones.rs"),
        lesson: include_str!("../../../course/studio/netfetch.md"),
        page: include_str!("../studio/netfetch_lesson.html"),
        hints: include_str!("../studio/netfetch_hints.md"),
        support: &[],
        samples: &[],
    },
    Product {
        id: "servicedesk",
        title: "ServiceDesk · 本地工单网页",
        steps: [
            "用 Rust 规则检查工单标题",
            "管理新增与关闭的业务状态",
            "把工单转成供网页读取的 JSON",
        ],
        starter: include_str!("../studio/servicedesk_starter.rs"),
        main: include_str!("../../servicedesk/src/main.rs"),
        tests: include_str!("../../servicedesk/tests/milestones.rs"),
        lesson: include_str!("../../../course/studio/servicedesk.md"),
        page: include_str!("../studio/servicedesk_lesson.html"),
        hints: include_str!("../studio/servicedesk_hints.md"),
        support: &[("http.rs", include_str!("../../servicedesk/src/http.rs"))],
        samples: &[(
            "index.html",
            include_str!("../../servicedesk/samples/index.html"),
        )],
    },
    Product {
        id: "minikv",
        title: "MiniKV · 本地配置与书签仓库",
        steps: [
            "用结构化错误验证键",
            "让内存状态与写入结果一致",
            "实现借用数据的前缀迭代器",
        ],
        starter: include_str!("../studio/minikv_starter.rs"),
        main: include_str!("../../minikv/src/main.rs"),
        tests: include_str!("../../minikv/tests/milestones.rs"),
        lesson: include_str!("../../../course/studio/minikv.md"),
        page: include_str!("../studio/minikv_lesson.html"),
        hints: include_str!("../studio/minikv_hints.md"),
        support: &[("journal.rs", include_str!("../../minikv/src/journal.rs"))],
        samples: &[],
    },
    Product {
        id: "asyncboard",
        title: "AsyncBoard · 离线异步提醒工作台",
        steps: [
            "解析任务清单并验证边界",
            "借用完成结果生成 TSV 报告",
            "在 async fn 中真正 .await 并同时调度任务",
        ],
        starter: include_str!("../studio/asyncboard_starter.rs"),
        main: include_str!("../../asyncboard/src/main.rs"),
        tests: include_str!("../../asyncboard/tests/milestones.rs"),
        lesson: include_str!("../../../course/studio/asyncboard.md"),
        page: include_str!("../studio/asyncboard_lesson.html"),
        hints: include_str!("../studio/asyncboard_hints.md"),
        support: &[(
            "runtime.rs",
            include_str!("../../asyncboard/src/runtime.rs"),
        )],
        samples: &[(
            "jobs.tsv",
            include_str!("../../asyncboard/samples/jobs.tsv"),
        )],
    },
];

pub fn product(id: &str) -> Result<&'static Product, String> {
    PRODUCTS
        .iter()
        .find(|p| p.id == id)
        .ok_or_else(|| "未知作品；运行 studio catalog 查看可选作品".into())
}
fn manifest(id: &str) -> String {
    format!("[package]\nname = \"{id}\"\nversion = \"0.1.0\"\nedition = \"2021\"\nrust-version = \"1.74\"\n\n[workspace]\n")
}
fn write_new(path: &Path, text: &str) -> Result<(), String> {
    OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .and_then(|mut f| f.write_all(text.as_bytes()))
        .map_err(|e| format!("无法新建 {}：{e}", path.display()))
}
fn require_dir(path: &Path) -> Result<(), String> {
    let meta =
        fs::symlink_metadata(path).map_err(|e| format!("读取目录 {}：{e}", path.display()))?;
    if !meta.is_dir() || meta.file_type().is_symlink() {
        return Err(format!("目录不能是符号链接或普通文件：{}", path.display()));
    }
    Ok(())
}
pub fn init(id: &str, root: &Path) -> Result<(), String> {
    let p = product(id)?;
    match fs::symlink_metadata(root) {
        Ok(_) => {
            return Err(format!(
                "已有路径不会覆盖：{}；打开已有 START.html 继续，或换新名字",
                root.display()
            ))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.to_string()),
    }
    let parent = root
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if !parent.is_dir() {
        return Err(format!("父目录不存在：{}", parent.display()));
    }
    fs::create_dir(root).map_err(|e| e.to_string())?;
    for sub in ["src", "tests", "samples"] {
        fs::create_dir(root.join(sub)).map_err(|e| e.to_string())?;
    }
    write_new(&root.join("Cargo.toml"), &manifest(id))?;
    write_new(
        &root.join(".applab-studio"),
        &format!("RustAppLab Studio v1\n{id}\n"),
    )?;
    write_new(&root.join("src/lib.rs"), p.starter)?;
    write_new(&root.join("src/main.rs"), p.main)?;
    for (name, text) in p.support {
        write_new(&root.join("src").join(name), text)?;
    }
    write_new(&root.join("tests/milestones.rs"), p.tests)?;
    write_new(&root.join("LESSON.md"), p.lesson)?;
    let page = if let Some(tasks) = crate::learning::printable_tasks(id) {
        write_new(&root.join("TASK_CARDS.md"), &tasks)?;
        if let Some(page) = crate::learning::printable_tasks_html(id) {
            write_new(&root.join("TASK_CARDS.html"), &page)?;
        }
        p.page.replace("<article>", "<article><p>基础薄弱时先阅读同目录 <a href=\"TASK_CARDS.html\"><strong>分步任务卡（点击打开）</strong></a>，或查看 TASK_CARDS.md：每步目标、预测、三级提示与自己的数据实验。Windows 可返回完整平台，双击 LEARN_NEXT_WINDOWS.cmd。</p>")
    } else {
        p.page.to_string()
    };
    write_new(&root.join("START.html"), &page)?;
    write_new(&root.join("HINTS.md"), p.hints)?;
    write_new(&root.join(".gitignore"), "/target/\n/.studio-reports/\n")?;
    write_new(&root.join("LEARNING_LOG.md"),"# 我的学习记录\n\n## 我的需求与用途\n\n## 第 1 步：先预测，再修改，再测试\n输入、预期、实际、失败原因：\n\n## 第 2 步：用自己的话解释 Rust 概念\n\n## 第 3 步：新输入和异常场景\n\n## 我的扩展需求与新增测试\n\n## 交付作品\n运行命令、产物路径、已知限制：\n\n## 使用的帮助\n记录看过哪些提示或参考代码，不把参考程序的测试成绩当成自己的理解。\n")?;
    for (name, text) in p.samples {
        write_new(&root.join("samples").join(name), text)?;
    }
    println!("已创建 {}：{}\n现在只做第 1 步：{}\n用浏览器打开 START.html，编辑 src/lib.rs 的 TODO(1)，保存后检查。",p.title,root.display(),p.steps[0]);
    println!("在本学员目录内：cargo test --offline --test milestones step1_\n在 RustAppLab 根目录：cargo run --offline -p labquest -- studio check \"{}\" 1",root.display());
    Ok(())
}
fn identify(root: &Path) -> Result<&'static Product, String> {
    require_dir(root)?;
    let marker = root.join(".applab-studio");
    let meta = fs::symlink_metadata(&marker)
        .map_err(|_| "不是 Studio 学员工程；先使用 studio start 创建".to_string())?;
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > 128 {
        return Err("学习工程标记无效".into());
    }
    let text = fs::read_to_string(marker).map_err(|e| e.to_string())?;
    let parts: Vec<_> = text.lines().collect();
    if parts.len() != 2 || parts[0] != "RustAppLab Studio v1" {
        return Err("学习工程标记版本无效".into());
    }
    product(parts[1])
}
/// Copy only source text, not editable test files or the student's Cargo/build scripts.
fn copy_source(
    from: &Path,
    to: &Path,
    base: &Path,
    hashes: &mut Vec<(String, String)>,
    budget: &mut u64,
    depth: usize,
) -> Result<(), String> {
    if depth > 16 {
        return Err("src 嵌套超过 16 层".into());
    }
    require_dir(from)?;
    fs::create_dir(to).map_err(|e| e.to_string())?;
    let mut children = fs::read_dir(from)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    children.sort_by_key(|e| e.file_name());
    for entry in children {
        let path = entry.path();
        let meta = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if meta.file_type().is_symlink() {
            return Err(format!("src 内不允许符号链接：{}", path.display()));
        }
        let dest = to.join(entry.file_name());
        if meta.is_dir() {
            copy_source(&path, &dest, base, hashes, budget, depth + 1)?;
            continue;
        }
        if !meta.is_file() {
            return Err("src 内仅支持普通文件".into());
        }
        if path.extension().and_then(|x| x.to_str()) != Some("rs") {
            return Err("分步验收暂时只复制 src 内 .rs 文件；嵌入资源请放在后续自主扩展中".into());
        }
        if meta.len() > 1024 * 1024 || *budget + meta.len() > 1024 * 1024 || hashes.len() >= 64 {
            return Err("学员源文件上限：64 个，总计 1 MiB".into());
        }
        let bytes = fs::read(&path).map_err(|e| e.to_string())?;
        *budget += bytes.len() as u64;
        if *budget > 1024 * 1024 {
            return Err("源码在读取时超出 1 MiB".into());
        }
        std::str::from_utf8(&bytes).map_err(|_| "源文件必须是 UTF-8".to_string())?;
        fs::write(dest, &bytes).map_err(|e| e.to_string())?;
        let mut hash = Sha256::new();
        hash.update(&bytes);
        hashes.push((
            path.strip_prefix(base)
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .into_owned(),
            hex(&hash.finalize()),
        ));
    }
    Ok(())
}
fn find_test_binary(scratch: &Path) -> Result<PathBuf, String> {
    let dir = scratch.join("target/debug/deps");
    let files = fs::read_dir(dir).map_err(|e| e.to_string())?;
    for item in files {
        let path = item.map_err(|e| e.to_string())?.path();
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        let extension = path.extension().and_then(|s| s.to_str());
        if name.starts_with("milestones-")
            && path.is_file()
            && (if cfg!(windows) {
                extension == Some("exe")
            } else {
                extension.is_none()
            })
        {
            return Ok(path);
        }
    }
    Err("编译完成但没有找到 milestones 测试程序；检查 Cargo 配置".into())
}
fn save_report(root: &Path, text: &str) -> Result<PathBuf, String> {
    let dir = root.join(".studio-reports");
    match fs::create_dir(&dir) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => require_dir(&dir)?,
        Err(e) => return Err(e.to_string()),
    }
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let path = dir.join(format!("ASSESSMENT-{stamp}-{}.md", std::process::id()));
    write_new(&path, text)?;
    Ok(path)
}
/// Re-evaluate all preceding milestones. Each run has a source snapshot, no permanent "passed" flag.
pub fn check(root: &Path, upto: usize) -> Result<bool, String> {
    if !(1..=3).contains(&upto) {
        return Err("步骤只支持 1、2、3 或 all".into());
    }
    let p = identify(root)?;
    let temp = TempWorkspace::new_named("studio")?;
    let mut hashes = Vec::new();
    let mut budget = 0;
    copy_source(
        &root.join("src"),
        &temp.0.join("src"),
        root,
        &mut hashes,
        &mut budget,
        0,
    )?;
    if !temp.0.join("src/lib.rs").is_file() || !temp.0.join("src/main.rs").is_file() {
        return Err("请保留 src/lib.rs 和 src/main.rs".into());
    }
    write_new(&temp.0.join("Cargo.toml"), &manifest(p.id))?;
    fs::create_dir(temp.0.join("tests")).map_err(|e| e.to_string())?;
    write_new(&temp.0.join("tests/milestones.rs"), p.tests)?;
    fs::create_dir(temp.0.join("samples")).map_err(|e| e.to_string())?;
    for (name, text) in p.samples {
        write_new(&temp.0.join("samples").join(name), text)?;
    }
    let mut report=format!("# {} · 本次源码分步验收\n\n目标：累计通过前 {} 步；不读取或信任可修改的 tests。\n\n## 本次编译的源码快照 SHA-256\n\n",p.title,upto);
    for (path, hash) in hashes {
        report.push_str(&format!("- `{path}`：`{hash}`\n"));
    }
    println!(
        "{}：先编译你的 src；使用内置合同检查第 1–{upto} 步。",
        p.title
    );
    let mut build = Command::new("cargo");
    build
        .current_dir(&temp.0)
        .args([
            "test",
            "--offline",
            "--quiet",
            "--test",
            "milestones",
            "--no-run",
            "--target-dir",
        ])
        .arg(temp.0.join("target"))
        .env_remove("CARGO_BUILD_TARGET");
    crate::process::learner_cargo(&mut build);
    let built = run_bounded(
        &mut build,
        Duration::from_secs(120),
        &temp.0,
        "Studio 学员源码编译",
    );
    let mut passed = 0usize;
    let ready = match built {
        Ok(out) if out.success => true,
        Ok(out) => {
            println!(
                "{}\n{}",
                compilation_guidance(&out.stderr),
                short(&out.stderr, 2200)
            );
            let feedback = crate::learning::diagnose_text(&out.stderr);
            println!("{feedback}");
            report.push_str(&format!(
                "\n## 编译 FAIL\n\n```text\n{}\n```\n\n### 教学解释\n{feedback}\n",
                out.stderr
            ));
            false
        }
        Err(err) => {
            println!("{err}");
            report.push_str(&format!("\n## 编译 FAIL\n\n{err}\n"));
            false
        }
    };
    if ready {
        let binary = find_test_binary(&temp.0)?;
        for step in 1..=upto {
            let mut command = Command::new(&binary);
            command
                .current_dir(&temp.0)
                .arg(format!("step{step}_"))
                .args(["--test-threads=1", "--nocapture"])
                .env("RUST_BACKTRACE", "0");
            let outcome = run_bounded(
                &mut command,
                Duration::from_secs(8),
                &temp.0,
                "Studio 当前里程碑运行",
            );
            match outcome {
                Ok(out) => {
                    // First two milestones have 2 tests, final milestone also runs 2 real CLI cases.
                    let expected = match (p.id, step) {
                        ("netfetch", 3) => 5,
                        ("servicedesk", 3) => 6,
                        (_, 3) => 4,
                        _ => 2,
                    };
                    let success =
                        out.success && out.stdout.contains(&format!("{expected} passed; 0 failed"));
                    let status = if success { "PASS" } else { "FAIL" };
                    println!("[{status}] 第 {step} 步：{}", p.steps[step - 1]);
                    report.push_str(&format!(
                        "\n## 第 {step} 步 {status}：{}\n\n```text\n{}\n{}\n```\n",
                        p.steps[step - 1],
                        out.stdout,
                        out.stderr
                    ));
                    if !success {
                        let feedback = crate::learning::failure_feedback(
                            p.id,
                            step,
                            &format!("{}\n{}", out.stdout, out.stderr),
                        );
                        println!("{feedback}");
                        report.push_str(&format!("\n### 教学反馈\n{feedback}\n"));
                        println!("{}\n下一步：只看 src/lib.rs 的 TODO({step}) 与 LESSON.md 第 {step} 步；后面的关卡还没开始检查。",short(&format!("{}\n{}",out.stdout,out.stderr),2200));
                        break;
                    }
                    passed += 1;
                }
                Err(err) => {
                    println!("[FAIL] 第 {step} 步：{err}\n先检查无限循环、等待输入，或没有关闭的通道发送端。");
                    report.push_str(&format!("\n## 第 {step} 步 FAIL\n\n{err}\n"));
                    break;
                }
            }
        }
    }
    report.push_str(&format!("\n本次累计：{passed}/{upto}。这是源码快照的功能证据，不是学员理解能力认证；旧报告不代表新代码仍然通过。\n"));
    let saved = save_report(root, &report)?;
    println!("本次记录：{}", saved.display());
    if passed == upto {
        if upto < 3 {
            println!(
                "当前阶段完成。接下来读第 {} 步，修改后运行 studio check \"{}\" {}。",
                upto + 1,
                root.display(),
                upto + 1
            );
        } else {
            println!("三个里程碑通过。现在用自己的数据运行软件，保存产物，填写 LEARNING_LOG.md，再添加一个自己的测试。并发等实现策略仍需阅读源码确认。");
        }
    }
    Ok(passed == upto)
}

pub fn run(args: &[OsString]) -> Result<(), String> {
    let cmd = args.first().and_then(|s| s.to_str()).unwrap_or("catalog");
    match cmd{
        "catalog" if args.len()<=1=>{for (i,p) in PRODUCTS.iter().enumerate(){println!("{} · {} ({})\n    {} → {} → {}",i+1,p.title,p.id,p.steps[0],p.steps[1],p.steps[2]);}println!("零基础先选 unitlab，再 quizdeck、textscope；termgarden/storedesk 是状态与数据库阶段；netfetch/servicedesk 为同步网络基础版；asyncboard 使用真正的 Future/await，但非 Tokio 网络运行时。\n创建：studio start <作品ID> <新目录>\n检查：studio check <目录> [1|2|3|all]，默认只检查第 1 步。");Ok(())},
        "start" if args.len()==3=>init(args[1].to_str().ok_or("作品 ID 不是 UTF-8")?,Path::new(&args[2])),
        "check" if (2..=3).contains(&args.len())=>{
            let step=match args.get(2).and_then(|s|s.to_str()).unwrap_or("1"){"1"=>1,"2"=>2,"3"|"all"=>3,_=>return Err("步骤只支持 1、2、3、all".into())};
            if check(Path::new(&args[1]),step)?{Ok(())}else{Err("当前阶段尚未完成；这不会删除你的源码，按上方第一项失败反馈继续修改".into())}
        },
        "lesson"|"hint" if args.len()==2=>{let p=product(args[1].to_str().ok_or("作品 ID 不是 UTF-8")?)?;println!("{}",if cmd=="lesson"{p.lesson}else{p.hints});Ok(())},
        _=>Err("用法：studio catalog | start <ID> <新目录> | check <目录> [1|2|3|all] | lesson <ID> | hint <ID>".into())
    }
}

fn compilation_guidance(stderr: &str) -> &'static str {
    let lower = stderr.to_lowercase();
    if (lower.contains("winsqlite3") || lower.contains("-lsqlite3"))
        && (lower.contains("cannot")
            || lower.contains("not found")
            || lower.contains("lnk1104")
            || lower.contains("unable"))
    {
        "数据库链接环境尚未就绪，不是 TODO 写错。Windows：检查 MSVC + Windows 10/11 SDK 中的 winsqlite3.lib；Linux：安装 SQLite 开发库；macOS：检查命令行开发工具。先读 course/SQLITE_SETUP.md，再试参考程序 storedesk doctor。不要删除你的源码。"
    } else {
        "编译未通过。先处理第一条 error，再检查函数的括号、分号、返回类型。"
    }
}
#[cfg(test)]
mod diagnostics_tests {
    use super::compilation_guidance;
    #[test]
    fn missing_sqlite_is_environment_error() {
        assert!(compilation_guidance(
            "LINK : fatal error LNK1104: cannot open file 'winsqlite3.lib'"
        )
        .contains("不是 TODO"));
        assert!(compilation_guidance("/usr/bin/ld: cannot find -lsqlite3").contains("开发库"));
    }
    #[test]
    fn ordinary_compile_error_keeps_learning_guidance() {
        assert!(compilation_guidance("error[E0308]: mismatched types").contains("返回类型"));
    }
}
