//! B4's registry dependency boundary: explicit preparation and fixed offline tests.
use crate::capstone::{run_bounded, TempWorkspace};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

const MARKER: &str = "RustAppLab Advanced B dependencies v1\n";
const MARKER_FILE: &str = ".rustapplab-json-dependencies";
const LEARNER: [(&str, &str); 6] = [
    (
        "Cargo.toml",
        include_str!("../../../projects/advanced-b/learner/json-export/Cargo.toml"),
    ),
    (
        "Cargo.lock",
        include_str!("../../../projects/advanced-b/learner/json-export/Cargo.lock"),
    ),
    (
        "src/lib.rs",
        include_str!("../../../projects/advanced-b/learner/json-export/src/lib.rs"),
    ),
    (
        "src/main.rs",
        include_str!("../../../projects/advanced-b/learner/json-export/src/main.rs"),
    ),
    (
        "tests/b4_api.rs",
        include_str!("../../../projects/advanced-b/learner/json-export/tests/b4_api.rs"),
    ),
    (
        "tests/b4_cli.rs",
        include_str!("../../../projects/advanced-b/learner/json-export/tests/b4_cli.rs"),
    ),
];
const REFERENCE: [(&str, &str); 6] = [
    (
        "Cargo.toml",
        include_str!("../../../projects/advanced-b/reference/json-export/Cargo.toml"),
    ),
    (
        "Cargo.lock",
        include_str!("../../../projects/advanced-b/reference/json-export/Cargo.lock"),
    ),
    (
        "src/lib.rs",
        include_str!("../../../projects/advanced-b/reference/json-export/src/lib.rs"),
    ),
    (
        "src/main.rs",
        include_str!("../../../projects/advanced-b/reference/json-export/src/main.rs"),
    ),
    (
        "tests/b4_api.rs",
        include_str!("../../../projects/advanced-b/reference/json-export/tests/b4_api.rs"),
    ),
    (
        "tests/b4_cli.rs",
        include_str!("../../../projects/advanced-b/reference/json-export/tests/b4_cli.rs"),
    ),
];

pub(crate) fn install(root: &Path, reference: bool) -> Result<(), String> {
    let extension = root.join("json-export");
    fs::create_dir(&extension)
        .map_err(|e| format!("JSON 扩展仅创建全新目录，不覆盖已有文件：{e}"))?;
    for (name, text) in if reference { &REFERENCE } else { &LEARNER } {
        let path = extension.join(name);
        fs::create_dir_all(path.parent().ok_or("模板路径非法")?).map_err(|e| e.to_string())?;
        fs::write(path, text).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn normal_directory(path: &Path) -> Result<(), String> {
    let m = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if !m.is_dir() || m.file_type().is_symlink() {
        return Err(format!("需要普通目录，拒绝符号链接：{}", path.display()));
    }
    Ok(())
}
fn write_owned(path: &Path, text: &str) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(m) if !m.is_file() || m.file_type().is_symlink() => {
            return Err(format!("不覆盖特殊文件：{}", path.display()));
        }
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.to_string()),
    }
    fs::write(path, text).map_err(|e| e.to_string())
}
fn preparation_directory(root: &Path) -> Result<PathBuf, String> {
    let path = root.join(".advanced-b-dependencies");
    match fs::symlink_metadata(&path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            fs::create_dir(&path).map_err(|e| e.to_string())?;
            write_owned(&path.join(MARKER_FILE), MARKER)?;
            fs::create_dir(path.join("logs")).map_err(|e| e.to_string())?;
        }
        Err(e) => return Err(e.to_string()),
        Ok(_) => {
            normal_directory(&path)?;
            let marker = path.join(MARKER_FILE);
            let m = fs::symlink_metadata(&marker).map_err(|e| e.to_string())?;
            if !m.is_file()
                || m.file_type().is_symlink()
                || m.len() > 128
                || fs::read(marker).map_err(|e| e.to_string())? != MARKER.as_bytes()
            {
                return Err("依赖准备目录标记非法；不会覆盖现有内容".into());
            }
            normal_directory(&path.join("logs"))?;
        }
    }
    Ok(path)
}

