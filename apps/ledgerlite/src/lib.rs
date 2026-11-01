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
    let parts: Vec<_> = text.split('.').collect();
    if parts.is_empty()
        || parts.len() > 2
        || parts[0].is_empty()
        || !parts[0].bytes().all(|c| c.is_ascii_digit())
    {
        return Err("金额必须是非负十进制数，例如 12.50".into());
    }
    let whole = parts[0]
        .parse::<u64>()
        .map_err(|_| "金额太大".to_string())?;
    let fraction = if parts.len() == 2 {
        let digits = parts[1];
        if !(1..=2).contains(&digits.len()) || !digits.bytes().all(|c| c.is_ascii_digit()) {
            return Err("小数只支持一位或两位".into());
        }
        let value = digits.parse::<u64>().map_err(|e| e.to_string())?;
        if digits.len() == 1 {
            value * 10
        } else {
            value
        }
    } else {
        0
    };
    whole
        .checked_mul(100)
        .and_then(|v| v.checked_add(fraction))
        .ok_or_else(|| "金额太大".into())
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
    let mut entries = Vec::new();
    for (i, line) in text.trim_start_matches('\u{feff}').lines().enumerate() {
        let parts: Vec<_> = line.split('\t').collect();
        if parts.len() != 3
            || !valid_day(parts[0])
            || parts[1].trim().is_empty()
            || parts[1].chars().any(char::is_control)
        {
            return Err(format!("第 {} 行：需要有效日期<TAB>分类<TAB>金额", i + 1));
        }
        entries.push(Entry {
            day: parts[0].into(),
            category: parts[1].trim().into(),
            cents: parse_cents(parts[2]).map_err(|e| format!("第 {} 行：{e}", i + 1))?,
        });
    }
    Ok(entries)
    // LESSON 2 END
}
pub fn totals(entries: &[Entry]) -> Result<BTreeMap<String, u64>, String> {
    // LESSON 3 BEGIN
    let mut sums = BTreeMap::new();
    for entry in entries {
        let current = sums.entry(entry.category.clone()).or_insert(0u64);
        *current = current
            .checked_add(entry.cents)
            .ok_or_else(|| "分类总额溢出".to_string())?;
    }
    Ok(sums)
    // LESSON 3 END
}
pub fn money(cents: u64) -> String {
    format!("{}.{:02}", cents / 100, cents % 100)
}
