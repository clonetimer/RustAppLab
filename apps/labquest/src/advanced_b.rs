//! 进阶 B：独立 Cargo Workspace 的可复用业务库、CLI 与报告。
//! 只检查固定合同，不信任学员工程自带 tests；评测仅运行可信的本地 Rust 源码。
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

const MARKER: &str = "RustAppLab Advanced B v1\n";
const GUIDE: &str = include_str!("../../../projects/advanced-b/ADVANCED_B.md");
const START: &str = include_str!("../../../projects/advanced-b/learner/START_HERE.md");
const FILES: [(&str, &str); 11] = [
    (
        "Cargo.toml",
        include_str!("../../../projects/advanced-b/learner/Cargo.toml"),
    ),
    (
        "Cargo.lock",
        include_str!("../../../projects/advanced-b/learner/Cargo.lock"),
    ),
    (
        "taskcore/Cargo.toml",
        include_str!("../../../projects/advanced-b/learner/taskcore/Cargo.toml"),
    ),
    (
        "taskcore/src/lib.rs",
        include_str!("../../../projects/advanced-b/learner/taskcore/src/lib.rs"),
    ),
    (
        "taskcore/tests/b1.rs",
        include_str!("../../../projects/advanced-b/learner/taskcore/tests/b1.rs"),
    ),
    (
        "taskcore/tests/b2.rs",
        include_str!("../../../projects/advanced-b/learner/taskcore/tests/b2.rs"),
    ),
    (
        "taskcli/Cargo.toml",
        include_str!("../../../projects/advanced-b/learner/taskcli/Cargo.toml"),
    ),
    (
        "taskcli/src/main.rs",
        include_str!("../../../projects/advanced-b/learner/taskcli/src/main.rs"),
    ),
    (
        "taskcli/tests/b3.rs",
        include_str!("../../../projects/advanced-b/learner/taskcli/tests/b3.rs"),
    ),
    (
        "taskreport/tests/report_cli.rs",
        include_str!("../../../projects/advanced-b/learner/taskreport/tests/report_cli.rs"),
    ),
    (
        "taskreport/Cargo.toml",
        include_str!("../../../projects/advanced-b/learner/taskreport/Cargo.toml"),
    ),
];
const REPORT_SRC: &str =
    include_str!("../../../projects/advanced-b/learner/taskreport/src/main.rs");

fn write(path: &Path, text: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?
    }
    fs::write(path, text).map_err(|e| e.to_string())
}
pub fn is_project(root: &Path) -> bool {
    root.join(".applab-advanced-b").is_file()
}
fn validate(root: &Path) -> Result<(), String> {
    let m = fs::symlink_metadata(root).map_err(|e| e.to_string())?;
    if !m.is_dir() || m.file_type().is_symlink() {
        return Err("进阶 B 工程必须是普通目录".into());
    }
    let m = fs::symlink_metadata(root.join(".applab-advanced-b")).map_err(|e| e.to_string())?;
    if !m.is_file() || m.file_type().is_symlink() || m.len() > 128 {
        return Err("进阶 B 工程标记非法".into());
    }
    if fs::read(root.join(".applab-advanced-b")).map_err(|e| e.to_string())? != MARKER.as_bytes() {
        return Err("不是进阶 B 工程".into());
    }
    Ok(())
}
pub fn init(root: &Path) -> Result<(), String> {
    let parent = root
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    if !parent.is_dir() {
        return Err("父目录不存在".into());
    }
    fs::create_dir(root).map_err(|e| format!("仅创建全新工程、不覆盖已存在文件：{e}"))?;
    for &(name, data) in &FILES {
        write(&root.join(name), data)?
    }
    write(&root.join("taskreport/src/main.rs"), REPORT_SRC)?;
    write(&root.join("START_HERE.md"), START)?;
    write(&root.join("ADVANCED_B.md"), GUIDE)?;
    write(&root.join(".applab-advanced-b"), MARKER)?;
    write(&root.join("LEARNING_LOG.md"),"# 进阶 B 学习记录\n\n先预测测试，再实现。记录我自己添加的回归用例、软件数据、Cargo tree 和新功能。\n")?;
    Ok(())
}