/// Only platform-owned files go into the temporary build; even taskcore is fixed.
pub(crate) fn prepare(root: &Path) -> Result<(), String> {
    let state = preparation_directory(root)?;
    let mut status =
        "# B4 依赖准备 · 当前机器\n\n使用固定 reference 源码与锁文件；不是 B4 通过记录。\n\n"
            .to_string();
    write_owned(
        &state.join("STATUS.md"),
        &format!("{status}状态：运行中，尚未通过。\n"),
    )?;
    let temp = TempWorkspace::new_named("b4-prepare")?;
    let project = temp.0.join("project");
    fs::create_dir_all(project.join("taskcore/src")).map_err(|e| e.to_string())?;
    fs::write(
        project.join("taskcore/Cargo.toml"),
        include_str!("../../../projects/advanced-b/reference/taskcore/Cargo.toml"),
    )
    .map_err(|e| e.to_string())?;
    fs::write(
        project.join("taskcore/src/lib.rs"),
        include_str!("../../../projects/advanced-b/reference/taskcore/src/lib.rs"),
    )
    .map_err(|e| e.to_string())?;
    install(&project, true)?;
    let steps: [(&str, &str, &[&str]); 7] = [
        ("rustc", "rustc", &["--version", "--verbose"]),
        ("cargo", "cargo", &["--version"]),
        ("fetch", "cargo", &["fetch", "--locked"]),
        (
            "offline-default",
            "cargo",
            &[
                "test",
                "--locked",
                "--offline",
                "--all-targets",
                "--no-default-features",
            ],
        ),
        (
            "offline-strict",
            "cargo",
            &[
                "test",
                "--locked",
                "--offline",
                "--all-targets",
                "--no-default-features",
                "--features",
                "strict-titles",
            ],
        ),
        (
            "doc-default",
            "cargo",
            &[
                "test",
                "--locked",
                "--offline",
                "--doc",
                "--no-default-features",
            ],
        ),
        (
            "doc-strict",
            "cargo",
            &[
                "test",
                "--locked",
                "--offline",
                "--doc",
                "--no-default-features",
                "--features",
                "strict-titles",
            ],
        ),
    ];
    for (label, program, args) in steps {
        println!("[B4 依赖 {label}] {program} {}", args.join(" "));
        let mut command = Command::new(program);
        command
            .args(args)
            .current_dir(project.join("json-export"))
            .env("CARGO_TARGET_DIR", temp.0.join("target"))
            .env("CARGO_TERM_COLOR", "never")
            .env_remove("CARGO_BUILD_TARGET");
        if args[0] == "test" {
            crate::process::learner_cargo(&mut command).args(["--", "--test-threads=1"]);
        }
        let result = run_bounded(&mut command, Duration::from_secs(300), &temp.0, label);
        let (success, details) = match result {
            Ok(out) => (
                out.success,
                format!("STDOUT:\n{}\nSTDERR:\n{}", out.stdout, out.stderr),
            ),
            Err(error) => (false, error),
        };
        write_owned(
            &state.join("logs").join(format!("{label}.txt")),
            &format!(
                "COMMAND: {program} {}\nSUCCESS: {success}\n{details}\n",
                args.join(" ")
            ),
        )?;
        status.push_str(&format!(
            "- {label}: {}；日志 logs/{label}.txt\n",
            if success { "PASS" } else { "FAIL" }
        ));
        if !success {
            status.push_str(
                "\n状态：失败，后续步骤未执行；这是依赖或工具环境问题，不是学员 TODO 错误。\n",
            );
            write_owned(&state.join("STATUS.md"), &status)?;
            return Err(format!("B4 依赖准备停在 {label}。查看 {}；修复网络或工具链后可在原工程重新运行 prepare-json。\n{details}", state.display()));
        }
        write_owned(
            &state.join("STATUS.md"),
            &format!("{status}\n状态：运行中，尚未通过。\n"),
        )?;
    }
    status.push_str("\n状态：PASS（本机固定依赖图与 reference 行为已验证）。不是 B4 通过记录。\n请实现 json-export/src 后运行 advanced-b check <目录> 4。更换 Cargo/工具链后若缓存缺失，需再次显式准备。\n");
    write_owned(&state.join("STATUS.md"), &status)?;
    println!("B4 依赖准备通过；状态和日志：{}", state.display());
    Ok(())
}

