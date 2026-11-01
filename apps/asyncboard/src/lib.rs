//! AsyncBoard: 可实际交付的离线提醒/任务队列。使用真正的 async/await。
//! 当前不提供 async TCP/TLS，也不假冒 Tokio。教材强调异步与 OS 线程不是同一概念。
pub mod runtime;
use runtime::{Delay, TimerDriver};
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
    let mut result = Vec::new();
    let mut seen = HashSet::new();
    for (line_no, line) in input.lines().enumerate() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let mut fields = line.split('\t');
        let id = fields.next().unwrap_or_default();
        let delay = fields
            .next()
            .ok_or_else(|| format!("第 {} 行需要两个 TAB", line_no + 1))?;
        let note = fields
            .next()
            .ok_or_else(|| format!("第 {} 行需要任务说明", line_no + 1))?;
        if fields.next().is_some() {
            return Err(format!("第 {} 行包含多余的 TAB", line_no + 1));
        }
        if id.is_empty()
            || id.len() > 24
            || !id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
        {
            return Err(format!("第 {} 行任务 ID 无效", line_no + 1));
        }
        if !seen.insert(id.to_string()) {
            return Err(format!("第 {} 行任务 ID 重复：{id}", line_no + 1));
        }
        let delay_ms = delay
            .parse::<u64>()
            .map_err(|_| format!("第 {} 行毫秒数必须为非负整数", line_no + 1))?;
        if delay_ms > MAX_DELAY_MS {
            return Err(format!(
                "第 {} 行延时不能超过 {MAX_DELAY_MS} 毫秒",
                line_no + 1
            ));
        }
        if note.trim().is_empty()
            || note.chars().count() > 160
            || note.chars().any(char::is_control)
        {
            return Err(format!(
                "第 {} 行任务说明为空、太长或包含控制字符",
                line_no + 1
            ));
        }
        result.push(Job {
            id: id.to_string(),
            delay_ms,
            note: note.to_string(),
        });
        if result.len() > MAX_JOBS {
            return Err(format!("一次最多安排 {MAX_JOBS} 项任务"));
        }
    }
    if result.is_empty() {
        return Err("任务清单为空；先添加一行 ID<TAB>毫秒<TAB>说明".into());
    }
    Ok(result)
}

/// 学员可以先读懂这段 async fn，逐步实践编写自己的版本。
pub async fn complete(job: Job, delay: Delay) -> Completion {
    delay.await;
    Completion {
        id: job.id,
        delay_ms: job.delay_ms,
        note: job.note,
    }
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
    let mut text = String::from("id\twait_ms\tnote\n");
    for job in completions {
        text.push_str(&format!("{}\t{}\t{}\n", job.id, job.delay_ms, job.note));
    }
    text
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;
    #[test]
    fn zero_delay_and_unicode() {
        let jobs = parse_jobs("a\t0\t你好 Rust\n").unwrap();
        assert_eq!(run_jobs(jobs).unwrap()[0].note, "你好 Rust");
    }
    #[test]
    fn order_and_overlap() {
        let jobs = parse_jobs("slow\t120\t慢\nfast\t30\t快\nmid\t65\t中\n").unwrap();
        let now = Instant::now();
        let result = run_jobs(jobs).unwrap();
        assert_eq!(
            result.iter().map(|j| j.id.as_str()).collect::<Vec<_>>(),
            ["fast", "mid", "slow"]
        );
        let elapsed = now.elapsed();
        assert!(elapsed >= Duration::from_millis(110));
        assert!(elapsed < Duration::from_secs(2));
    }
    #[test]
    fn reject_invalid_before_execution() {
        for text in [
            "",
            "a\t4\tn\na\t6\tn\n",
            "a\t9999999\tn",
            "a\t-1\tn",
            "evil/a\t0\tx",
            "a\t0\tx\ty",
        ] {
            assert!(parse_jobs(text).is_err(), "{text:?}");
        }
    }
}
