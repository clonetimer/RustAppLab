//! Actual existing checker execution. A successful personal run is not a check.
use super::model::{json_string, validate_step};
use super::{book_projects, ApiError, App};
use std::collections::VecDeque;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, Instant};

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

fn commands(
    course_id: &str,
    action: &str,
    step: usize,
    args: &[String],
    root: &Path,
) -> Result<Vec<Command>, ApiError> {
    let mut plan = Vec::new();
    if let Some(book) = book_projects::get(course_id) {
        if book.is_modern() {
            // Explicit rustup override; classic root MSRV is unchanged.
            // Frozen mode avoids implicit registry/network resolution.
            let mut add = |arguments: &[&str], trailing: &[String]| {
                let mut command = Command::new("cargo");
                command
                    .arg("+1.100.0")
                    .args(arguments)
                    .arg("--manifest-path")
                    .arg(root.join("Cargo.toml"))
                    .arg("--target-dir")
                    .arg(root.join("target"));
                crate::process::learner_cargo(&mut command);
                if !trailing.is_empty() {
                    command.arg("--").args(trailing);
                }
                plan.push(command);
            };
            if action == "check" {
                if step == book.steps().len() {
                    for target in ["--all-targets", "--doc"] {
                        add(&["test", "--frozen", target], &[]);
                    }
                } else {
                    let filter = format!("step{step}_");
                    add(&["test", "--frozen", "--test", "contracts", &filter], &[]);
                }
            } else {
                let target = book.run.as_ref().ok_or_else(|| {
                    ApiError::new(400, "本课外部 C/WASM 客户端需按讲义单独运行")
                })?;
                add(
                    &["run", "--frozen", "--release", "--example", target.binary],
                    args,
                );
            }
        } else if action == "check" {
            // --all-targets intentionally excludes rustdoc's examples; run both
            // commands and stop on the first failure instead of hiding it.
            for target in ["--all-targets", "--doc"] {
                let mut command = Command::new("cargo");
                command.args(["test", "--locked", "--offline", "--workspace", target]);
                if course_id == "book-d1" {
                    command.arg("--all-features");
                }
                command
                    .arg("--manifest-path")
                    .arg(root.join("Cargo.toml"))
                    .arg("--target-dir")
                    .arg(root.join("target"));
                crate::process::learner_cargo(&mut command);
                plan.push(command);
            }
        } else {
            let target = book.run.as_ref().ok_or_else(|| {
                ApiError::new(
                    400,
                    "本实验没有适合浏览器启动的有限运行入口；请用“检查实验”运行单测与文档测试",
                )
            })?;
            let mut command = Command::new("cargo");
            command
                .args([
                    "run",
                    "--locked",
                    "--offline",
                    "--package",
                    target.package,
                    "--bin",
                    target.binary,
                    "--manifest-path",
                ])
                .arg(root.join("Cargo.toml"))
                .arg("--target-dir")
                .arg(root.join("target"));
            crate::process::learner_cargo(&mut command);
            command.arg("--").args(args);
            plan.push(command);
        }
    } else if action == "check" {
        let exe = std::env::current_exe()
            .map_err(|e| ApiError::new(500, format!("找不到学习检查程序：{e}")))?;
        let mut command = Command::new(exe);
        command
            .arg("learn")
            .arg("check")
            .arg(root)
            .arg(step.to_string());
        plan.push(command);
    } else {
        let mut command = Command::new("cargo");
        command
            .args(["run", "--offline", "--manifest-path"])
            .arg(root.join("Cargo.toml"))
            .arg("--target-dir")
            .arg(root.join("target"));
        crate::process::learner_cargo(&mut command);
        command.arg("--").args(args);
        plan.push(command);
    }
    for command in &mut plan {
        command.current_dir(root);
    }
    Ok(plan)
}

