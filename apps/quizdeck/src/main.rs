use std::{
    fs,
    io::{self, Write},
};
fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || args == ["help"] || args == ["--help"] {
        println!("QuizDeck: play <题库.tsv> | check <题库.tsv> <答案.txt> | demo");
        return Ok(());
    }
    let (cards, answers) = match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["demo"] => (
            quizdeck::parse_deck("Rust 构建工具\tcargo\n2+3\t5\n打印宏\tprintln!\n")?,
            vec!["Cargo".into(), "5".into(), "print".into()],
        ),
        ["check", deck, answer] => {
            let cards =
                quizdeck::parse_deck(&fs::read_to_string(deck).map_err(|e| e.to_string())?)?;
            let data = fs::read_to_string(answer).map_err(|e| e.to_string())?;
            let answers = data
                .trim_start_matches('\u{feff}')
                .lines()
                .map(str::to_string)
                .collect();
            (cards, answers)
        }
        ["play", deck] => {
            let cards =
                quizdeck::parse_deck(&fs::read_to_string(deck).map_err(|e| e.to_string())?)?;
            let mut answers = Vec::new();
            for (i, card) in cards.iter().enumerate() {
                print!("问题 {}/{}：{}\n> ", i + 1, cards.len(), card.question);
                io::stdout().flush().map_err(|e| e.to_string())?;
                let mut answer = String::new();
                if io::stdin()
                    .read_line(&mut answer)
                    .map_err(|e| e.to_string())?
                    == 0
                {
                    return Err("输入已结束；本轮未完成".into());
                }
                answers.push(answer.trim_end_matches(['\r', '\n']).to_string());
            }
            (cards, answers)
        }
        _ => return Err("请使用 play <题库> 或 check <题库> <答案>".into()),
    };
    let score = quizdeck::grade(&cards, &answers)?;
    println!("score={}/{}", score.correct, score.total);
    for question in score.missed {
        println!("复习：{question}");
    }
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("QuizDeck: {e}");
        std::process::exit(2);
    }
}
