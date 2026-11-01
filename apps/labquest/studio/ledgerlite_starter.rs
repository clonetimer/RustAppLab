//! 学习定点数、数据校验和分组聚合；不是并发数据库或财务软件。
use std::collections::BTreeMap;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub day: String,
    pub category: String,
    pub cents: u64,
}
pub fn parse_cents(text: &str) -> Result<u64, String> {
    // LESSON 1 BEGIN
    // TODO(1): 用整数分精确表示金额。先读 LESSON.md 对应步骤。
    let _ = text;
    Err("请完成第 1 步：金额转分".into())
    // LESSON 1 END
}
/// 日期校验基础设施已提供，学习主线先聚焦金额、记录、聚合。
fn valid_day(day: &str) -> bool {
    let b = day.as_bytes();
    if b.len() != 10
        || b[4] != b'-'
        || b[7] != b'-'
        || b.iter()
            .enumerate()
            .any(|(i, c)| i != 4 && i != 7 && !c.is_ascii_digit())
    {
        return false;
    }
    let y = day[..4].parse::<u32>().unwrap_or(0);
    let m = day[5..7].parse::<u32>().unwrap_or(0);
    let d = day[8..].parse::<u32>().unwrap_or(0);
    let last = match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if y % 400 == 0 || (y % 4 == 0 && y % 100 != 0) => 29,
        2 => 28,
        _ => 0,
    };
    y > 0 && (1..=last).contains(&d)
}
pub fn parse_entries(text: &str) -> Result<Vec<Entry>, String> {
    // LESSON 2 BEGIN
    // TODO(2): 解析并验证可持久化记录。先读 LESSON.md 对应步骤。
    let _ = (text, valid_day("2026-10-28"));
    Err("请完成第 2 步：记录解析".into())
    // LESSON 2 END
}
pub fn totals(entries: &[Entry]) -> Result<BTreeMap<String, u64>, String> {
    // LESSON 3 BEGIN
    // TODO(3): 借用记录并按分类累计。先读 LESSON.md 对应步骤。
    let _ = entries;
    Err("请完成第 3 步：分类汇总".into())
    // LESSON 3 END
}
pub fn money(cents: u64) -> String {
    format!("{}.{:02}", cents / 100, cents % 100)
}
