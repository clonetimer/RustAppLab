mod http;
use servicedesk::{ensure_store, init, load, persist, render_json, Action};
use std::{
    env, fs,
    net::TcpListener,
    path::{Path, PathBuf},
};
fn demo(path: &Path) -> Result<(), String> {
    fs::create_dir(path).map_err(|e| format!("演示目录需不存在：{e}"))?;
    let journal = path.join("tickets.log");
    init(&journal)?;
    persist(&journal, Action::Add("排查下载失败".into()))?;
    persist(&journal, Action::Add("为网页增加中文检索".into()))?;
    persist(&journal, Action::Close(1))?;
    fs::write(path.join("tickets.json"), render_json(&load(&journal)?))
        .map_err(|e| e.to_string())?;
    fs::write(path.join("README.md"),"# ServiceDesk 实际工单\n\n使用真实日志持久化，并可通过浏览器查看和修改。\n\n启动：`servicedesk serve tickets.log 8766`，打开 http://127.0.0.1:8766/ 。\n").map_err(|e|e.to_string())?;
    println!("ServiceDesk demo: {}，共 2 条工单", path.display());
    Ok(())
}
fn serve(path: &Path, port: u16) -> Result<(), String> {
    if port == 0 {
        return Err("需要非零端口".into());
    }
    ensure_store(path)?;
    let listener = TcpListener::bind(("127.0.0.1", port))
        .map_err(|e| format!("无法绑定本机端口 {port}：{e}"))?;
    println!("ServiceDesk 已启动：http://127.0.0.1:{port}/  （Ctrl+C 退出）\n同步串行教学服务器，不可用于公网部署。");
    for connection in listener.incoming() {
        match connection {
            Ok(mut stream) => {
                if let Err(e) = http::handle(&mut stream, port, path) {
                    eprintln!("HTTP 客户端操作失败：{e}");
                }
            }
            Err(e) => eprintln!("HTTP 接入失败：{e}"),
        }
    }
    Ok(())
}
fn run() -> Result<(), String> {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    let text = |i: usize| {
        args.get(i)
            .and_then(|s| s.to_str())
            .ok_or("参数应为 UTF-8 文本".to_string())
    };
    match args.first().and_then(|s|s.to_str()).unwrap_or("help"){
        "help"|"--help" if args.len()<=1=>{println!("ServiceDesk · 本机工单 Web 服务（同步教学版）\n  demo <新目录>\n  init <新日志文件>\n  serve <日志文件> [端口, 默认 8766]\n  list <日志文件>\n仅监听本机，不开放公网。异步 Axum 版仍待依赖验证。");Ok(())}
        "demo" if args.len()==2=>demo(&PathBuf::from(&args[1])),
        "init" if args.len()==2=>init(&PathBuf::from(&args[1])),
        "list" if args.len()==2=>{println!("{}",render_json(&load(&PathBuf::from(&args[1]))?));Ok(())},
        "serve" if (2..=3).contains(&args.len())=>{let port=if args.len()==3 {text(2)?.parse::<u16>().map_err(|_|"端口必须为整数")?} else {8766};serve(&PathBuf::from(&args[1]),port)},
        _=>Err("用法：servicedesk help | demo <新目录> | init <新日志> | serve <日志> [端口] | list <日志>".into()),
    }
}
fn main() {
    if let Err(e) = run() {
        eprintln!("ServiceDesk 错误：{e}");
        std::process::exit(2)
    }
}
