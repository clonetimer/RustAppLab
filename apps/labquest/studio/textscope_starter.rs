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
    // TODO(1): 分清字节、字符、行和词元。先读 LESSON.md 对应步骤。
    let _ = tokens(text);
    Stats::default()
    // LESSON 1 END
}
pub fn top_words(text: &str, limit: usize) -> Vec<(String, usize)> {
    // LESSON 2 BEGIN
    // TODO(2): 统计词频并建立确定性排序。先读 LESSON.md 对应步骤。
    let _ = (text, limit);
    // HashMap 是下一步会使用的容器，暂时声明一个空表。
    let _counts: HashMap<String, usize> = HashMap::new();
    Vec::new()
    // LESSON 2 END
}
pub fn search<'a>(text: &'a str, query: &str) -> Vec<(usize, &'a str)> {
    // LESSON 3 BEGIN
    // TODO(3): 返回借用原文的搜索结果。先读 LESSON.md 对应步骤。
    let _ = (text, query);
    Vec::new()
    // LESSON 3 END
}
