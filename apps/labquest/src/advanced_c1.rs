//! C1 fixed-contract assessment. Runs trusted learner Rust, not a security sandbox.
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
const MARKER: &str = "RustAppLab C1 v1\n";
const GUIDE: &str = include_str!("../../../course/ADVANCED_C1.md");
const MANIFEST: &str = include_str!("../../../projects/advanced-c1/reference/Cargo.toml");
const LOCK: &str = include_str!("../../../projects/advanced-c1/reference/Cargo.lock");
const TESTS: &str = include_str!("../../../projects/advanced-c1/reference/tests/contracts.rs");
const MODEL: &str = include_str!("../../../projects/advanced-c1/reference/src/model.rs");
const MAIN: &str = include_str!("../../../projects/advanced-c1/reference/src/main.rs");
const REFERENCE: [(&str, &str); 3] = [
    (
        "lib.rs",
        include_str!("../../../projects/advanced-c1/reference/src/lib.rs"),
    ),
    (
        "transport.rs",
        include_str!("../../../projects/advanced-c1/reference/src/transport.rs"),
    ),
    (
        "queue.rs",
        include_str!("../../../projects/advanced-c1/reference/src/queue.rs"),
    ),
];
const STARTER: &str = include_str!("../../../projects/advanced-c1/learner/src/lib.rs");
fn write(path: &Path, text: &str) -> Result<(), String> {
    fs::create_dir_all(path.parent().ok_or("missing parent")?).map_err(|e| e.to_string())?;
    fs::write(path, text).map_err(|e| e.to_string())
}
fn directory(path: &Path) -> Result<(), String> {
    let m = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if !m.is_dir() || m.file_type().is_symlink() {
        return Err("需要普通目录，拒绝目录符号链接".into());
    }
    Ok(())
}
pub fn is_project(root: &Path) -> bool {
    crate::learning::require_regular(&root.join(".applab-c1"), 128).is_ok_and(|s| s == MARKER)
}
fn validate(root: &Path) -> Result<(), String> {
    directory(root)?;
    if !is_project(root) {
        return Err("不是 C1 学员工程".into());
    }
    Ok(())
}
fn install(root: &Path, reference: bool) -> Result<(), String> {
    for (path, text) in [
        ("Cargo.toml", MANIFEST),
        ("Cargo.lock", LOCK),
        ("src/model.rs", MODEL),
        ("src/main.rs", MAIN),
        ("tests/contracts.rs", TESTS),
    ] {
        write(&root.join(path), text)?;
    }
    if reference {
        for (path, text) in REFERENCE {
            write(&root.join("src").join(path), text)?;
        }
    } else {
        write(&root.join("src/lib.rs"), STARTER)?;
    }
    Ok(())
}
pub fn init(root: &Path) -> Result<(), String> {
    fs::create_dir(root).map_err(|e| format!("仅创建全新学员目录：{e}"))?;
    install(root, false)?;
    write(&root.join("START_HERE.md"), GUIDE)?;
    write(&root.join(".applab-c1"), MARKER)
}
fn sources(root: &Path) -> Result<BTreeMap<String, Vec<u8>>, String> {
    validate(root)?;
    fn walk(
        base: &Path,
        dir: &Path,
        out: &mut BTreeMap<String, Vec<u8>>,
        depth: usize,
    ) -> Result<(), String> {
        directory(dir)?;
        if depth > 8 {
            return Err("源码目录过深".into());
        }
        for item in fs::read_dir(dir).map_err(|e| e.to_string())? {
            let path = item.map_err(|e| e.to_string())?.path();
            let meta = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
            if meta.file_type().is_symlink() {
                return Err("源码中不能包含符号链接".into());
            }
            if meta.is_dir() {
                walk(base, &path, out, depth + 1)?;
            } else if meta.is_file()
                && path.extension().is_some_and(|x| x == "rs")
                && meta.len() <= 256 * 1024
                && out.len() < 64
            {
                let name = path
                    .strip_prefix(base)
                    .map_err(|e| e.to_string())?
                    .to_str()
                    .ok_or("源码路径需为 Unicode")?
                    .replace('\\', "/");
                out.insert(name, fs::read(path).map_err(|e| e.to_string())?);
            } else {
                return Err("src 仅支持最多64个普通 Rust 文件，每个不超过256 KiB".into());
            }
        }
        Ok(())
    }
    let mut files = BTreeMap::new();
    walk(&root.join("src"), &root.join("src"), &mut files, 0)?;
    if !files.contains_key("lib.rs") || !files.contains_key("main.rs") {
        return Err("缺少 lib.rs/main.rs".into());
    }
    if files.values().map(Vec::len).sum::<usize>() > 2 * 1024 * 1024 {
        return Err("源码总量超过2 MiB".into());
    }
    Ok(files)
}
fn digest(files: &BTreeMap<String, Vec<u8>>) -> String {
    let mut hash = Sha256::new();
    for (path, data) in files {
        hash.update(path.as_bytes());
        hash.update(&[0]);
        hash.update(&(data.len() as u64).to_le_bytes());
        hash.update(data);
    }
    hex(&hash.finalize())
}
fn contract() -> String {
    let mut hash = Sha256::new();
    for text in ["C1 Report v1\n", MANIFEST, LOCK, TESTS] {
        hash.update(text.as_bytes());
    }
    hex(&hash.finalize())
}
fn record(root: &Path, group: &str, text: &str) -> Result<(), String> {
    let folder = root.join(group);
    if !folder.exists() {
        fs::create_dir(&folder).map_err(|e| e.to_string())?;
        write(&folder.join(".owner"), MARKER)?;
    }
    directory(&folder)?;
    if crate::learning::require_regular(&folder.join(".owner"), 128)? != MARKER {
        return Err("日志目录缺少所有权标记".into());
    }
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let path = folder.join(format!("{stamp:030}-{}.txt", std::process::id()));
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .and_then(|mut f| f.write_all(text.as_bytes()))
        .map_err(|e| e.to_string())
}
fn configure_vendor(root: &Path) -> Result<(), String> {
    if let Some(path) = std::env::var_os("RUSTAPPLAB_C1_VENDOR") {
        let path = fs::canonicalize(path).map_err(|e| format!("vendor 路径无效：{e}"))?;
        directory(&path)?;
        let s = path.to_str().ok_or("vendor 路径需为 Unicode")?;
        if s.chars().any(char::is_control) {
            return Err("vendor 路径包含控制字符".into());
        }
        let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
        write(&root.join(".cargo/config.toml"), &format!("[source.crates-io]\nreplace-with = \"c1-vendor\"\n[source.c1-vendor]\ndirectory = \"{escaped}\"\n"))?;
    }
    Ok(())
}
fn cargo(
    temp: &TempWorkspace,
    action: &str,
    extra: &[&str],
    offline: bool,
) -> Result<(bool, String), String> {
    let mut cmd = Command::new("cargo");
    cmd.arg("+1.85.0").arg(action).arg("--locked");
    if offline {
        cmd.arg("--offline");
    }
    cmd.args(extra)
        .current_dir(&temp.0)
        .env("CARGO_TARGET_DIR", temp.0.join("target"))
        .env("CARGO_BUILD_JOBS", "2")
        .env("CARGO_TERM_COLOR", "never")
        .env_remove("CARGO_BUILD_TARGET");
    let result = run_bounded(&mut cmd, Duration::from_secs(300), &temp.0, "C1 Cargo")?;
    Ok((
        result.success,
        format!(
            "$ cargo +1.85.0 {action} --locked {} {}\n{}\n{}",
            if offline { "--offline" } else { "" },
            extra.join(" "),
            result.stdout,
            result.stderr
        ),
    ))
}
pub fn prepare(root: &Path) -> Result<(), String> {
    validate(root)?;
    let temp = TempWorkspace::new_named("c1-prepare")?;
    install(&temp.0, true)?;
    configure_vendor(&temp.0)?;
    let mut log = String::from("C1 dependency preparation; NOT a learner grade\n");
    for (action, extra, offline) in [
        ("fetch", vec![], false),
        ("test", vec!["--all-targets"], true),
        ("test", vec!["--doc"], true),
    ] {
        let result = cargo(&temp, action, &extra, offline);
        match result {
            Ok((ok, text)) => {
                log.push_str(&text);
                if !ok {
                    record(root, ".c1-prepare", &log)?;
                    return Err(
                        "依赖准备失败，参见 .c1-prepare；需要 Rust 1.85.0 与可用依赖，不是学员成绩"
                            .into(),
                    );
                }
            }
            Err(e) => {
                log.push_str(&e);
                record(root, ".c1-prepare", &log)?;
                return Err(e);
            }
        }
    }
    log.push_str("PREPARATION PASS (not a grade)\n");
    record(root, ".c1-prepare", &log)
}
pub fn check(root: &Path, requested: usize) -> Result<bool, String> {
    if !(1..=3).contains(&requested) {
        return Err("阶段为1、2、3或all".into());
    }
    let files = sources(root)?;
    let source = digest(&files);
    let temp = TempWorkspace::new_named("c1-check")?;
    install(&temp.0, true)?;
    configure_vendor(&temp.0)?;
    let (ok, env_log) = cargo(&temp, "metadata", &["--format-version", "1"], true)?;
    if !ok {
        record(root, ".c1-prepare", &env_log)?;
        return Err("C1 环境预检失败；查看 .c1-prepare 原始诊断，确认 Rust1.85.0、固定锁和依赖缓存，显式运行 prepare。没有生成新成绩。".into());
    }
    fs::remove_dir_all(temp.0.join("src")).map_err(|e| e.to_string())?;
    for (path, data) in files {
        let target = temp.0.join("src").join(path);
        fs::create_dir_all(target.parent().ok_or("path")?).map_err(|e| e.to_string())?;
        fs::write(target, data).map_err(|e| e.to_string())?;
    }
    let mut passed = 0;
    let mut logs = String::new();
    for stage in 1..=requested {
        let filter = format!("c1_{stage}_");
        let result = cargo(&temp, "test", &["--test", "contracts", &filter], true);
        let (ok, text) = match result {
            Ok(out) => out,
            Err(e) => (false, e),
        };
        logs.push_str(&text);
        let expected = TESTS.matches(&format!("fn c1_{stage}_")).count();
        if !ok
            || expected == 0
            || !text.contains(&format!(
                "test result: ok. {expected} passed; 0 failed; 0 ignored;"
            ))
        {
            break;
        }
        passed = stage;
    }
    if passed == 3 {
        let result = cargo(&temp, "test", &["--doc"], true);
        let (ok, text) = match result {
            Ok(out) => out,
            Err(e) => (false, e),
        };
        logs.push_str(&text);
        if !ok {
            passed = 2;
        }
    }
    let report = format!("C1 Report v1\nsource={source}\ncontract={}\nrequested={requested}\npassed={passed}\n\n{logs}",contract());
    record(root, ".c1-reports", &report)?;
    println!("C1 检查：{passed}/3；日志在 .c1-reports");
    Ok(passed == requested)
}
pub fn next(root: &Path) -> Result<String, String> {
    let hash = digest(&sources(root)?);
    let reports = root.join(".c1-reports");
    if !reports.exists() {
        return Ok(
            "C1 尚未检查；从阶段1开始。先阅读 START_HERE.md，首次依赖需要显式 prepare。".into(),
        );
    }
    directory(&reports)?;
    let mut entries: Vec<PathBuf> = fs::read_dir(&reports)
        .map_err(|e| e.to_string())?
        .map(|x| x.map(|x| x.path()))
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
    entries.retain(|p| p.extension().is_some_and(|e| e == "txt"));
    entries.sort();
    let Some(last) = entries.last() else {
        return Ok("C1 尚未检查".into());
    };
    let report = crate::learning::require_regular(last, 2 * 1024 * 1024)?;
    let header: Vec<_> = report.lines().take(5).collect();
    if header.len() != 5
        || header[0] != "C1 Report v1"
        || header[1] != format!("source={hash}")
        || header[2] != format!("contract={}", contract())
    {
        return Ok("C1 报告过期或合同已更新；请重新 check。".into());
    }
    let passed = header[4]
        .strip_prefix("passed=")
        .and_then(|n| n.parse::<usize>().ok())
        .filter(|n| *n <= 3)
        .ok_or("成绩格式异常")?;
    Ok(if passed == 3 {
        "C1 已通过 3/3；用自己的回环数据完成练习。C2 Axum 尚未交付。".into()
    } else {
        format!("C1 已通过 {passed}/3；下一步检查阶段 {}", passed + 1)
    })
}
pub fn run(args: &[OsString]) -> Result<(), String> {
    let action = args.first().and_then(|x| x.to_str()).unwrap_or("help");
    if args.len() == 1 && matches!(action, "lesson" | "help" | "--help") {
        println!("{GUIDE}");
        return Ok(());
    }
    if args.len() < 2 {
        return Err("advanced-c1 start|prepare|check|next|lesson <工程> [1|2|3|all]".into());
    }
    let root = Path::new(&args[1]);
    match action {
        "start" if args.len() == 2 => init(root),
        "prepare" if args.len() == 2 => prepare(root),
        "next" if args.len() == 2 => {
            println!("{}", next(root)?);
            Ok(())
        }
        "lesson" | "reading" if args.len() == 2 => {
            validate(root)?;
            println!("{GUIDE}");
            Ok(())
        }
        "check" if args.len() == 2 || args.len() == 3 => {
            let stage = match args.get(2).and_then(|s| s.to_str()).unwrap_or("all") {
                "1" => 1,
                "2" => 2,
                "3" | "all" => 3,
                _ => return Err("阶段应为1、2、3或all".into()),
            };
            if check(root, stage)? {
                Ok(())
            } else {
                Err("C1 尚未通过，按日志修复后重试".into())
            }
        }
        _ => Err("advanced-c1 参数无效；运行 advanced-c1 lesson 查看讲义和三级提示".into()),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generator_is_non_overwriting_and_navigation_is_read_only() {
        let tmp = TempWorkspace::new_named("c1-unit").unwrap();
        let root = tmp.0.join("learner");
        init(&root).unwrap();
        assert!(init(&root).is_err());
        assert!(is_project(&root));
        assert_eq!(
            fs::read_to_string(root.join("src/lib.rs"))
                .unwrap()
                .matches("TODO(")
                .count(),
            3
        );
        let before = fs::read_dir(&root).unwrap().count();
        assert!(next(&root).unwrap().contains("尚未检查"));
        assert_eq!(fs::read_dir(&root).unwrap().count(), before);
    }
    #[test]
    fn changed_source_and_contract_invalidate_report() {
        let tmp = TempWorkspace::new_named("c1-unit").unwrap();
        let root = tmp.0.join("learner");
        init(&root).unwrap();
        let source = digest(&sources(&root).unwrap());
        record(
            &root,
            ".c1-reports",
            &format!(
                "C1 Report v1\nsource={source}\ncontract={}\nrequested=3\npassed=3\n",
                contract()
            ),
        )
        .unwrap();
        assert!(next(&root).unwrap().contains("3/3"));
        fs::write(root.join("src/lib.rs"), "// changed").unwrap();
        assert!(next(&root).unwrap().contains("过期"));
    }
    #[cfg(unix)]
    #[test]
    fn linked_source_is_rejected() {
        let tmp = TempWorkspace::new_named("c1-unit").unwrap();
        let root = tmp.0.join("learner");
        init(&root).unwrap();
        fs::remove_file(root.join("src/lib.rs")).unwrap();
        std::os::unix::fs::symlink("/etc/hosts", root.join("src/lib.rs")).unwrap();
        assert!(next(&root).is_err());
    }
}
