//! 将实际构建并运行的小程序输出变成可复算的学习证据。
//! status 只读文件，不会擅自执行学员编辑的代码；只有显式 test-app 才执行 Cargo。
use crate::{
    case::{self, CaseVersion},
    simulation,
};
use safesync::sha256::{hex, Sha256};
use safesync::tree::{self, Kind};
use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use std::time::Duration;

const PROOF_HEADER_V1: &str = "LabQuest-Program-Proof-v1";
const PROOF_HEADER_V2: &str = "LabQuest-Program-Proof-v2";
/// 150ms: 多条错误；400ms: 有等于边界值的请求；750ms: 边界上恰好无错误。
const TRANSFER_TIMEOUTS: [u32; 3] = [150, 400, 750];

fn digest(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex(&hasher.finalize())
}

/// 包含源程序、配置、Cargo.lock 及全部项目文件；文件变化会使旧证据失效。
fn project_digest(root: &Path) -> Result<String, String> {
    let entries =
        tree::scan(&root.join("project")).map_err(|err| format!("无法检查项目文件：{err}"))?;
    let mut hasher = Sha256::new();
    for entry in entries {
        hasher.update(entry.path.as_bytes());
        hasher.update(&[0]);
        match entry.kind {
            Kind::Directory => hasher.update(b"D\n"),
            Kind::File { bytes, sha256 } => {
                hasher.update(b"F");
                hasher.update(bytes.to_string().as_bytes());
                hasher.update(b":");
                hasher.update(sha256.as_bytes());
                hasher.update(b"\n");
            }
        }
    }
    Ok(hex(&hasher.finalize()))
}

fn expected_output(root: &Path) -> Result<String, String> {
    let timeout = simulation::read_timeout(&root.join("project"))?;
    Ok(expected_for_timeout(timeout))
}

fn expected_for_timeout(timeout: u32) -> String {
    let base = simulation::render_log(timeout);
    let errors = simulation::LATENCIES
        .iter()
        .filter(|&&ms| ms > timeout)
        .count();
    format!("{base}SUMMARY requests=12 errors={errors}\n")
}

fn evidence(root: &Path, output: &str) -> Result<String, String> {
    let project = project_digest(root)?;
    let output_hash = digest(output.as_bytes());
    match case::version(root)? {
        CaseVersion::Legacy => Ok(format!(
            "{PROOF_HEADER_V1}\nproject_sha256={project}\noutput_sha256={output_hash}\n"
        )),
        CaseVersion::Transfer => Ok(format!(
            "{PROOF_HEADER_V2}\nproject_sha256={project}\noutput_sha256={output_hash}\ntransfer_timeouts=150,400,750\n"
        )),
    }
}

/// 不引入外部 crate：为新手的构建和运行设置明确的等待上限。
/// 仅能终止直接子进程，不是任意用户代码的沙箱或完整进程树隔离。
fn bounded_output(
    command: &mut Command,
    timeout: Duration,
    context: &str,
) -> Result<Output, String> {
    let temp = crate::capstone::TempWorkspace::new_named("case-output")?;
    crate::process::capture(command, timeout, &temp.0, context)
}

/// 只编译一次，随后使用同一可执行程序运行多个输入；不修改 config 或源码。
fn build_program(manifest: &Path, build: &Path) -> Result<std::path::PathBuf, String> {
    let mut command = Command::new("cargo");
    command
        .args([
            "build",
            "--quiet",
            "--offline",
            "--locked",
            "--manifest-path",
        ])
        .arg(manifest)
        .arg("--target-dir")
        .arg(build)
        .env_remove("CARGO_BUILD_TARGET"); // 实际运行本机学生程序，不做交叉编译
    crate::process::learner_cargo(&mut command);
    let result = bounded_output(&mut command, Duration::from_secs(120), "Rust 编译")?;
    if !result.status.success() {
        let excerpt: String = String::from_utf8_lossy(&result.stderr)
            .chars()
            .take(2400)
            .collect();
        return Err(format!("独立 Rust 项目编译失败：{excerpt}"));
    }
    let exe = build
        .join("debug")
        .join(format!("case-gateway{}", std::env::consts::EXE_SUFFIX));
    if !exe.is_file() {
        return Err(format!(
            "编译成功但缺少案例可执行文件 {}。请保持 case-gateway 包名不变",
            exe.display()
        ));
    }
    Ok(exe)
}

