fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || args == ["help"] || args == ["--help"] {
        println!("TextScope: stats <文件> | top <文件> <1..100> | find <文件> <关键词> | demo");
        return Ok(());
    }
    if args == ["demo"] {
        let s = textscope::stats("Rust rust 学习\n");
        println!(
            "bytes={} chars={} lines={} words={}",
            s.bytes, s.chars, s.lines, s.words
        );
        return Ok(());
    }
    if !(args.len() == 2 && args[0] == "stats"
        || args.len() == 3 && ["top", "find"].contains(&args[0].as_str()))
    {
        return Err("参数不完整，运行 help 查看用法".into());
    }
    let source =
        std::fs::read_to_string(&args[1]).map_err(|e| format!("读取 {} 失败：{e}", args[1]))?;
    let text = source.trim_start_matches('\u{feff}');
    match args[0].as_str() {
        "stats" => {
            let s = textscope::stats(text);
            println!(
                "bytes={} chars={} lines={} words={}",
                s.bytes, s.chars, s.lines, s.words
            );
        }
        "top" => {
            let count = args[2]
                .parse::<usize>()
                .map_err(|_| "数量必须是 1..100".to_string())?;
            if !(1..=100).contains(&count) {
                return Err("数量必须是 1..100".into());
            }
            for (word, n) in textscope::top_words(text, count) {
                println!("{word}\t{n}");
            }
        }
        "find" => {
            if args[2].is_empty() {
                return Err("关键词不能为空".into());
            }
            for (n, line) in textscope::search(text, &args[2]) {
                println!("{n}:{line}");
            }
        }
        _ => unreachable!(),
    }
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("TextScope: {e}");
        std::process::exit(2);
    }
}