fn learner_sources(root: &Path) -> Result<BTreeMap<String, Vec<u8>>, String> {
    validate(root)?;
    fn walk(
        root: &Path,
        dir: &Path,
        files: &mut BTreeMap<String, Vec<u8>>,
        depth: usize,
    ) -> Result<(), String> {
        if depth > 12 || files.len() > 80 {
            return Err("源码数量或深度超过教学限制".into());
        }
        let m = fs::symlink_metadata(dir).map_err(|e| e.to_string())?;
        if !m.is_dir() || m.file_type().is_symlink() {
            return Err("源码文件夹非法".into());
        }
        for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
            let path = entry.map_err(|e| e.to_string())?.path();
            let meta = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
            if meta.file_type().is_symlink() {
                return Err("拒绝源码目录内的符号链接".into());
            }
            if meta.is_dir() {
                walk(root, &path, files, depth + 1)?
            } else if meta.is_file() && path.extension().is_some_and(|e| e == "rs") {
                if meta.len() > 1024 * 1024 {
                    return Err("单个源码文件超过1MiB".into());
                }
                let name = path
                    .strip_prefix(root)
                    .map_err(|e| e.to_string())?
                    .to_str()
                    .ok_or("源码路径不是Unicode")?
                    .replace('\\', "/");
                files.insert(name, fs::read(&path).map_err(|e| e.to_string())?);
            } else {
                return Err("src 内仅接受源码和目录".into());
            }
        }
        Ok(())
    }
    let mut files = BTreeMap::new();
    for dir in ["taskcore/src", "taskcli/src", "taskreport/src"] {
        walk(root, &root.join(dir), &mut files, 0)?
    }
    for key in [
        "taskcore/src/lib.rs",
        "taskcli/src/main.rs",
        "taskreport/src/main.rs",
    ] {
        if !files.contains_key(key) {
            return Err(format!("缺少 {key}"));
        }
    }
    Ok(files)
}
fn fingerprint(files: &BTreeMap<String, Vec<u8>>) -> String {
    let mut h = Sha256::new();
    for (name, data) in files {
        h.update(&(name.len() as u64).to_le_bytes());
        h.update(name.as_bytes());
        h.update(&(data.len() as u64).to_le_bytes());
        h.update(data);
    }
    hex(&h.finalize())
}
fn stage(v: &OsString) -> Result<usize, String> {
    match v.to_str() {
        Some("1") => Ok(1),
        Some("2") => Ok(2),
        Some("3" | "all") => Ok(3),
        _ => Err("检查点应为1、2、3".into()),
    }
}
fn reports(root: &Path) -> Result<PathBuf, String> {
    let path = root.join(".advanced-b-reports");
    match fs::symlink_metadata(&path) {
        Ok(m) if m.is_dir() && !m.file_type().is_symlink() => {}
        Ok(_) => return Err("报告路径不是普通目录".into()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            fs::create_dir(&path).map_err(|e| e.to_string())?
        }
        Err(e) => return Err(e.to_string()),
    }
    Ok(path)
}
pub fn next(root: &Path) -> Result<String, String> {
    let hash = fingerprint(&learner_sources(root)?);
    let report = root.join(".advanced-b-reports");
    if let Ok(metadata) = fs::symlink_metadata(&report) {
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err("报告目录非法（不能是符号链接）".into());
        }
        let mut paths: Vec<_> = fs::read_dir(&report)
            .map_err(|e| e.to_string())?
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "txt"))
            .collect();
        paths.sort();
        if let Some(p) = paths.last() {
            let m = fs::symlink_metadata(p).map_err(|e| e.to_string())?;
            if !m.is_file() || m.file_type().is_symlink() || m.len() > 128 * 1024 {
                return Err("验收记录非法".into());
            }
            let txt = fs::read_to_string(p).map_err(|e| e.to_string())?;
            if txt.starts_with("RustAppLab Advanced B Report v1\n") {
                if !txt.lines().any(|s| s == format!("source_sha256={hash}")) {
                    return Ok("进阶 B：源码改变，历史报告已过期；请从 B1 回归。".into());
                }
                if let Some(passed) = txt
                    .lines()
                    .find_map(|s| {
                        s.strip_prefix("passed=")
                            .and_then(|s| s.parse::<usize>().ok())
                    })
                    .filter(|n| *n <= 3)
                {
                    if passed == 3 {
                        return Ok("进阶 B 3/3 本地检查有通过记录。下一步：自己增加功能、回归测试与个人报告，不能只提交平台PASS。".into());
                    }
                    return Ok(format!("进阶 B：当前源码已通过 B1..B{passed}，建议运行 advanced-b check <目录> {}。",passed+1));
                }
            }
        }
    }
    Ok("进阶 B：从 B1 开始。阅读 START_HERE.md，实现 taskcore/src/lib.rs 的解析和汇总，再执行 advanced-b check <目录> 1。".into())
}
pub fn check(root: &Path, required: usize) -> Result<bool, String> {
    if !(1..=3).contains(&required) {
        return Err("检查点为1、2、3".into());
    }
    let files = learner_sources(root)?;
    let hash = fingerprint(&files);
    let temp = TempWorkspace::new_named("advanced-b")?;
    let project = temp.0.join("project");
    fs::create_dir(&project).map_err(|e| e.to_string())?;
    for &(name, data) in &FILES {
        write(&project.join(name), data)?
    }
    write(&project.join("taskreport/src/main.rs"), REPORT_SRC)?;
    // Tests/manifests always come from the platform, not from the learner workspace.
    for (name, data) in &files {
        let dest = project.join(name);
        fs::create_dir_all(dest.parent().ok_or("源码路径非法")?).map_err(|e| e.to_string())?;
        fs::write(dest, data).map_err(|e| e.to_string())?;
    }
    let mut passed = 0;
    let mut log = String::new();
    for s in 1..=required {
        let (package, test) = match s {
            1 => ("taskcore", "b1"),
            2 => ("taskcore", "b2"),
            _ => ("taskpocket-pro", "b3"),
        };
        println!("[进阶 B{s}] 正在编译和运行固定合同；前序检查会累计回归。");
        let mut cmd = Command::new("cargo");
        cmd.args([
            "test",
            "--locked",
            "--offline",
            "-p",
            package,
            "--test",
            test,
        ]);
        crate::process::learner_cargo(&mut cmd)
            .arg("--")
            .arg("--test-threads=1")
            .current_dir(&project)
            .env("CARGO_TARGET_DIR", temp.0.join("target"));
        match run_bounded(
            &mut cmd,
            Duration::from_secs(120),
            &temp.0,
            "进阶 B 固定行为验收",
        ) {
            Ok(out) => {
                log.push_str(&format!("\n=== B{s} ===\n{}\n{}", out.stdout, out.stderr));
                if out.success {
                    // B3 also runs the independent Markdown executable, not only library calls.
                    if s == 3 {
                        let mut report_cmd = Command::new("cargo");
                        report_cmd.args([
                            "test",
                            "--locked",
                            "--offline",
                            "-p",
                            "taskpocket-report",
                            "--test",
                            "report_cli",
                        ]);
                        crate::process::learner_cargo(&mut report_cmd)
                            .arg("--")
                            .arg("--test-threads=1")
                            .current_dir(&project)
                            .env("CARGO_TARGET_DIR", temp.0.join("target"));
                        match run_bounded(
                            &mut report_cmd,
                            Duration::from_secs(120),
                            &temp.0,
                            "B3 报告程序验收",
                        ) {
                            Ok(report_out) => {
                                log.push_str(&format!(
                                    "\n=== B3-report ===\n{}\n{}",
                                    report_out.stdout, report_out.stderr
                                ));
                                if !report_out.success {
                                    break;
                                }
                            }
                            Err(e) => {
                                log.push_str(&format!("\n=== B3-report 失败 ===\n{e}\n"));
                                break;
                            }
                        }
                    }
                    passed = s
                } else {
                    break;
                }
            }
            Err(e) => {
                log.push_str(&format!("\n=== B{s} 失败 ===\n{e}\n"));
                break;
            }
        }
    }
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let dest = reports(root)?.join(format!("B-{timestamp}-{}.txt", std::process::id()));
    let mut f = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&dest)
        .map_err(|e| e.to_string())?;
    let report=format!("RustAppLab Advanced B Report v1\nsource_sha256={hash}\npassed={passed}\nrequested={required}\n\n固定合同成功 {passed}/{required}。这不是个人设计评审或恶意代码沙箱。\n{log}");
    f.write_all(report.as_bytes()).map_err(|e| e.to_string())?;
    println!("通过阶段 {passed}/{required}；报告：{}", dest.display());
    if passed != required {
        println!("{}", crate::learning::diagnose_text(&log))
    }
    Ok(passed == required)
}
pub fn run(args: &[OsString]) -> Result<(), String> {
    match args.first().and_then(|s| s.to_str()) {
        None | Some("help") => {
            println!("进阶 B · TaskPocket Pro\nadvanced-b start <新目录>\nadvanced-b lesson\nadvanced-b next <学员目录>\nadvanced-b check <学员目录> <1|2|3>");
            Ok(())
        }
        Some("lesson") if args.len() == 1 => {
            println!("{GUIDE}");
            Ok(())
        }
        Some("start") if args.len() == 2 => {
            init(Path::new(&args[1]))?;
            println!("进阶 B 工程已创建：{}", Path::new(&args[1]).display());
            Ok(())
        }
        Some("next") if args.len() == 2 => {
            println!("{}", next(Path::new(&args[1]))?);
            Ok(())
        }
        Some("check") if args.len() == 3 => {
            if check(Path::new(&args[1]), stage(&args[2])?)? {
                Ok(())
            } else {
                Err("当前 B 阶段未通过，请查看报告的第一条错误并继续修改".into())
            }
        }
        _ => Err(
            "用法：advanced-b start <新目录> | lesson | next <目录> | check <目录> <1|2|3>".into(),
        ),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reference_contract_source_embedded() {
        assert!(FILES.iter().any(|x| x.0 == "taskcore/tests/b1.rs"));
        assert!(GUIDE.contains("B4"));
        assert!(START.contains("B1"));
    }
    #[test]
    fn rejects_existing_directory() {
        let root = TempWorkspace::new_named("ab-start-test").unwrap();
        assert!(init(&root.0).is_err())
    }
    #[test]
    fn fixed_input_marker() {
        assert!(MARKER.starts_with("RustAppLab Advanced B"))
    }
}