fn run_program(exe: &Path, override_timeout: Option<u32>) -> Result<String, String> {
    let mut command = Command::new(exe);
    if let Some(timeout) = override_timeout {
        command.args(["--timeout-ms", &timeout.to_string()]);
    }
    let context = override_timeout.map_or_else(
        || "当前配置".to_string(),
        |value| format!("迁移输入 timeout_ms={value}"),
    );
    let result = bounded_output(
        &mut command,
        Duration::from_secs(5),
        &format!("Rust 程序在{context}下运行"),
    )?;
    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        let excerpt: String = stderr.chars().take(2400).collect();
        return Err(format!("独立 Rust 程序在{context}下运行失败：{excerpt}"));
    }
    if result.stdout.len() > 64 * 1024 {
        return Err("程序输出超过 64 KiB，拒绝记录验收证据".into());
    }
    String::from_utf8(result.stdout).map_err(|_| "程序输出不是 UTF-8 文本".to_string())
}

fn assert_output(actual: &str, expected: &str, scenario: &str) -> Result<(), String> {
    if actual != expected {
        return Err(format!(
            "{scenario}未通过：编译运行成功，但日志或 SUMMARY 与预期不一致（实际 {} 行，预期 {} 行）。\n请从 LATENCIES 和实际 timeout 动态计算 ERROR 数量；不能写死 errors=0。注意判定条件是 latency > timeout，不是 >=。",
            actual.lines().count(), expected.lines().count()
        ));
    }
    Ok(())
}

/// test-app 是用户显式调用的代码执行入口。迁移测试不会写入 project/。
/// status / board / next 仍然仅读取证据，绝不执行学员代码。
pub fn test_app(root: &Path) -> Result<(), String> {
    case::ensure_case(root)?;
    let root = fs::canonicalize(root).map_err(|err| format!("无法定位案例：{err}"))?;
    let manifest = root.join("project/Cargo.toml");
    let build = root.join("build");
    let executable = build_program(&manifest, &build)?;
    let actual = run_program(&executable, None)?;
    let expected = expected_output(&root)?;
    assert_output(&actual, &expected, "当前配置")?;
    if case::version(&root)? == CaseVersion::Transfer {
        for timeout in TRANSFER_TIMEOUTS {
            let output = run_program(&executable, Some(timeout))?;
            assert_output(
                &output,
                &expected_for_timeout(timeout),
                &format!("迁移测试 timeout_ms={timeout}"),
            )?;
        }
    }
    let proof = evidence(&root, &actual)?;
    // 所有用例通过后才会写入新的证据；失败时保留现场。
    case::write_owned(&root.join("logs/program.log"), &actual)?;
    case::write_owned(&root.join("deliverables/PROGRAM_PROOF.txt"), &proof)?;
    Ok(())
}

/// 只读取本地证据，不重新执行学员代码。拒绝过期或与项目当前状态不一致的证明。
pub fn check(root: &Path) -> bool {
    let Ok(program_output) = fs::read_to_string(root.join("logs/program.log")) else {
        return false;
    };
    let Ok(saved_proof) = fs::read_to_string(root.join("deliverables/PROGRAM_PROOF.txt")) else {
        return false;
    };
    let Ok(expected) = expected_output(root) else {
        return false;
    };
    if program_output != expected {
        return false;
    }
    evidence(root, &program_output).is_ok_and(|proof| proof == saved_proof)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    #[test]
    fn infinite_loop_like_process_is_terminated_with_explanation() {
        let mut command = Command::new("sh");
        command.args(["-c", "sleep 2"]);
        let error =
            bounded_output(&mut command, Duration::from_millis(100), "模拟阻塞").unwrap_err();
        assert!(error.contains("超过"), "{error}");
        assert!(error.contains("无限循环"), "{error}");
    }

    #[test]
    fn fingerprints_change_when_content_changes() {
        assert_ne!(digest(b"hello"), digest(b"hello!"));
        assert_eq!(
            digest(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
