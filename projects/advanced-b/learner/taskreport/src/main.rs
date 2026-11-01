use std::{env, path::Path};
fn run() -> Result<(), String> {
    let args: Vec<_> = env::args().skip(1).collect();
    if args.len() != 1 {
        return Err("用法：taskpocket-report <db>".into());
    }
    let tasks = taskcore::load(Path::new(&args[0]))?;
    print!("{}", taskcore::markdown(&tasks));
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(2)
    }
}
