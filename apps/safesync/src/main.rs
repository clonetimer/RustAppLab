fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if let Err(err) = safesync::run(&args) {
        eprintln!("SafeSync 错误：{err}");
        std::process::exit(1);
    }
}
