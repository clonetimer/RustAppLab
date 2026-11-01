//! Rust AppLab 第三款产品：只读、仅回环地址可访问的本地 HTTP 面板。
pub mod cli;
pub mod data;
pub mod guide;
pub mod http;

use cli::Command;
use data::Snapshot;
use std::ffi::OsString;
use std::fs;
use std::io::Write as _;
use std::net::TcpListener;
use std::path::Path;

pub const HELP: &str = r#"LocalHub 0.3.0 · 本机浏览器工作台

从 RustAppLab 根目录运行：
  cargo run -p localhub -- demo [新目录]
  cargo run -p localhub -- serve --root <目录> [--log <日志文件>] [--port 8765]
  cargo run -p localhub -- missions [12-15]
  cargo run -p localhub -- help

体验演示：
  cargo run -p localhub -- demo
  cargo run -p localhub -- serve --root localhub-playground/files \
    --log localhub-playground/logs/candidate.log
  在浏览器打开 http://127.0.0.1:8765/ ，按 Ctrl+C 停止服务。

说明：只绑定 127.0.0.1；仅允许 GET；启动时制作只读快照。
      --port 0 使用随机空闲端口，--max-requests N 可供自动化测试。
      没有认证及互联网访问功能，请勿使用代理或将本地端口公开到外网。
"#;

pub fn run(args: &[OsString]) -> Result<(), String> {
    match cli::parse(args)? {
        Command::Help => print!("{HELP}"),
        Command::Missions { id } => guide::show(id.as_deref())?,
        Command::Demo { root } => {
            create_demo(&root)
                .map_err(|err| format!("创建演示目录 {} 失败：{err}", root.display()))?;
            println!("演示工作区已创建：{}", root.display());
            println!(
                "启动面板：cargo run -p localhub -- serve --root \"{}\" --log \"{}\"",
                root.join("files").display(),
                root.join("logs/candidate.log").display()
            );
        }
        Command::Serve {
            root,
            log,
            port,
            max_requests,
        } => {
            let snapshot = Snapshot::load(&root, log.as_deref())?;
            // 绑定值固定，只允许本机访问，不提供 --host 选项。
            let listener = TcpListener::bind(("127.0.0.1", port))
                .map_err(|err| format!("无法绑定本机端口 {port}：{err}"))?;
            let assigned_port = listener.local_addr().map_err(|err| err.to_string())?.port();
            println!("LocalHub 已启动：http://127.0.0.1:{assigned_port}/");
            println!(
                "只读快照：{} 个文件；日志分析：{}；按 Ctrl+C 退出。",
                snapshot.files.files.len(),
                if snapshot.logs.is_some() {
                    "启用"
                } else {
                    "未配置"
                }
            );
            std::io::stdout().flush().map_err(|err| err.to_string())?;
            http::serve(listener, &snapshot, max_requests)
                .map_err(|err| format!("本地 HTTP 服务异常：{err}"))?;
        }
    }
    Ok(())
}

pub fn create_demo(root: &Path) -> std::io::Result<()> {
    // 不覆盖任何已有目录；如中途失败，可能留下部分新创建的样例文件。
    fs::create_dir(root)?;
    filequest::demo::create_demo(&root.join("files"))?;
    loglens::demo::create_demo(&root.join("logs"))?;
    Ok(())
}