fn execute(mut plan: Vec<Command>, root: &Path, limit: Duration) -> (String, Option<i32>) {
    let scratch = match crate::capstone::TempWorkspace::new_named("workbench-operation") {
        Ok(scratch) => scratch,
        Err(error) => return (format!("无法创建操作日志：{error}"), None),
    };
    let started = Instant::now();
    let mut text = String::new();
    for command in &mut plan {
        text.push_str(&format!(
            "程序：{:?}\n参数：{:?}\n工程：{}\n\n",
            command.get_program(),
            command.get_args().collect::<Vec<_>>(),
            root.display()
        ));
        let Some(remaining) = limit.checked_sub(started.elapsed()) else {
            text.push_str("本次操作总时限已到，后续命令没有执行。\n");
            return (text, None);
        };
        match crate::process::capture(command, remaining, &scratch.0, "学习工作台操作") {
            Ok(output) => {
                text.push_str(&String::from_utf8_lossy(&output.stdout));
                if !output.stdout.ends_with(b"\n") && !output.stdout.is_empty() {
                    text.push('\n');
                }
                text.push_str(&String::from_utf8_lossy(&output.stderr));
                if !output.status.success() {
                    return (text, output.status.code());
                }
            }
            Err(error) => {
                text.push_str(&format!(
                    "执行失败：{error}\n请检查 Rust/Cargo 与当前课程所需环境，然后重试。"
                ));
                return (text, None);
            }
        }
    }
    (text, Some(0))
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
    let plan = commands(course_id, action, step, &args, &root)?;
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
                if book_projects::get(course_id).is_some_and(|book| book.is_modern()) {
                    "正在用 Rust 1.100.0 检查本步真实合同；最后一步执行全部目标和文档测试…"
                } else if book_projects::get(course_id).is_some() {
                    "正在检查当前 Book 实验副本的单测与文档测试；通过不自动完成学习步骤…"
                } else {
                    "正在检查当前步骤…"
                }
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
    let worker = std::thread::Builder::new()
        .name(format!("learning-operation-{id}"))
        .spawn(move || {
            let (mut text, exit_code) = execute(plan, &root, Duration::from_secs(limit));
            let after = shared
                .workspace
                .lock()
                .ok()
                .and_then(|model| model.source_digest(&course).ok());
            let stale = after.as_deref() != Some(digest.as_str());
            if stale {
                text.push_str(
                    "\n源码在本次操作期间发生变化；这份结果不能验证当前代码，请重新检查。\n",
                );
            }
            if let Ok(mut jobs) = shared.jobs.lock() {
                jobs.finish(id, text, exit_code, stale);
            }
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
    fn book_app(id: &str) -> (crate::capstone::TempWorkspace, Arc<App>) {
        let temp = crate::capstone::TempWorkspace::new_named("book-workbench-job").unwrap();
        let mut workspace = super::super::model::Workspace::new(temp.0.clone()).unwrap();
        workspace.create(id).unwrap();
        let app = Arc::new(App {
            workspace: std::sync::Mutex::new(workspace),
            jobs: std::sync::Mutex::new(Jobs::new()),
            experiments: std::sync::Mutex::new(super::super::experiments::Store::new()),
        });
        (temp, app)
    }

    fn completed(app: &Arc<App>, id: u64) -> Job {
        let deadline = std::time::Instant::now() + Duration::from_secs(180);
        loop {
            let job = get(app, id).unwrap();
            if job.status != "running" {
                return job;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "Book job never completed"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    #[test]
    fn book_job_runs_real_workspace_tests_and_doctests_on_the_selected_copy() {
        let (_temp, app) = book_app("book-d6");
        let job = start(&app, "book-d6", "check", 2, "", None).unwrap();
        let result = completed(&app, job.id);
        assert_eq!(result.exit_code, Some(0), "{}", result.output);
        assert!(result.output.contains("test result: ok"));
        assert!(result.output.contains("Doc-tests"));
        assert!(result.output.contains("--locked"));
        assert!(!result.stale);
        let path = app
            .workspace
            .lock()
            .unwrap()
            .project_path("book-d6")
            .unwrap();
        std::fs::write(
            path.join("appendix-lab/src/lib.rs"),
            "compile_error!(\"learner diagnostic\");",
        )
        .unwrap();
        assert!(get(&app, result.id).unwrap().stale);
        let failed = start(&app, "book-d6", "check", 2, "", None).unwrap();
        let failed = completed(&app, failed.id);
        assert_eq!(failed.status, "failed");
        assert!(failed.output.contains("learner diagnostic"));
        assert!(!failed.output.contains("Doc-tests"));
        assert!(!failed.stale);
    }

    #[test]
    fn every_embedded_book_copy_executes_its_real_contract_without_repository_paths() {
        for book in book_projects::COURSES {
            let (_temp, app) = book_app(book.id);
            let job = start(&app, book.id, "check", 1, "", None).unwrap();
            let done = completed(&app, job.id);
            assert_eq!(done.exit_code, Some(0), "{}: {}", book.id, done.output);
            assert!(!done.stale, "{}: {}", book.id, done.output);
            assert!(
                done.output.contains("Doc-tests"),
                "{}: {}",
                book.id,
                done.output
            );
            assert!(app
                .workspace
                .lock()
                .unwrap()
                .course_json(book.id, 1)
                .unwrap()
                .contains("\"history\":null"));
        }
    }

    #[test]
    fn empty_book_library_cannot_pass_by_removing_its_embedded_unit_tests() {
        let (_temp, app) = book_app("book-d6");
        let root = app
            .workspace
            .lock()
            .unwrap()
            .project_path("book-d6")
            .unwrap();
        let path = root.join("appendix-lab/src/lib.rs");
        let original = std::fs::read(&path).unwrap();
        std::fs::write(&path, "").unwrap();
        let job = start(&app, "book-d6", "check", 2, "", None).unwrap();
        let failed = completed(&app, job.id);
        assert_eq!(failed.status, "failed", "{}", failed.output);
        assert_ne!(failed.exit_code, Some(0));
        std::fs::write(&path, original).unwrap();
        let job = start(&app, "book-d6", "check", 2, "", None).unwrap();
        let repaired = completed(&app, job.id);
        assert_eq!(repaired.exit_code, Some(0), "{}", repaired.output);
        assert!(repaired.output.contains("contracts"));
        assert!(!repaired.stale);
    }

    #[test]
    fn book_without_bounded_binary_refuses_run() {
        for id in ["book-d5", "book-d6"] {
            let (_temp, app) = book_app(id);
            match start(&app, id, "run", 1, "", None) {
                Err(error) => assert_eq!(error.status, 400),
                Ok(_) => panic!("{id} must not start a server or missing binary"),
            }
        }
    }

    #[test]
    fn book_check_covers_all_features_and_never_turns_run_args_into_cargo_flags() {
        let root = Path::new("/learner");
        let plan = commands("book-d1", "check", 2, &[], root).unwrap();
        assert_eq!(plan.len(), 2);
        for command in plan {
            let args: Vec<_> = command
                .get_args()
                .map(|arg| arg.to_string_lossy())
                .collect();
            for required in ["--locked", "--offline", "--workspace", "--all-features"] {
                assert!(args.iter().any(|arg| arg == required));
            }
            assert_eq!(command.get_current_dir(), Some(root));
        }
        let literal = vec!["--manifest-path".into(), "somewhere else".into()];
        let run = commands("book-d1", "run", 1, &literal, root).unwrap();
        let args: Vec<_> = run[0].get_args().map(|arg| arg.to_string_lossy()).collect();
        let separator = args.iter().position(|arg| arg == "--").unwrap();
        assert_eq!(
            &args[separator + 1..],
            &["--manifest-path", "somewhere else"]
        );
    }

    #[test]
    fn book_binary_arguments_are_not_cargo_options() {
        let (_temp, app) = book_app("book-d1");
        let started = start(&app, "book-d1", "run", 1, "workspace", None).unwrap();
        let done = completed(&app, started.id);
        assert_eq!(done.exit_code, Some(0), "{}", done.output);
        assert!(done.output.contains("workspace"));
        assert!(done.output.contains("cargo-applab-map"));
        assert!(!done.output.contains("Doc-tests"));
    }

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
