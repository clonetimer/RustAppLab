//! Reusable domain layer: no command-line parsing and no printing.
use std::{collections::HashSet, fs, path::Path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    pub id: u64,
    pub done: bool,
    pub title: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Summary {
    pub total: usize,
    pub open: usize,
    pub done: usize,
}

pub fn validate_title(title: &str) -> Result<String, String> {
    let t = title.trim();
    if t.is_empty() || t.contains(['\t', '\r', '\n']) {
        return Err("标题不能为空或包含制表符/换行".into());
    }
    #[cfg(feature = "strict-titles")]
    if t.chars().count() > 80 {
        return Err("strict-titles: 标题超过80个字符".into());
    }
    Ok(t.to_string())
}

/// Parse task records, reject malformed data, and return tasks ordered by ID.
///
/// ```
/// let tasks = taskcore::parse_tsv("2\tDONE\t写测试\n1\tOPEN\t学 Rust\n")?;
/// assert_eq!(tasks[0].id, 1);
/// assert_eq!(taskcore::summary(&tasks).done, 1);
/// assert!(taskcore::parse_tsv("1\tUNKNOWN\tbad\n").is_err());
/// # Ok::<(), String>(())
/// ```
pub fn parse_tsv(content: &str) -> Result<Vec<Task>, String> {
    let mut tasks = Vec::new();
    let mut ids = HashSet::new();
    for (i, line) in content.lines().enumerate() {
        if line.is_empty() {
            return Err(format!("第{}行为空", i + 1));
        }
        let mut parts = line.split('\t');
        let id: u64 = parts
            .next()
            .unwrap_or("")
            .parse()
            .map_err(|_| format!("第{}行编号非法", i + 1))?;
        let state = parts
            .next()
            .ok_or_else(|| format!("第{}行缺少状态", i + 1))?;
        let title = parts
            .next()
            .ok_or_else(|| format!("第{}行缺少标题", i + 1))?;
        if parts.next().is_some() || id == 0 || !ids.insert(id) {
            return Err(format!("第{}行列数、编号或重复编号错误", i + 1));
        }
        let done = match state {
            "OPEN" => false,
            "DONE" => true,
            _ => return Err(format!("第{}行状态非法", i + 1)),
        };
        let title = validate_title(title)?;
        tasks.push(Task { id, done, title });
    }
    tasks.sort_by_key(|x| x.id);
    Ok(tasks)
}

pub fn to_tsv(tasks: &[Task]) -> String {
    let mut items = tasks.to_vec();
    items.sort_by_key(|x| x.id);
    let mut out = String::new();
    for item in items {
        out.push_str(&format!(
            "{}\t{}\t{}\n",
            item.id,
            if item.done { "DONE" } else { "OPEN" },
            item.title
        ));
    }
    out
}

pub fn summary(tasks: &[Task]) -> Summary {
    let done = tasks.iter().filter(|t| t.done).count();
    Summary {
        total: tasks.len(),
        open: tasks.len() - done,
        done,
    }
}

pub fn add(tasks: &mut Vec<Task>, title: &str) -> Result<u64, String> {
    let title = validate_title(title)?;
    let id = tasks
        .iter()
        .map(|x| x.id)
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .ok_or("任务编号溢出")?;
    tasks.push(Task {
        id,
        done: false,
        title,
    });
    Ok(id)
}

pub fn complete(tasks: &mut [Task], id: u64) -> Result<(), String> {
    if id == 0 {
        return Err("任务编号不能为0".into());
    }
    let task = tasks.iter_mut().find(|t| t.id == id).ok_or("任务不存在")?;
    task.done = true;
    Ok(())
}

pub fn load(path: &Path) -> Result<Vec<Task>, String> {
    match fs::read_to_string(path) {
        Ok(v) => parse_tsv(&v),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(format!("无法读取任务文件：{e}")),
    }
}

/// Basic teaching persistence: validates before writing, NOT crash-safe or cross-process locked.
pub fn save(path: &Path, tasks: &[Task]) -> Result<(), String> {
    let text = to_tsv(tasks);
    // Validate a complete snapshot first; don't persist malformed tasks.
    let checked = parse_tsv(&text)?;
    let mut expected = tasks.to_vec();
    expected.sort_by_key(|x| x.id);
    if checked != expected {
        return Err("任务重复/数据不一致".into());
    }
    if let Ok(m) = fs::symlink_metadata(path) {
        if !m.is_file() || m.file_type().is_symlink() {
            return Err("拒绝覆盖非普通文件或符号链接".into());
        }
    }
    fs::write(path, text).map_err(|e| format!("无法保存任务：{e}"))
}

pub fn markdown(tasks: &[Task]) -> String {
    let sum = summary(tasks);
    let mut report = format!(
        "# 我的任务报告\n\n总数：{}，待办：{}，完成：{}\n\n",
        sum.total, sum.open, sum.done
    );
    for t in tasks {
        let escaped = t
            .title
            .replace('\\', "\\\\")
            .replace('`', "\\`")
            .replace('\n', " ");
        report.push_str(&format!(
            "- [{}] {} · {}\n",
            if t.done { 'x' } else { ' ' },
            t.id,
            escaped
        ));
    }
    report
}
