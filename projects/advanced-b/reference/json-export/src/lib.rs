//! TaskPocket 的只读 JSON 快照导出；基础业务类型不依赖 Serde。
//!
//! ```
//! let tasks = vec![taskcore::Task { id: 1, done: false, title: "学 Rust".into() }];
//! let json = taskpocket_json::to_json(&tasks)?;
//! assert_eq!(json, r#"[{"id":1,"done":false,"title":"学 Rust"}]"#);
//! # Ok::<(), String>(())
//! ```

use std::collections::HashSet;

#[derive(serde::Serialize)]
struct JsonTask<'a> {
    id: u64,
    done: bool,
    title: &'a str,
}

/// 校验并按编号升序导出快照；不修改输入。
///
/// 拒绝零/重复编号、非法标题及需要 trim 的标题。`strict-titles`
/// 启用时，沿用 taskcore 的 80 字符限制。
pub fn to_json(tasks: &[taskcore::Task]) -> Result<String, String> {
    let mut ids = HashSet::new();
    let mut items = Vec::with_capacity(tasks.len());
    for task in tasks {
        if task.id == 0 || !ids.insert(task.id) {
            return Err("任务编号不能为0或重复".into());
        }
        let checked_title = taskcore::validate_title(&task.title)?;
        if checked_title != task.title {
            return Err("快照标题不能包含首尾空白".into());
        }
        items.push(JsonTask {
            id: task.id,
            done: task.done,
            title: &task.title,
        });
    }
    items.sort_by_key(|task| task.id);
    serde_json::to_string(&items).map_err(|error| format!("无法导出 JSON：{error}"))
}
