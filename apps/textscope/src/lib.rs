//! 字符数为 Unicode scalar 数，不是显示字形；中文连续字串算一个词元。
use std::collections::HashMap;
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Stats {
    pub bytes: usize,
    pub chars: usize,
    pub lines: usize,
    pub words: usize,
}
fn tokens(text: &str) -> impl Iterator<Item = &str> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|part| !part.is_empty())
}
pub fn stats(text: &str) -> Stats {
    // LESSON 1 BEGIN
    Stats {
        bytes: text.len(),
        chars: text.chars().count(),
        lines: text.lines().count(),
        words: tokens(text).count(),
    }
    // LESSON 1 END
}
pub fn top_words(text: &str, limit: usize) -> Vec<(String, usize)> {
    // LESSON 2 BEGIN
    let mut counts = HashMap::new();
    for word in tokens(text) {
        *counts.entry(word.to_lowercase()).or_insert(0usize) += 1;
    }
    let mut values: Vec<_> = counts.into_iter().collect();
    values.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    values.truncate(limit);
    values
    // LESSON 2 END
}
pub fn search<'a>(text: &'a str, query: &str) -> Vec<(usize, &'a str)> {
    // LESSON 3 BEGIN
    if query.is_empty() {
        return Vec::new();
    }
    text.lines()
        .enumerate()
        .filter(|(_, line)| line.contains(query))
        .map(|(i, line)| (i + 1, line))
        .collect()
    // LESSON 3 END
}
