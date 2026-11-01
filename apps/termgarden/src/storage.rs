//! Supplied infrastructure. Only task data is persisted; selection/filter are session state.
use crate::{valid_title, Garden, Task};
use std::{
    collections::HashSet,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn decode(text: &str) -> Result<Garden, String> {
    let mut lines = text.lines();
    if lines.next() != Some("TERMGARDEN\t1") {
        return Err("不是 TermGarden v1 文件；不会自动覆盖".into());
    }
    let mut garden = Garden::default();
    let mut ids = HashSet::new();
    for line in lines {
        let parts: Vec<_> = line.split('\t').collect();
        if parts.len() != 3 {
            return Err("任务记录应为 编号<Tab>状态<Tab>标题".into());
        }
        let id: u64 = parts[0].parse().map_err(|_| "任务编号必须是正整数")?;
        if id == 0 || !ids.insert(id) || id == u64::MAX {
            return Err("任务编号重复、为零或已耗尽".into());
        }
        let done = match parts[1] {
            "open" => false,
            "done" => true,
            _ => return Err("状态只能是 open 或 done".into()),
        };
        valid_title(parts[2])?;
        garden.next_id = garden.next_id.max(id + 1);
        garden.tasks.push(Task {
            id,
            title: parts[2].into(),
            done,
        });
        if garden.tasks.len() > 1000 {
            return Err("任务数超过 1000".into());
        }
    }
    garden.selected = garden.tasks.first().map(|t| t.id);
    Ok(garden)
}
pub fn encode(garden: &Garden) -> Result<String, String> {
    let mut out = String::from("TERMGARDEN\t1\n");
    for task in &garden.tasks {
        valid_title(&task.title)?;
        out.push_str(&format!(
            "{}\t{}\t{}\n",
            task.id,
            if task.done { "done" } else { "open" },
            task.title
        ));
    }
    decode(&out)?;
    Ok(out)
}
pub fn load(path: &Path) -> Result<Garden, String> {
    match fs::symlink_metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Garden::default()),
        Err(e) => Err(e.to_string()),
        Ok(m) if !m.is_file() || m.file_type().is_symlink() || m.len() > 1024 * 1024 => {
            Err("任务文件必须是非链接普通文件，且不超过 1 MiB".into())
        }
        Ok(_) => decode(&fs::read_to_string(path).map_err(|e| e.to_string())?),
    }
}
/// Same-directory staging and rename. Single writer only; not a power-loss guarantee.
pub fn save(path: &Path, garden: &Garden) -> Result<(), String> {
    let text = encode(garden)?;
    if let Ok(meta) = fs::symlink_metadata(path) {
        if !meta.is_file() || meta.file_type().is_symlink() {
            return Err("拒绝覆盖非普通文件或符号链接".into());
        }
    }
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let stage = parent.join(format!(".termgarden-{}-{stamp}.tmp", std::process::id()));
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&stage)
        .map_err(|e| e.to_string())?;
    let result = (|| {
        f.write_all(text.as_bytes())?;
        f.sync_all()?;
        drop(f);
        fs::rename(&stage, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(stage);
    }
    result.map_err(|e| e.to_string())
}
