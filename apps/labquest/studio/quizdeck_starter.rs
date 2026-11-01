//! 从自己的 TSV 卡片出题；支持中文、Windows 换行和文件开头的 UTF-8 BOM。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    pub question: String,
    pub answer: String,
}
#[derive(Debug, PartialEq, Eq)]
pub struct Score {
    pub total: usize,
    pub correct: usize,
    pub missed: Vec<String>,
}
pub fn is_correct(expected: &str, actual: &str) -> bool {
    // LESSON 1 BEGIN
    // TODO(1): 比较答案并处理空白。先读 LESSON.md 对应步骤。
    let _ = (expected, actual);
    false
    // LESSON 1 END
}
pub fn parse_deck(text: &str) -> Result<Vec<Card>, String> {
    // LESSON 2 BEGIN
    // TODO(2): 把文本解析为拥有数据的结构体。先读 LESSON.md 对应步骤。
    let _ = text;
    Err("请完成第 2 步：解析题库".into())
    // LESSON 2 END
}
pub fn grade(cards: &[Card], answers: &[String]) -> Result<Score, String> {
    // LESSON 3 BEGIN
    // TODO(3): 用切片、迭代器生成复习清单。先读 LESSON.md 对应步骤。
    let _ = (cards, answers);
    Err("请完成第 3 步：生成成绩".into())
    // LESSON 3 END
}
