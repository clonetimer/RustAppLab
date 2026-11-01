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
    expected.trim().eq_ignore_ascii_case(actual.trim())
    // LESSON 1 END
}
pub fn parse_deck(text: &str) -> Result<Vec<Card>, String> {
    // LESSON 2 BEGIN
    let mut cards = Vec::new();
    for (index, line) in text.trim_start_matches('\u{feff}').lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 2 || fields.iter().any(|part| part.trim().is_empty()) {
            return Err(format!(
                "第 {} 行必须是 问题<TAB>答案，且都不能为空",
                index + 1
            ));
        }
        cards.push(Card {
            question: fields[0].trim().to_string(),
            answer: fields[1].trim().to_string(),
        });
    }
    if cards.is_empty() {
        return Err("题库没有有效题目".into());
    }
    Ok(cards)
    // LESSON 2 END
}
pub fn grade(cards: &[Card], answers: &[String]) -> Result<Score, String> {
    // LESSON 3 BEGIN
    if cards.len() != answers.len() {
        return Err("答案数量必须等于题目数量".into());
    }
    let missed = cards
        .iter()
        .zip(answers)
        .filter(|(card, answer)| !is_correct(&card.answer, answer))
        .map(|(card, _)| card.question.clone())
        .collect::<Vec<_>>();
    Ok(Score {
        total: cards.len(),
        correct: cards.len() - missed.len(),
        missed,
    })
    // LESSON 3 END
}
