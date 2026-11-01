fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if let Err(err) = localhub::run(&args) {
        eprintln!("错误：{err}\n\n使用 `cargo run -p localhub -- help` 查看指令。");
        std::process::exit(2);
    }
}
