//! AsyncBoard 学员起步工程：本次只修改 3 处标记代码。
//! 当前不提供 async TCP/TLS，也不假冒 Tokio。教材强调异步与 OS 线程不是同一概念。
pub mod runtime;
use runtime::{Delay, TimerDriver};
#[allow(unused_imports)]
use std::{
    collections::HashSet,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    time::Duration,
};

pub const MAX_JOBS: usize = 128;
pub const MAX_DELAY_MS: u64 = 30_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    pub id: String,
    pub delay_ms: u64,
    pub note: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Completion {
    pub id: String,
    pub delay_ms: u64,
    pub note: String,
}

/// 解析：每行 `ID<TAB>延时毫秒<TAB>说明`。全部校验完成才开始执行。
pub fn parse_jobs(input: &str) -> Result<Vec<Job>, String> {
    // LESSON 1 BEGIN
    // TODO(1): 每行由两个 TAB 分成 ID、毫秒、说明；拒绝重复 ID、非法时间和空清单。
    let _ = input;
    Err("请完成第 1 步：解析任务清单".into())
    // LESSON 1 END
}

/// 学员可以先读懂这段 async fn，逐步实践编写自己的版本。
pub async fn complete(job: Job, delay: Delay) -> Completion {
    // LESSON 3 BEGIN
    // TODO(3): 在生成结果前真正等待 delay；没用 .await 的代码会太早完成。
    let _ = delay;
    Completion { id: job.id, delay_ms: job.delay_ms, note: job.note }
    // LESSON 3 END
}

pub fn run_jobs(jobs: Vec<Job>) -> Result<Vec<Completion>, String> {
    if jobs.is_empty() || jobs.len() > MAX_JOBS || jobs.iter().any(|j| j.delay_ms > MAX_DELAY_MS) {
        return Err("任务数量或延时超出限制，请先 parse_jobs 验证".into());
    }
    let max_delay = jobs.iter().map(|j| j.delay_ms).max().unwrap_or(0);
    let mut driver = TimerDriver::new()?;
    let futures = jobs
        .into_iter()
        .map(|job| {
            let delay = driver.delay(Duration::from_millis(job.delay_ms))?;
            Ok(complete(job, delay))
        })
        .collect::<Result<Vec<_>, String>>()?;
    runtime::drive(futures, Duration::from_millis(max_delay + 3_000))
}

/// 与输出到达次序相同；不同时间的任务应先后完成。
pub fn render_report(completions: &[Completion]) -> String {
    // LESSON 2 BEGIN
    // TODO(2): 用完成顺序生成 TSV：固定表头，再逐条输出 id、wait_ms、note。
    let _ = completions;
    "id\twait_ms\tnote\n".into()
    // LESSON 2 END
}

pub fn run_manifest(input: &str) -> Result<String, String> {
    let jobs = parse_jobs(input)?;
    let result = run_jobs(jobs)?;
    Ok(render_report(&result))
}

/// 不覆盖已存在的报告；写入异常时尽力清理本次新文件。
pub fn write_new(path: &Path, content: &str) -> Result<(), String> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if !parent.is_dir() {
        return Err(format!("输出上级目录不存在：{}", parent.display()));
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("输出文件不会覆盖 {}：{e}", path.display()))?;
    if let Err(e) = file
        .write_all(content.as_bytes())
        .and_then(|()| file.sync_all())
    {
        drop(file);
        let _ = fs::remove_file(path);
        return Err(format!("写入报告失败：{e}"));
    }
    Ok(())
}

pub fn demo(folder: &Path) -> Result<(), String> {
    fs::create_dir(folder).map_err(|e| format!("必须指定全新演示目录：{e}"))?;
    let source = "# 本地异步提醒；第一列 ID，中间毫秒，最后说明\nread\t110\t复习 Future 与 await\nwrite\t40\t为自己的项目添加测试\nreview\t75\t检查输入与错误处理\n";
    write_new(&folder.join("jobs.tsv"), source)?;
    let report = run_manifest(source)?;
    write_new(&folder.join("RESULTS.tsv"), &report)?;
    write_new(&folder.join("LEARNING_NOTE.md"), "# AsyncBoard 演示结果\n\n三项提醒同时开始等待，结果按完成顺序列出。\n\n## 接下来\n\n修改 jobs.tsv 的等待时间，运行 `asyncboard run jobs.tsv my-results.tsv`，再比较结果。\n\n注意：这里是教学执行器和定时器，并不是 Tokio 网络运行时。\n")?;
    Ok(())
}
