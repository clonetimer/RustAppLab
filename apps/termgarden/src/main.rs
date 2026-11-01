use std::{
    env, fs,
    io::{self, BufRead, Write},
    path::Path,
};
use termgarden::{apply_event, parse_command, render, renderer, storage};
const HELP: &str = "TermGarden · 终端任务花园\nplay <任务文件> [board|compact]    按行输入，每条命令按 Enter；自动保存\nreplay <任务文件> <命令文件>    整段成功才保存；用于可重复实验\ndemo <新目录>    生成任务文件与会话报告，不覆盖已有目录\nhelp\n命令：add 标题 | n | p | x | f | q\n这是按行交互版，不要求终端原始模式；不是方向键即时 TUI。";
fn replay(path: &Path, commands: &str) -> Result<String, String> {
    let mut garden = storage::load(path)?;
    let mut transcript = render(&garden);
    for line in commands.lines().filter(|l| !l.trim().is_empty()) {
        transcript.push_str(&format!("\n> {line}\n"));
        if !apply_event(&mut garden, parse_command(line)?)? {
            break;
        }
        transcript.push_str(&render(&garden));
    }
    storage::save(path, &garden)?;
    Ok(transcript)
}
fn run() -> Result<(), String> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    match args.first().and_then(|s| s.to_str()).unwrap_or("help") {
        "help" | "--help" if args.len() <= 1 => println!("{HELP}"),
        "demo" if args.len() == 2 => {
            let dir = Path::new(&args[1]);
            fs::create_dir(dir).map_err(|e| format!("demo 只创建新目录：{e}"))?;
            let commands = include_str!("../samples/session.txt");
            fs::write(dir.join("session.txt"), commands).map_err(|e| e.to_string())?;
            let text = replay(&dir.join("garden.tasks"), commands)?;
            fs::write(dir.join("SESSION_REPORT.txt"), &text).map_err(|e| e.to_string())?;
            println!("{text}\n已保存：{}", dir.display());
        }
        "replay" if args.len() == 3 => {
            let commands = fs::read_to_string(&args[2]).map_err(|e| e.to_string())?;
            println!("{}", replay(Path::new(&args[1]), &commands)?);
        }
        "play" if (2..=3).contains(&args.len()) => {
            let view = renderer(args.get(2).and_then(|s| s.to_str()).unwrap_or("board"))?;
            let path = Path::new(&args[1]);
            let mut garden = storage::load(path)?;
            let input = io::stdin();
            let mut lines = input.lock().lines();
            println!("{HELP}\n{}", view.draw(&garden));
            loop {
                print!("> ");
                io::stdout().flush().map_err(|e| e.to_string())?;
                let Some(line) = lines.next() else {
                    println!("\n输入结束，已保存的任务仍保留。");
                    break;
                };
                let line = line.map_err(|e| e.to_string())?;
                let before = garden.clone();
                let result = parse_command(&line).and_then(|event| apply_event(&mut garden, event));
                match result {
                    Ok(false) => {
                        println!("已退出。重新 play 同一文件可继续。 ");
                        break;
                    }
                    Ok(true) => {
                        if garden.tasks != before.tasks {
                            storage::save(path, &garden)?;
                        }
                        println!("{}", view.draw(&garden));
                    }
                    Err(e) => eprintln!("未执行：{e}"),
                }
            }
        }
        _ => return Err(HELP.into()),
    }
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("TermGarden 错误：{e}");
        std::process::exit(2);
    }
}
