//! Actual existing checker execution. A successful personal run is not a check.
use super::model::{json_string, validate_step};
use super::{ApiError, App};
use std::collections::VecDeque;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub(super) struct Job {
    pub id: u64,
    course_id: String,
    action: String,
    step: usize,
    status: &'static str,
    output: String,
    exit_code: Option<i32>,
    source_digest: String,
    stale: bool,
}

impl Job {
    pub fn json(&self) -> String {
        format!("{{\"id\":{},\"course_id\":{},\"action\":{},\"step\":{},\"status\":{},\"output\":{},\"exit_code\":{},\"source_digest\":{},\"stale\":{}}}",self.id,json_string(&self.course_id),json_string(&self.action),self.step,json_string(self.status),json_string(&self.output),self.exit_code.map(|x|x.to_string()).unwrap_or_else(||"null".into()),json_string(&self.source_digest),self.stale)
    }
}

pub(super) struct Jobs {
    next: u64,
    entries: VecDeque<Job>,
}
impl Jobs {
    pub fn new() -> Self {
        Self {
            next: 1,
            entries: VecDeque::new(),
        }
    }
    pub fn ensure_idle(&self) -> Result<(), ApiError> {
        if self.entries.iter().any(|job| job.status == "running") {
            return Err(ApiError::new(
                409,
                "当前操作正在执行，请等待结果后再修改或运行",
            ));
        }
        Ok(())
    }
    pub fn get(&self, id: u64) -> Result<Job, ApiError> {
        self.entries
            .iter()
            .find(|job| job.id == id)
            .cloned()
            .ok_or_else(|| ApiError::new(404, "没有找到这次操作；请重新检查当前源码"))
    }
    fn finish(&mut self, id: u64, output: String, exit_code: Option<i32>, stale: bool) {
        if let Some(job) = self.entries.iter_mut().find(|job| job.id == id) {
            job.status = if exit_code == Some(0) {
                "succeeded"
            } else {
                "failed"
            };
            job.output = output;
            job.exit_code = exit_code;
            job.stale = stale;
        }
    }
}

fn arguments(text: &str) -> Result<Vec<String>, ApiError> {
    if text.len() > 16 * 1024 || text.contains('\0') {
        return Err(ApiError::new(400, "运行参数过长或包含空字符"));
    }
    let args: Vec<_> = text
        .split('\n')
        .map(|line| line.trim_end_matches('\r'))
        .filter(|line| !line.is_empty())
        .map(String::from)
        .collect();
    if args.len() > 32 || args.iter().any(|x| x.len() > 4096) {
        return Err(ApiError::new(
            400,
            "最多输入 32 个参数，每个参数不超过 4096 字节",
        ));
    }
    Ok(args)
}

