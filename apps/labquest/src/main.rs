fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if let Err(err) = labquest::run(&args) {
        eprintln!("LabQuest 错误：{err}\n运行 `cargo run -p labquest -- help` 查看帮助。");
        std::process::exit(2);
    }
}
