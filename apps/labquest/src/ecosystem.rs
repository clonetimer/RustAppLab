//! Optional dependency preparation. Does not participate in or download during base courses.
use crate::capstone::{run_bounded, RunOutput, TempWorkspace};
use std::{ffi::OsString, fs, path::Path, process::Command, time::Duration};
const MARKER: &str = "RustAppLab network preparation v1\n";
const MANIFEST: &str = include_str!("../ecosystem/Cargo.toml");
const PROBE: &str = include_str!("../ecosystem/probe.rs");
const PACK: &str = include_str!("../ecosystem/PACK_WINDOWS.ps1");
const README:&str="本目录是网络依赖准备包，不是已完成的 NetFetch/ServiceDesk。\nprepare 会联网获取依赖并执行受信任的依赖构建脚本；不会关闭TLS校验，不会安装系统工具或扫描个人目录。\n只有 STATUS.md 中写明已通过的步骤才算本机证据。Cargo.lock 锁定传递版本；vendor 保存依赖源代码和许可证；.cargo/config.toml 指向该目录。\n无真实联网编译证据时不得把此探针计作课程作品。Windows 打包运行 PACK_WINDOWS.ps1；输出ZIP在本目录旁，不包含编译缓存或用户 Cargo 配置。\n";
fn write_owned(path: &Path, text: &str) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(m) if !m.is_file() || m.file_type().is_symlink() => {
            return Err(format!("不覆盖特殊文件：{}", path.display()))
        }
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.to_string()),
    }
    fs::write(path, text).map_err(|e| e.to_string())
}
fn directory(path: &Path) -> Result<(), String> {
    let m = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if !m.is_dir() || m.file_type().is_symlink() {
        return Err("准备目录不能是符号链接".into());
    }
    Ok(())
}
pub fn init(root: &Path) -> Result<(), String> {
    fs::create_dir(root).map_err(|e| format!("准备包需要全新目录（不会覆盖已有文件）：{e}"))?;
    for dir in ["src", "logs"] {
        fs::create_dir(root.join(dir)).map_err(|e| e.to_string())?;
    }
    for (path, text) in [
        (".rustapplab-network-kit", MARKER),
        ("Cargo.toml", MANIFEST),
        ("src/lib.rs", PROBE),
        ("README.txt", README),
        ("PACK_WINDOWS.ps1", PACK),
        (
            "STATUS.md",
            "# 网络依赖准备\n\n状态：未执行。没有下载依赖，没有通过网络测试。\n",
        ),
    ] {
        write_owned(&root.join(path), text)?;
    }
    Ok(())
}
fn require_kit(root: &Path) -> Result<(), String> {
    directory(root)?;
    for dir in ["src", "logs"] {
        directory(&root.join(dir))?;
    }
    for (path, expected) in [
        (".rustapplab-network-kit", MARKER),
        ("Cargo.toml", MANIFEST),
        ("src/lib.rs", PROBE),
    ] {
        let p = root.join(path);
        let m = fs::symlink_metadata(&p).map_err(|e| e.to_string())?;
        if !m.is_file()
            || m.file_type().is_symlink()
            || m.len() > 64 * 1024
            || fs::read_to_string(&p).map_err(|e| e.to_string())? != expected
        {
            return Err(format!("准备包标记或源码已改变，不覆盖/执行：{path}"));
        }
    }
    Ok(())
}
#[derive(Clone)]
struct Step {
    label: &'static str,
    program: &'static str,
    args: Vec<String>,
}
fn steps(root: &Path, target: &Path) -> Vec<Step> {
    let mut list = vec![
        Step {
            label: "rustc",
            program: "rustc",
            args: vec!["--version".into(), "--verbose".into()],
        },
        Step {
            label: "cargo",
            program: "cargo",
            args: vec!["--version".into()],
        },
    ];
    if !root.join("Cargo.lock").is_file() {
        list.push(Step {
            label: "resolve",
            program: "cargo",
            args: vec!["generate-lockfile".into()],
        });
    }
    for (label, args) in [
        ("fetch", vec!["fetch", "--locked"]),
        (
            "vendor",
            vec!["vendor", "--locked", "--versioned-dirs", "vendor"],
        ),
    ] {
        list.push(Step {
            label,
            program: "cargo",
            args: args.into_iter().map(str::to_string).collect(),
        });
    }
    list.push(Step {
        label: "offline-test",
        program: "cargo",
        args: vec![
            "test".into(),
            "--locked".into(),
            "--offline".into(),
            "--all-targets".into(),
            "--target-dir".into(),
            target.to_string_lossy().into_owned(),
        ],
    });
    list
}
fn pipeline(
    root: &Path,
    target: &Path,
    mut execute: impl FnMut(&Step) -> Result<RunOutput, String>,
) -> Result<(), String> {
    require_kit(root)?;
    let mut status="# 网络依赖准备 · 当前机器的实测结果\n\n这里只验证依赖探针，不能认定 NetFetch/ServiceDesk 已完成；不代表Windows/Linux互相通过。\n\n".to_string();
    write_owned(
        &root.join("STATUS.md"),
        &format!("{status}状态：运行中，尚未通过。\n"),
    )?;
    for step in steps(root, target) {
        println!("[{}] {} {}", step.label, step.program, step.args.join(" "));
        let result = execute(&step);
        let out = match result {
            Ok(out) => out,
            Err(error) => {
                status.push_str(&format!(
                    "- {}: FAIL {error}\n\n状态：失败，后续步骤未执行。\n",
                    step.label
                ));
                write_owned(&root.join("STATUS.md"), &status)?;
                return Err(error);
            }
        };
        write_owned(
            &root.join("logs").join(format!("{}.txt", step.label)),
            &format!(
                "COMMAND: {} {}\nSUCCESS: {}\nSTDOUT:\n{}\nSTDERR:\n{}",
                step.program,
                step.args.join(" "),
                out.success,
                out.stdout,
                out.stderr
            ),
        )?;
        if !out.success {
            status.push_str(&format!("- {}: FAIL；查看 logs/{}.txt\n\n状态：失败，后续步骤未执行。不是学员 TODO 错误。\n",step.label,step.label));
            write_owned(&root.join("STATUS.md"), &status)?;
            return Err(format!("依赖准备停在 {}；日志在 {}。保留目录，修复网络/工具链后可原路径重试，不要删除个人工程。",step.label,root.display()));
        }
        if step.label == "vendor" {
            directory(&root.join("vendor"))?;
            let config = root.join(".cargo");
            if config.exists() {
                directory(&config)?;
            } else {
                fs::create_dir(&config).map_err(|e| e.to_string())?;
            }
            // Use a known portable relative path, not machine-specific paths from Cargo stdout.
            write_owned(&config.join("config.toml"),"[source.crates-io]\nreplace-with = \"vendored-sources\"\n[source.vendored-sources]\ndirectory = \"vendor\"\n")?;
        }
        status.push_str(&format!("- {}: PASS\n", step.label));
        write_owned(
            &root.join("STATUS.md"),
            &format!("{status}\n状态：运行中，仍以最后阶段结果为准。\n"),
        )?;
    }
    status.push_str("\n状态：PASS（本机依赖探针）。网络课程尚未完成；测试不包含外网HTTPS站点验收。\nWindows运行 PACK_WINDOWS.ps1 生成包含锁文件、vendor 和日志的离线包。\n");
    write_owned(&root.join("STATUS.md"), &status)
}
pub fn prepare(root: &Path) -> Result<(), String> {
    match fs::symlink_metadata(root) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => init(root)?,
        Err(e) => return Err(e.to_string()),
        Ok(_) => require_kit(root)?,
    }
    if root.join("build.rs").exists() {
        return Err("准备包不允许附加 build.rs；请用新目录重试".into());
    }
    let target = TempWorkspace::new_named("network-build")?;
    pipeline(root, &target.0.join("target"), |step| {
        let mut command = Command::new(step.program);
        command
            .args(&step.args)
            .current_dir(root)
            .env_remove("CARGO_BUILD_TARGET")
            .env("CARGO_TERM_COLOR", "never");
        run_bounded(
            &mut command,
            Duration::from_secs(900),
            &target.0,
            step.label,
        )
    })
}
pub fn run(args: &[OsString]) -> Result<(), String> {
    match args.first().and_then(|x| x.to_str()).unwrap_or("plan") {
        "plan" if args.len() <= 1 => {
            println!("网络课程准备：Tokio 1.48.0 / Axum 0.8.6 / Reqwest 0.12.24 / Serde 1.0.228 / serde_json 1.0.145 候选组合。不是最新版声明或安全审计结论。\n基础课程不下载这些依赖。init 仅生成探针；prepare 明确联网并在本机执行依赖构建脚本，生成锁文件和vendor后运行离线探针测试。\nNetFetch/ServiceDesk 同步标准库基础版已可离线运行；Tokio/Reqwest/Axum 异步框架版仍未完成。没有真实 Cargo.lock/vendor 就不声称框架依赖已准备好。\n命令：ecosystem init <新目录> | prepare <新目录或原准备目录>\nWindows打包：在生成目录运行 PACK_WINDOWS.ps1。只打包生成的目录，不包含你的其他项目。");
            Ok(())
        }
        "init" if args.len() == 2 => init(Path::new(&args[1])),
        "prepare" if args.len() == 2 => prepare(Path::new(&args[1])),
        _ => Err("用法：ecosystem plan | init <新目录> | prepare <准备目录>".into()),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn kit() -> TempWorkspace {
        let tmp = TempWorkspace::new_named("network-unit").unwrap();
        init(&tmp.0.join("kit")).unwrap();
        tmp
    }
    #[test]
    fn seed_is_offline_and_never_overwrites() {
        let t = kit();
        let p = t.0.join("kit");
        assert!(init(&p).is_err());
        assert!(!p.join("vendor").exists());
        assert!(fs::read_to_string(p.join("STATUS.md"))
            .unwrap()
            .contains("未执行"));
    }
    #[test]
    fn failure_stops_pipeline_and_never_claims_pass() {
        let t = kit();
        let p = t.0.join("kit");
        let mut names = Vec::new();
        assert!(pipeline(&p, &t.0.join("target"), |s| {
            names.push(s.label);
            Ok(RunOutput {
                success: s.label != "resolve",
                stdout: String::new(),
                stderr: "simulated network failure".into(),
            })
        })
        .is_err());
        assert_eq!(names, vec!["rustc", "cargo", "resolve"]);
        let status = fs::read_to_string(p.join("STATUS.md")).unwrap();
        assert!(status.contains("resolve: FAIL"));
        assert!(!status.contains("状态：PASS"));
        assert!(!p.join(".cargo").exists());
    }
    #[test]
    fn orchestration_success_with_fake_runner_is_not_real_dependency_test() {
        let t = kit();
        let p = t.0.join("kit");
        pipeline(&p, &t.0.join("target"), |s| {
            if s.label == "resolve" {
                fs::write(p.join("Cargo.lock"), "test fixture").unwrap();
            }
            if s.label == "vendor" {
                fs::create_dir(p.join("vendor")).unwrap();
            }
            Ok(RunOutput {
                success: true,
                stdout: String::new(),
                stderr: String::new(),
            })
        })
        .unwrap();
        assert!(p.join(".cargo/config.toml").is_file());
        assert!(!steps(&p, &t.0).iter().any(|s| s.label == "resolve"));
    }
    #[test]
    fn changed_probe_is_not_executed() {
        let t = kit();
        let p = t.0.join("kit");
        fs::write(p.join("src/lib.rs"), "changed").unwrap();
        assert!(pipeline(&p, &t.0, |_| panic!("must not execute")).is_err());
    }
    #[test]
    fn spawn_error_is_recorded() {
        let t = kit();
        let p = t.0.join("kit");
        assert!(pipeline(&p, &t.0, |_| Err("missing cargo".into())).is_err());
        assert!(fs::read_to_string(p.join("STATUS.md"))
            .unwrap()
            .contains("missing cargo"));
    }
}