pub(super) fn start(
    app: &Arc<App>,
    course_id: &str,
    action: &str,
    step: usize,
    args: &str,
    expected_project: Option<&str>,
) -> Result<Job, ApiError> {
    validate_step(course_id, step)?;
    if !matches!(action, "check" | "run") {
        return Err(ApiError::new(400, "只能检查当前步骤或运行自己的程序"));
    }
    let args = arguments(args)?;
    if action == "check" && !args.is_empty() {
        return Err(ApiError::new(400, "正式步骤检查不接受自定义运行参数"));
    }
    let mut jobs = app
        .jobs
        .lock()
        .map_err(|_| ApiError::new(500, "操作状态不可用，请重新启动工作台"))?;
    jobs.ensure_idle()?;
    let (root, digest) = {
        let model = app
            .workspace
            .lock()
            .map_err(|_| ApiError::new(500, "工作区状态不可用"))?;
        let root = model.project_path(course_id)?;
        if expected_project.is_some_and(|expected| root.to_string_lossy() != expected) {
            return Err(ApiError::new(
                409,
                "此课程已在另一个页面打开其他工程；请重新选择工程后运行",
            ));
        }
        (root, model.source_digest(course_id)?)
    };
    let mut command = if action == "check" {
        let exe = std::env::current_exe()
            .map_err(|e| ApiError::new(500, format!("找不到学习检查程序：{e}")))?;
        let mut command = Command::new(exe);
        command
            .arg("learn")
            .arg("check")
            .arg(&root)
            .arg(step.to_string());
        command
    } else {
        let mut command = Command::new("cargo");
        command
            .args(["run", "--offline", "--manifest-path"])
            .arg(root.join("Cargo.toml"))
            .arg("--target-dir")
            .arg(root.join("target"));
        crate::process::learner_cargo(&mut command);
        command.arg("--").args(&args);
        command
    };
    command.current_dir(&root);
    let printed = format!(
        "程序：{:?}\n参数：{:?}\n工程：{}\n\n",
        command.get_program(),
        command.get_args().collect::<Vec<_>>(),
        root.display()
    );
    let id = jobs.next;
    jobs.next += 1;
    let job = Job {
        id,
        course_id: course_id.into(),
        action: action.into(),
        step,
        status: "running",
        output: format!(
            "{}\n首次编译可能需要稍等，完成后在这里显示真实输出。",
            if action == "check" {
                "正在检查当前步骤…"
            } else {
                "正在运行自己的程序…"
            }
        ),
        exit_code: None,
        source_digest: digest.clone(),
        stale: false,
    };
    jobs.entries.push_back(job.clone());
    while jobs.entries.len() > 32 {
        jobs.entries.pop_front();
    }
    let shared = Arc::clone(app);
    let course = course_id.to_string();
    let limit = if action == "check" { 180 } else { 120 };
    let worker=std::thread::Builder::new().name(format!("learning-operation-{id}")).spawn(move|| {
        let outcome=(|| {
            let scratch=crate::capstone::TempWorkspace::new_named("workbench-operation")?;
            crate::process::capture(&mut command,Duration::from_secs(limit),&scratch.0,"学习工作台操作")
        })();
        let (mut text,exit_code)=match outcome {
            Ok(output)=>(format!("{printed}{}{}{}",String::from_utf8_lossy(&output.stdout),if output.stdout.ends_with(b"\n") || output.stdout.is_empty() {""} else {"\n"},String::from_utf8_lossy(&output.stderr)),output.status.code()),
            Err(error)=>(format!("{printed}执行失败：{error}\n请检查 Rust/Cargo 与当前课程所需环境，然后重试。"),None),
        };
        let after=shared.workspace.lock().ok().and_then(|model|model.source_digest(&course).ok());
        let stale=after.as_deref()!=Some(digest.as_str());
        if stale { text.push_str("\n源码在本次操作期间发生变化；这份结果不能验证当前代码，请重新检查。\n"); }
        if let Ok(mut jobs)=shared.jobs.lock() { jobs.finish(id,text,exit_code,stale); }
    });
    if let Err(error) = worker {
        jobs.finish(id, format!("无法启动操作：{error}。请重试。"), None, false);
        return jobs.get(id);
    }
    Ok(job)
}

/// Recheck freshness even after a completed operation, including external edits.
pub(super) fn get(app: &App, id: u64) -> Result<Job, ApiError> {
    let mut job = app
        .jobs
        .lock()
        .map_err(|_| ApiError::new(500, "操作状态不可用"))?
        .get(id)?;
    let current = app
        .workspace
        .lock()
        .map_err(|_| ApiError::new(500, "工作区状态不可用"))?
        .source_digest(&job.course_id);
    job.stale |= current.ok().as_deref() != Some(job.source_digest.as_str());
    Ok(job)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn argument_lines_remain_literal_without_shell_parsing() {
        let args = arguments("林 小\nA&B <test>\n$(touch sentinel)\n\"quoted\"\r\n").unwrap();
        assert_eq!(
            args,
            vec!["林 小", "A&B <test>", "$(touch sentinel)", "\"quoted\""]
        );
    }
    #[test]
    fn argument_count_and_size_are_bounded() {
        assert!(arguments(&vec!["x"; 33].join("\n")).is_err());
        assert!(arguments(&"x".repeat(4097)).is_err());
        assert!(arguments("bad\0value").is_err());
    }
    #[test]
    fn failed_operation_releases_busy_state() {
        let mut jobs = Jobs::new();
        jobs.entries.push_back(Job {
            id: 1,
            course_id: "firstcard".into(),
            action: "run".into(),
            step: 1,
            status: "running",
            output: String::new(),
            exit_code: None,
            source_digest: "x".into(),
            stale: false,
        });
        assert_eq!(jobs.ensure_idle().unwrap_err().status, 409);
        jobs.finish(1, "real launch error".into(), None, false);
        assert!(jobs.ensure_idle().is_ok());
        let job = jobs.get(1).unwrap();
        assert!(job.json().contains("\"action\":\"run\""));
        assert_eq!(job.status, "failed");
    }
}
