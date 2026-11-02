//! B4：为 TaskPocket 提供只读 JSON 快照导出。
//!
//! ```
//! let tasks = vec![taskcore::Task { id: 1, done: false, title: "学 Rust".into() }];
//! let json = taskpocket_json::to_json(&tasks)?;
//! assert_eq!(json, r#"[{"id":1,"done":false,"title":"学 Rust"}]"#);
//! # Ok::<(), String>(())
//! ```

/// 校验并按编号升序导出快照；不修改输入。
pub fn to_json(_tasks: &[taskcore::Task]) -> Result<String, String> {
    // B4 TODO：用本包 DTO 的 Serialize derive 和 serde_json 完成导出。
    // 提示 1：先校验 ID、标题及不能静默 trim 的快照，再排序引用。
    // 提示 2：DTO 只包含 id: u64、done: bool、title: &str。
    // 提示 3：复用 taskcore::validate_title，比较规范化前后的标题；
    // 用 HashSet 检查 ID，用 serde_json::to_string 编码 DTO 数组。
    Err("B4 TODO: to_json".into())
}
