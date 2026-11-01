//! B1/B2：请实现这些函数。允许自行拆模块，不要求照抄参考答案。
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
pub fn validate_title(_title: &str) -> Result<String, String> {
    Err("B1 TODO: validate_title".into())
}
pub fn parse_tsv(_data: &str) -> Result<Vec<Task>, String> {
    Err("B1 TODO: parse_tsv".into())
}
pub fn to_tsv(_tasks: &[Task]) -> String {
    String::new()
}
pub fn summary(_tasks: &[Task]) -> Summary {
    Summary {
        total: 0,
        open: 0,
        done: 0,
    }
}
pub fn add(_tasks: &mut Vec<Task>, _title: &str) -> Result<u64, String> {
    Err("B2 TODO: add".into())
}
pub fn complete(_tasks: &mut [Task], _id: u64) -> Result<(), String> {
    Err("B2 TODO: complete".into())
}
pub fn load(_path: &std::path::Path) -> Result<Vec<Task>, String> {
    Err("B3 TODO: load".into())
}
pub fn save(_path: &std::path::Path, _tasks: &[Task]) -> Result<(), String> {
    Err("B3 TODO: save".into())
}
pub fn markdown(_tasks: &[Task]) -> String {
    String::new()
}
