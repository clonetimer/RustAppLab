//! Bounded child execution for trusted learner projects, not a security sandbox.
//!
//! Redirect to uniquely created files: waiting for exit before draining a pipe can
//! block a verbose child. Limits are sampled, not a strict disk quota. Only the
//! direct child is terminated; descendants and hostile build scripts are out of scope.
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

pub(crate) const MAX_OUTPUT: u64 = 1024 * 1024;
static NEXT: AtomicU64 = AtomicU64::new(0);

struct Captures(Vec<PathBuf>);
impl Drop for Captures {
    fn drop(&mut self) {
        for path in &self.0 {
            let _ = fs::remove_file(path);
        }
    }
}
struct RunningChild(Child);
impl Drop for RunningChild {
    fn drop(&mut self) {
        // try_wait already reaps completed children; kill on success is harmless.
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn read_bounded(path: &Path) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|e| e.to_string())?
        .take(MAX_OUTPUT + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_OUTPUT {
        return Err("子进程输出超过 1 MiB，请减少打印内容".into());
    }
    Ok(bytes)
}

// Preserve a bounded suffix of each log before Captures removes the scratch files.
// This is best-effort diagnostics only, never evidence of a completed test.
fn timeout_tail(path: &Path) -> String {
    const LIMIT: u64 = 4096;
    let read = || -> std::io::Result<String> {
        let mut file = File::open(path)?;
        let length = file.metadata()?.len();
        let start = length.saturating_sub(LIMIT);
        file.seek(SeekFrom::Start(start))?;
        let mut bytes = Vec::new();
        file.take(LIMIT).read_to_end(&mut bytes)?;
        let mut text = String::new();
        if start > 0 {
            text.push_str("[仅显示末尾 4096 字节]\n");
        }
        text.push_str(&String::from_utf8_lossy(&bytes));
        Ok(text)
    };
    match read() {
        Ok(text) if text.is_empty() => "(无已捕获输出)".into(),
        Ok(text) => text,
        Err(error) => format!("(无法读取部分输出：{error})"),
    }
}

/// Use a separate target directory at each call site. Fixed build parallelism
/// avoids multiplying compiler threads for every parallel assessment test.
pub(crate) fn learner_cargo(command: &mut Command) -> &mut Command {
    command
        .args(["--jobs", "2"])
        // Do not let the parent release gate's CARGO_TARGET_DIR redirect
        // learner outputs into the root workspace. Each call site explicitly
        // selects its own isolated --target-dir.
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_BUILD_TARGET")
        .env_remove("CARGO_MAKEFLAGS")
        .env_remove("MAKEFLAGS")
        .env_remove("MFLAGS")
}

pub(crate) fn capture(
    command: &mut Command,
    timeout: Duration,
    scratch: &Path,
    label: &str,
) -> Result<Output, String> {
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    let mut files = Captures(Vec::new());
    let mut streams = Vec::new();
    for suffix in ["out", "err"] {
        let path = scratch.join(format!("child-{}-{id}.{suffix}", std::process::id()));
        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .map_err(|e| format!("无法创建{label}日志：{e}"))?;
        files.0.push(path);
        streams.push(file);
    }
    let stderr = streams.pop().expect("two log files");
    let stdout = streams.pop().expect("two log files");
    let spawned = command
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
        .map_err(|e| format!("启动{label}失败：{e}"));
    // Do not keep handles on Command: Windows cannot remove an open log file.
    command.stdout(Stdio::null()).stderr(Stdio::null());
    let mut child = RunningChild(spawned?);
    let started = Instant::now();
    loop {
        if let Some(status) = child.0.try_wait().map_err(|e| e.to_string())? {
            return Ok(Output {
                status,
                stdout: read_bounded(&files.0[0])?,
                stderr: read_bounded(&files.0[1])?,
            });
        }
        if files
            .0
            .iter()
            .any(|p| fs::metadata(p).is_ok_and(|m| m.len() > MAX_OUTPUT))
        {
            return Err(format!(
                "{label}输出超过 1 MiB，已尝试终止直接子进程；请减少打印内容"
            ));
        }
        if started.elapsed() >= timeout {
            // Retain the existing deadline and failure decision. Reap the direct
            // child before reading whatever it already wrote; descendants are
            // still outside this bounded runner's termination guarantee.
            let elapsed = started.elapsed();
            let _ = child.0.kill();
            let _ = child.0.wait();
            let stdout = timeout_tail(&files.0[0]);
            let stderr = timeout_tail(&files.0[1]);
            return Err(format!(
                "{label}超过 {} 秒（观测耗时 {:.3} 秒），已尝试终止直接子进程；检查无限循环或阻塞等待。这不是安全沙箱。\n--- 部分 stdout（不是通过证据）---\n{stdout}\n--- 部分 stderr ---\n{stderr}",
                timeout.as_secs_f32(),
                elapsed.as_secs_f64()
            ));
        }
        thread::sleep(Duration::from_millis(10));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capstone::TempWorkspace;

    // This inert entry is also invoked in a subprocess using an exact filter.
    #[test]
    fn child_fixture() {
        match std::env::var("RUSTAPPLAB_CAPTURE_TEST_MODE").as_deref() {
            Ok("verbose") => {
                use std::io::Write;
                std::io::stdout()
                    .write_all(&vec![b'a'; 192 * 1024])
                    .unwrap();
                std::io::stderr()
                    .write_all(&vec![b'b'; 192 * 1024])
                    .unwrap();
            }
            Ok("oversize") => {
                use std::io::Write;
                std::io::stdout()
                    .write_all(&vec![b'x'; MAX_OUTPUT as usize + 1])
                    .unwrap();
            }
            Ok("wait") => thread::sleep(Duration::from_secs(10)),
            Ok("stdin") => {
                let mut input = String::new();
                std::io::stdin().read_to_string(&mut input).unwrap();
                assert!(input.is_empty());
                println!("STDIN_CLOSED");
            }
            _ => {}
        }
    }
    fn command(mode: &str) -> Command {
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args(["--exact", "process::tests::child_fixture", "--nocapture"])
            .env("RUSTAPPLAB_CAPTURE_TEST_MODE", mode);
        command
    }
    #[test]
    fn learner_cargo_does_not_inherit_parent_build_destination() {
        let mut cmd = Command::new("cargo");
        learner_cargo(&mut cmd);
        let removed: Vec<_> = cmd
            .get_envs()
            .filter_map(|(name, value)| {
                if value.is_none() {
                    Some(name.to_string_lossy().into_owned())
                } else {
                    None
                }
            })
            .collect();
        assert!(removed.iter().any(|x| x == "CARGO_TARGET_DIR"));
        assert!(removed.iter().any(|x| x == "CARGO_BUILD_TARGET"));
    }
    #[test]
    fn verbose_stdout_and_stderr_do_not_block_and_files_are_cleaned() {
        let temp = TempWorkspace::new_named("capture").unwrap();
        let output = capture(
            &mut command("verbose"),
            Duration::from_secs(3),
            &temp.0,
            "verbose",
        )
        .unwrap();
        assert!(output.status.success());
        assert!(output.stdout.len() >= 192 * 1024);
        assert!(output.stderr.len() >= 192 * 1024);
        assert_eq!(fs::read_dir(&temp.0).unwrap().count(), 0);
    }
    #[test]
    fn excessive_output_is_not_a_success() {
        let temp = TempWorkspace::new_named("capture-limit").unwrap();
        let error = capture(
            &mut command("oversize"),
            Duration::from_secs(3),
            &temp.0,
            "output",
        )
        .unwrap_err();
        assert!(error.contains("1 MiB"), "{error}");
        assert_eq!(fs::read_dir(&temp.0).unwrap().count(), 0);
    }
    #[test]
    fn timeout_kills_direct_child_and_closes_logs() {
        let temp = TempWorkspace::new_named("capture-timeout").unwrap();
        let start = Instant::now();
        let error = capture(
            &mut command("wait"),
            Duration::from_millis(100),
            &temp.0,
            "wait",
        )
        .unwrap_err();
        assert!(error.contains("超过"));
        assert!(error.contains("部分 stdout") && error.contains("部分 stderr"));
        assert!(start.elapsed() < Duration::from_secs(5));
        assert_eq!(fs::read_dir(&temp.0).unwrap().count(), 0);
    }
    #[test]
    fn timeout_log_tail_is_bounded_and_preserves_last_progress() {
        let temp = TempWorkspace::new_named("capture-tail").unwrap();
        let path = temp.0.join("progress.log");
        let mut contents = vec![b'x'; 16 * 1024];
        contents.extend_from_slice(b"\nCLI adjust started\n");
        fs::write(&path, &contents).unwrap();
        let tail = timeout_tail(&path);
        assert!(tail.contains("4096"));
        assert!(tail.ends_with("CLI adjust started\n"));
        assert!(tail.len() < 4200);
        fs::write(&path, []).unwrap();
        assert!(timeout_tail(&path).contains("无已捕获输出"));
    }
    #[test]
    fn inherited_stdin_is_closed() {
        let temp = TempWorkspace::new_named("capture-stdin").unwrap();
        let output = capture(
            &mut command("stdin"),
            Duration::from_secs(3),
            &temp.0,
            "stdin",
        )
        .unwrap();
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("STDIN_CLOSED"));
    }
    #[test]
    fn spawn_failure_cleans_only_own_files() {
        let temp = TempWorkspace::new_named("capture-missing").unwrap();
        fs::write(temp.0.join("keep.txt"), "keep").unwrap();
        let mut cmd = Command::new(temp.0.join("definitely-not-a-program"));
        assert!(capture(&mut cmd, Duration::from_secs(1), &temp.0, "missing").is_err());
        assert_eq!(fs::read_to_string(temp.0.join("keep.txt")).unwrap(), "keep");
        // Command owns the redirection handles on spawn failure; capture must reset them.
        assert_eq!(fs::read_dir(&temp.0).unwrap().count(), 1);
    }
}
