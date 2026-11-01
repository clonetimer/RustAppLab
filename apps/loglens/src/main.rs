fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if let Err(err) = loglens::run(&args) {
        eprintln!("错误：{err}\n提示：运行 `cargo run -p loglens -- help` 查看命令。");
        std::process::exit(2);
    }
}
