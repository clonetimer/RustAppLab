fn main() {
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    if let Err(message) = filequest::run(&arguments) {
        eprintln!("错误：{message}\n\n提示：运行 `cargo run -- help` 查看命令。");
        std::process::exit(2);
    }
}
