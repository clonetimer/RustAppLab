use std::{fs::OpenOptions, io::Write};
fn run() -> Result<(), String> {
    let a: Vec<String> = std::env::args().skip(1).collect();
    match a.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        [] | ["help"] | ["--help"] => {
            println!("BadgeForge: make <标题> <副标题> <新文件.svg> | demo")
        }
        ["demo"] => println!(
            "{}",
            badgeforge::document(&badgeforge::Badge {
                title: "我的 Rust 作品".into(),
                subtitle: "编写 · 运行 · 分享".into()
            })
        ),
        ["make", title, subtitle, path] => {
            if title.trim().is_empty()
                || title.chars().count() > 20
                || subtitle.chars().count() > 40
                || title.chars().chain(subtitle.chars()).any(char::is_control)
            {
                return Err("标题限 1–20 个字符，副标题最多 40 个字符，不能含控制字符".into());
            }
            let svg = badgeforge::document(&badgeforge::Badge {
                title: (*title).into(),
                subtitle: (*subtitle).into(),
            });
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
                .and_then(|mut f| f.write_all(svg.as_bytes()))
                .map_err(|e| format!("不能新建 {path}：{e}；不会覆盖已有文件"))?;
            println!("created={path}");
        }
        _ => return Err("用法：make 标题 副标题 新文件.svg".into()),
    }
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("BadgeForge: {e}");
        std::process::exit(2);
    }
}
