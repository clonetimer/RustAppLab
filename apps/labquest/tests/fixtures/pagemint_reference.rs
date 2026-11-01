//! 教师评测基准实现：验证 PageMint 合同确实存在可通过的正常程序。
//! 正式学习时不要直接查看此文件；先自己实现，再阅读作对照复盘。
use std::env;
use std::ffi::OsString;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

struct Note {
    file: String,
    title: String,
    paragraphs: Vec<String>,
}

fn load(dir: &Path) -> Result<Vec<Note>, String> {
    if !dir.is_dir() { return Err("输入笔记目录不存在".into()); }
    let mut notes = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let file = entry.path();
        if file.extension().and_then(|s| s.to_str()) != Some("note") { continue; }
        let ty = fs::symlink_metadata(&file).map_err(|e| e.to_string())?.file_type();
        if ty.is_dir() { continue; }
        if !ty.is_file() { return Err("拒绝处理符号链接和非普通文件".into()); }
        let name = entry.file_name().into_string().map_err(|_| "笔记文件名必须是 UTF-8".to_string())?;
        let raw = fs::read_to_string(&file).map_err(|e| e.to_string())?;
        let mut lines = raw.lines();
        let title = lines.next().unwrap_or("").trim().to_string();
        if title.is_empty() { return Err(format!("{name} 缺少标题")); }
        let paragraphs = lines.map(str::trim).filter(|line| !line.is_empty()).map(str::to_string).collect();
        notes.push(Note { file: name, title, paragraphs });
    }
    notes.sort_by(|a, b| a.file.cmp(&b.file));
    Ok(notes)
}
fn escape(input: &str) -> String {
    let mut result = String::new();
    for c in input.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&#39;"),
            other => result.push(other),
        }
    }
    result
}
fn build(notes: &[Note]) -> String {
    let mut s = String::from("<!doctype html>\n<html lang=\"zh-CN\">\n<head><meta charset=\"utf-8\"><title>PageMint</title>\n<style>body{font:16px/1.65 system-ui,sans-serif;background:#f4f6fb;color:#162235;margin:0}main{max-width:780px;margin:36px auto;padding:16px}article{background:white;margin:20px 0;padding:22px;border-radius:14px;box-shadow:0 4px 15px #2334}h1,h2{line-height:1.3}p{overflow-wrap:anywhere}</style></head>\n<body><main><h1>PageMint · 我的离线笔记</h1>\n");
    for note in notes {
        s.push_str(&format!("<article data-file=\"{}\"><h2>{}</h2>", escape(&note.file), escape(&note.title)));
        for paragraph in &note.paragraphs { s.push_str(&format!("<p>{}</p>", escape(paragraph))); }
        s.push_str("</article>\n");
    }
    s.push_str("</main></body></html>\n");
    s
}
fn run() -> Result<(), String> {
    let args: Vec<OsString> = env::args_os().skip(1).collect();
    if args.len() == 1 && args[0] == "help" {
        println!("PageMint: stats <notes_dir> | build <notes_dir> <output.html>");
        return Ok(());
    }
    match args.as_slice() {
        [cmd, input] if cmd == "stats" => {
            let notes = load(&PathBuf::from(input))?;
            let count: usize = notes.iter().map(|n| n.paragraphs.len()).sum();
            println!("notes={} paragraphs={count}", notes.len());
        }
        [cmd, input, output] if cmd == "build" => {
            let input = PathBuf::from(input);
            let output = PathBuf::from(output);
            let src = fs::canonicalize(&input).map_err(|e| e.to_string())?;
            let parent = output.parent().ok_or("输出目录不存在")?;
            let dest_parent = fs::canonicalize(parent).map_err(|e| e.to_string())?;
            if dest_parent.starts_with(src) { return Err("输出不得放到输入目录内".into()); }
            let notes = load(&input)?;
            let data = build(&notes);
            let mut handle = OpenOptions::new().write(true).create_new(true).open(&output).map_err(|e| e.to_string())?;
            handle.write_all(data.as_bytes()).map_err(|e| e.to_string())?;
            println!("built={}", notes.len());
        }
        _ => return Err("用法错误".into()),
    }
    Ok(())
}
fn main() {
    if let Err(err) = run() { eprintln!("{err}"); std::process::exit(2); }
}
