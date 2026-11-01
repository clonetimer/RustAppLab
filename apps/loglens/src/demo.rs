//! 演示数据固定、可重复、不会接触用户的真实日志。
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const BASELINE: &str = concat!(
    "2026-10-28T09:00:00Z INFO server started\n",
    "2026-10-28T09:00:01Z INFO accepted request\n",
    "2026-10-28T09:00:02Z INFO accepted request\n",
    "2026-10-28T09:00:03Z WARN slow query\n",
    "2026-10-28T09:00:04Z INFO accepted request\n",
    "2026-10-28T09:00:05Z ERROR timeout reading cache\n",
    "2026-10-28T09:00:06Z INFO retry success\n",
    "2026-10-28T09:00:07Z INFO server stopped\n",
);

const CANDIDATE: &str = concat!(
    "2026-10-28T10:00:00Z INFO server started\n",
    "2026-10-28T10:00:01Z WARN slow query\n",
    "2026-10-28T10:00:02Z ERROR timeout reading cache\n",
    "2026-10-28T10:00:03Z ERROR timeout reading cache\n",
    "2026-10-28T10:00:04Z INFO retry success\n",
    "2026-10-28T10:00:05Z ERROR database unavailable\n",
    "2026-10-28T10:00:06Z WARN slow query\n",
    "2026-10-28T10:00:07Z INFO server stopped\n",
    "stack trace: line 42\n",
);

pub fn create_demo(root: &Path) -> io::Result<(PathBuf, PathBuf)> {
    // create_dir 拒绝已存在的路径，以免覆盖用户自己整理的日志。
    fs::create_dir(root)?;
    let baseline = root.join("baseline.log");
    let candidate = root.join("candidate.log");
    fs::write(&baseline, BASELINE)?;
    fs::write(&candidate, CANDIDATE)?;
    Ok((baseline, candidate))
}
