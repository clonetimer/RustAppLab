//! Advanced track A: cumulative checks for an independently generated TermGarden extension.
//! Only `check` executes trusted learner code. The fixed manifest/tests/core are regenerated.
use crate::capstone::{run_bounded, TempWorkspace};
use safesync::sha256::{hex, Sha256};
use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
const MARKER: &str = "RustAppLab Advanced A v1\n";
const MANIFEST: &str = r#"[package]
name = "garden-workbench"
version = "0.1.0"
edition = "2021"
rust-version = "1.74"
[dependencies]
termgarden = { path = "support/termgarden" }
[workspace]
"#;
const STARTER: &str = include_str!("../advanced/workbench_starter.rs");
const GUIDE: &str = include_str!("../../../course/ADVANCED_A.md");
const PAGE: &str = include_str!("../../../ADVANCED_A.html");
const PLAN: &str = include_str!("../advanced/DESIGN_REVIEW.md");
const TESTS: [&str; 3] = [
    include_str!("../../termgarden/tests/advanced_a1.rs"),
    include_str!("../../termgarden/tests/advanced_a2.rs"),
    include_str!("../../termgarden/tests/advanced_a3.rs"),
];
const HELP:&str="进阶 A · TermGarden 工作区\nadvanced start <全新目录>\nadvanced lesson [1|2|3]\nadvanced check <学员目录> <1|2|3>\nadvanced next <学员目录>\n原基础工程不需要改动；独立创建进阶工程。\n";
fn write(path: &Path, text: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(path, text).map_err(|e| e.to_string())
}
fn regular(path: &Path) -> Result<Vec<u8>, String> {
    let m = fs::symlink_metadata(path).map_err(|e| format!("{}：{e}", path.display()))?;
    if !m.is_file() || m.file_type().is_symlink() || m.len() > 1024 * 1024 {
        return Err("只读取不超过1MiB的非链接普通源码/报告".into());
    }
    fs::read(path).map_err(|e| e.to_string())
}
pub fn is_project(root: &Path) -> bool {
    root.join(".applab-advanced").exists()
}
fn validate_root(root: &Path) -> Result<(), String> {
    let m = fs::symlink_metadata(root).map_err(|e| e.to_string())?;
    if !m.is_dir() || m.file_type().is_symlink() {
        return Err("工程必须是普通目录，不能是符号链接".into());
    }
    if regular(&root.join(".applab-advanced"))? != MARKER.as_bytes() {
        return Err("不是进阶 A 工程".into());
    }
    Ok(())
}
fn source_files(root: &Path) -> Result<BTreeMap<String, Vec<u8>>, String> {
    fn visit(
        root: &Path,
        dir: &Path,
        depth: usize,
        files: &mut BTreeMap<String, Vec<u8>>,
    ) -> Result<(), String> {
        let m = fs::symlink_metadata(dir).map_err(|e| e.to_string())?;
        if depth > 8 || !m.is_dir() || m.file_type().is_symlink() {
            return Err("源码目录过深或含符号链接".into());
        }
        for e in fs::read_dir(dir).map_err(|e| e.to_string())? {
            let path = e.map_err(|e| e.to_string())?.path();
            let m = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
            if m.file_type().is_symlink() {
                return Err("源码不能包含符号链接".into());
            }
            if m.is_dir() {
                visit(root, &path, depth + 1, files)?;
            } else if path.extension().is_some_and(|x| x == "rs") {
                if files.len() >= 64 {
                    return Err("最多64个Rust源码文件".into());
                }
                files.insert(
                    path.strip_prefix(root)
                        .map_err(|e| e.to_string())?
                        .to_string_lossy()
                        .replace('\\', "/"),
                    regular(&path)?,
                );
            }
        }
        Ok(())
    }
    validate_root(root)?;
    let mut files = BTreeMap::new();
    visit(root, &root.join("src"), 0, &mut files)?;
    if !files.contains_key("src/lib.rs") {
        return Err("缺少 src/lib.rs".into());
    }
    Ok(files)
}
fn fingerprint(files: &BTreeMap<String, Vec<u8>>) -> String {
    let mut sha = Sha256::new();
    for (name, bytes) in files {
        sha.update(&(name.len() as u64).to_le_bytes());
        sha.update(name.as_bytes());
        sha.update(&(bytes.len() as u64).to_le_bytes());
        sha.update(bytes);
    }
    hex(&sha.finalize())
}
fn prepare(root: &Path) -> Result<(), String> {
    write(&root.join("Cargo.toml"), MANIFEST)?;
    write(
        &root.join("support/termgarden/Cargo.toml"),
        include_str!("../../termgarden/Cargo.toml"),
    )?;
    // Supply the *old* core and persistence only. Do not include the advanced answer.
    write(
        &root.join("support/termgarden/src/lib.rs"),
        &include_str!("../../termgarden/src/lib.rs").replace("pub mod workbench;\n", ""),
    )?;
    write(
        &root.join("support/termgarden/src/storage.rs"),
        include_str!("../../termgarden/src/storage.rs"),
    )?;
    write(&root.join("src/lib.rs"), STARTER)?;
    let main = include_str!("../../termgarden/examples/workbench.rs")
        .replace("termgarden::workbench", "garden_workbench");
    write(&root.join("src/main.rs"), &main)?;
    for (i, test) in TESTS.iter().enumerate() {
        write(
            &root.join(format!("tests/a{}.rs", i + 1)),
            &test.replace("termgarden::workbench", "garden_workbench"),
        )?;
    }
    Ok(())
}
pub fn init(root: &Path) -> Result<(), String> {
    let parent = root
        .parent()
        .filter(|x| !x.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    if !parent.is_dir() {
        return Err("上级目录不存在；请先选一个已有练习目录".into());
    }
    fs::create_dir(root).map_err(|e| format!("只创建全新目录，不覆盖已有作品：{e}"))?;
    prepare(root)?;
    write(&root.join(".applab-advanced"), MARKER)?;
    write(&root.join("START.html"), PAGE)?;
    write(&root.join("GUIDE.md"), GUIDE)?;
    write(&root.join("DESIGN_REVIEW.md"), PLAN)?;
    write(&root.join("LEARNING_LOG.md"),"# 进阶 A 学习记录\n\n记录自己的预测、报错、修改、个人测试与真实任务文件。\n不要把通过合同测试写成教师已认可设计；请完成 DESIGN_REVIEW.md。\n")?;
    write(
        &root.join("samples/session.txt"),
        "add 学习所有权\nadd 写自己的测试\nprefix 本周：\nundo\nsave\nq\n",
    )?;
    Ok(())
}
fn reports(root: &Path) -> Result<PathBuf, String> {
    let dir = root.join(".advanced-reports");
    match fs::symlink_metadata(&dir) {
        Ok(m) if m.is_dir() && !m.file_type().is_symlink() => {}
        Ok(_) => return Err("报告路径必须是非符号链接目录".into()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            fs::create_dir(&dir).map_err(|e| e.to_string())?
        }
        Err(e) => return Err(e.to_string()),
    }
    Ok(dir)
}
fn last_stage(root: &Path) -> Result<(usize, bool), String> {
    let hash = fingerprint(&source_files(root)?);
    let dir = root.join(".advanced-reports");
    if !dir.exists() {
        return Ok((1, false));
    }
    let m = fs::symlink_metadata(&dir).map_err(|e| e.to_string())?;
    if !m.is_dir() || m.file_type().is_symlink() {
        return Err("报告目录非法".into());
    }
    let mut paths = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())?.take(1000) {
        let p = entry.map_err(|e| e.to_string())?.path();
        if p.extension().is_some_and(|x| x == "txt") {
            paths.push(p);
        }
    }
    paths.sort();
    for p in paths.into_iter().rev() {
        let text = String::from_utf8(regular(&p)?).map_err(|e| e.to_string())?;
        if !text.starts_with("RustAppLab Advanced Report v1\n") {
            continue;
        }
        let lines: Vec<_> = text.lines().collect();
        if lines.get(1) != Some(&format!("source_sha256={hash}").as_str()) {
            return Ok((1, true));
        }
        if let Some(passed) = lines
            .get(2)
            .and_then(|s| s.strip_prefix("passed="))
            .and_then(|s| s.parse::<usize>().ok())
            .filter(|n| *n <= 3)
        {
            return Ok((passed + 1, false));
        }
    }
    Ok((1, false))
}
pub fn next(root: &Path) -> Result<String, String> {
    let (stage, stale) = last_stage(root)?;
    if stage == 4 {
        return Ok(format!("进阶 A · 当前源码有3/3历史通过记录。\n下一步不是重复填函数：用自己的数据运行play/save/report，写新回归测试，完成 {}/DESIGN_REVIEW.md。\n这是可编辑的本地记录，不是教师认证；修改源码后需要重新回归。\n",root.display()));
    }
    Ok(format!("进阶 A · 当前建议第 {stage} 步\n{}\n编辑：{}/src/lib.rs\n检查：labquest advanced check \"{}\" {stage}\n阶段3通过后：用自己的任务运行 play/save/report，写一项新测试并完成 DESIGN_REVIEW.md。记录可编辑，不是能力认证。\n",if stale{"源码已变化，旧报告失效；从第1步重新回归。"}else{"查看 START.html；每次检查均重新运行前面阶段。"},root.display(),root.display()))
}
pub fn check(root: &Path, stage: usize) -> Result<bool, String> {
    if !(1..=3).contains(&stage) {
        return Err("进阶检查点只能为1、2、3".into());
    }
    let files = source_files(root)?;
    let hash = fingerprint(&files);
    let temp = TempWorkspace::new_named("advanced-a")?;
    let project = temp.0.join("project");
    fs::create_dir(&project).map_err(|e| e.to_string())?;
    prepare(&project)?;
    for (name, bytes) in &files {
        let target = project.join(name);
        fs::create_dir_all(target.parent().unwrap()).map_err(|e| e.to_string())?;
        fs::write(target, bytes).map_err(|e| e.to_string())?;
    }
    let mut log = String::new();
    let mut passed = 0;
    for step in 1..=stage {
        println!("[进阶 A{step}] 正在编译并检查；前面阶段也会回归。");
        let mut cmd = Command::new("cargo");
        cmd.args(["test", "--offline", "--test", &format!("a{step}")]);
        crate::process::learner_cargo(&mut cmd)
            .arg("--")
            .arg("--test-threads=1")
            .current_dir(&project)
            .env("CARGO_TARGET_DIR", temp.0.join("target"));
        match run_bounded(&mut cmd, Duration::from_secs(120), &temp.0, "进阶行为测试") {
            Ok(out) => {
                log.push_str(&format!(
                    "\n=== A{step} ===\n{}\n{}",
                    out.stdout, out.stderr
                ));
                if out.success {
                    passed = step;
                } else {
                    break;
                }
            }
            Err(e) => {
                log.push_str(&format!("\n运行错误：{e}\n"));
                break;
            }
        }
    }
    let dir = reports(root)?;
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let path = dir.join(format!("CHECK-{nanos}-{}.txt", std::process::id()));
    let text=format!("RustAppLab Advanced Report v1\nsource_sha256={hash}\npassed={passed}\nrequested={stage}\n\n本次累计通过{passed}/{stage}。只检查src/*.rs的行为；使用平台固定依赖/合同，不执行个人tests。请另行运行个人cargo test。\n{log}");
    let mut f = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&path)
        .map_err(|e| e.to_string())?;
    f.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
    println!("本次累计：{passed}/{stage}；报告：{}", path.display());
    if passed != stage {
        println!("{}", crate::learning::diagnose_text(&log));
    }
    Ok(passed == stage)
}
pub fn lesson(stage: usize) -> Result<String, String> {
    if !(1..=3).contains(&stage) {
        return Err("步骤应为1、2、3".into());
    }
    let heading = format!("## A{stage} ");
    let from = GUIDE.find(&heading).ok_or("讲义缺失")?;
    let rest = &GUIDE[from..];
    let end = rest[1..].find("\n## ").map(|i| i + 1).unwrap_or(rest.len());
    Ok(rest[..end].to_string())
}
pub fn learn_run(args: &[OsString]) -> Result<(), String> {
    let root = Path::new(&args[1]);
    validate_root(root)?;
    match args[0].to_str() {
        Some("next") if args.len() == 2 => {
            print!("{}", next(root)?);
            Ok(())
        }
        Some("lesson" | "check") if (2..=3).contains(&args.len()) => {
            let stage = if let Some(value) = args.get(2) {
                parse_stage(value)?
            } else {
                last_stage(root)?.0.min(3)
            };
            if args[0] == "lesson" {
                println!("{}", lesson(stage)?);
                Ok(())
            } else if check(root, stage)? {
                Ok(())
            } else {
                Err("进阶当前检查点尚未通过".into())
            }
        }
        Some("hint") if (3..=4).contains(&args.len()) => {
            let level = parse_stage(&args[2])?;
            let stage = if let Some(v) = args.get(3) {
                parse_stage(v)?
            } else {
                last_stage(root)?.0.min(3)
            };
            let text = lesson(stage)?;
            let heading = format!("### 提示 {level}");
            let start = text.find(&heading).ok_or("提示缺失")?;
            let rest = &text[start..];
            let end = rest[1..]
                .find("\n### ")
                .map(|n| n + 1)
                .unwrap_or(rest.len());
            println!("{}", &rest[..end]);
            Ok(())
        }
        _ => Err(HELP.into()),
    }
}
fn parse_stage(s: &OsString) -> Result<usize, String> {
    match s.to_str() {
        Some("1") => Ok(1),
        Some("2") => Ok(2),
        Some("3" | "all") => Ok(3),
        _ => Err("阶段应为1、2、3或all".into()),
    }
}
pub fn run(args: &[OsString]) -> Result<(), String> {
    match args.first().and_then(|s| s.to_str()) {
        None | Some("help") => {
            print!("{HELP}");
            Ok(())
        }
        Some("start") if args.len() == 2 => {
            init(Path::new(&args[1]))?;
            println!(
                "已创建：{}；打开 START.html；编辑自己的 src/lib.rs。",
                Path::new(&args[1]).display()
            );
            Ok(())
        }
        Some("lesson") if args.len() == 1 => {
            print!("{GUIDE}");
            Ok(())
        }
        Some("lesson") if args.len() == 2 => {
            print!("{}", lesson(parse_stage(&args[1])?)?);
            Ok(())
        }
        Some("next") if args.len() == 2 => {
            print!("{}", next(Path::new(&args[1]))?);
            Ok(())
        }
        Some("check") if args.len() == 3 => {
            if check(Path::new(&args[1]), parse_stage(&args[2])?)? {
                Ok(())
            } else {
                Err("尚未通过；先读报告里的第一条错误。".into())
            }
        }
        _ => Err(HELP.into()),
    }
}