pub(crate) struct CheckResult {
    pub(crate) passed: bool,
    pub(crate) environment_failure: bool,
}
pub(crate) fn check(project: &Path, scratch: &Path, log: &mut String) -> CheckResult {
    let extension = project.join("json-export");
    let mut metadata = Command::new("cargo");
    metadata
        .args(["metadata", "--locked", "--offline", "--format-version", "1"])
        .current_dir(&extension)
        .env("CARGO_TERM_COLOR", "never")
        .env_remove("CARGO_BUILD_TARGET");
    let failure = match run_bounded(
        &mut metadata,
        Duration::from_secs(120),
        scratch,
        "B4 依赖预检",
    ) {
        Ok(out) if out.success => None,
        Ok(out) => Some(format!("{}\n{}", out.stdout, out.stderr)),
        Err(error) => Some(error),
    };
    if let Some(error) = failure {
        let missing = error.contains("no matching package")
            || error.contains("failed to download")
            || error.contains("attempting to make an HTTP request, but --offline was specified");
        let hint = if missing {
            "B4 离线依赖缓存不完整。请显式运行 advanced-b prepare-json <目录>，成功后重新 check 4；这不是学员 TODO 失败。"
        } else {
            "B4 依赖预检失败；请依据原始错误检查 Cargo、工具链、锁文件或缓存。需要准备依赖时显式运行 advanced-b prepare-json <目录>；这不是学员 TODO 失败。"
        };
        println!("{hint}");
        log.push_str(&format!("\n=== B4-dependencies ===\n{hint}\n{error}\n"));
        return CheckResult {
            passed: false,
            environment_failure: true,
        };
    }
    let checks: [(&str, &[&str]); 4] = [
        (
            "B4-default",
            &[
                "test",
                "--locked",
                "--offline",
                "--all-targets",
                "--no-default-features",
            ],
        ),
        (
            "B4-strict",
            &[
                "test",
                "--locked",
                "--offline",
                "--all-targets",
                "--no-default-features",
                "--features",
                "strict-titles",
            ],
        ),
        (
            "B4-doc-default",
            &[
                "test",
                "--locked",
                "--offline",
                "--doc",
                "--no-default-features",
            ],
        ),
        (
            "B4-doc-strict",
            &[
                "test",
                "--locked",
                "--offline",
                "--doc",
                "--no-default-features",
                "--features",
                "strict-titles",
            ],
        ),
    ];
    for (label, args) in checks {
        println!("[{label}] 正在离线运行固定 JSON 导出合同。");
        let mut command = Command::new("cargo");
        command
            .args(args)
            .current_dir(&extension)
            .env("CARGO_TARGET_DIR", scratch.join("target-json"))
            .env("CARGO_TERM_COLOR", "never");
        crate::process::learner_cargo(&mut command).args(["--", "--test-threads=1"]);
        let success = match run_bounded(&mut command, Duration::from_secs(180), scratch, label) {
            Ok(out) => {
                log.push_str(&format!(
                    "\n=== {label} ===\n{}\n{}",
                    out.stdout, out.stderr
                ));
                out.success
            }
            Err(error) => {
                log.push_str(&format!("\n=== {label} 失败 ===\n{error}\n"));
                false
            }
        };
        if !success {
            return CheckResult {
                passed: false,
                environment_failure: false,
            };
        }
    }
    CheckResult {
        passed: true,
        environment_failure: false,
    }
}
