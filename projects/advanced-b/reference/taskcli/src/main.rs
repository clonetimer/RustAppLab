use std::{env, path::Path};
fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() < 2 {
        return Err("用法：taskpocket-pro add|done|list|stats <db> [title|id]".into());
    }
    let cmd = args[0].as_str();
    let path = Path::new(&args[1]);
    let mut tasks = taskcore::load(path)?;
    match cmd {
        "add" if args.len() == 3 => {
            let id = taskcore::add(&mut tasks, &args[2])?;
            taskcore::save(path, &tasks)?;
            println!("added={id}");
        }
        "done" if args.len() == 3 => {
            let id = args[2].parse::<u64>().map_err(|_| "非法编号")?;
            taskcore::complete(&mut tasks, id)?;
            taskcore::save(path, &tasks)?;
            println!("done={id}");
        }
        "list" if args.len() == 2 => {
            print!("{}", taskcore::to_tsv(&tasks));
        }
        "stats" if args.len() == 2 => {
            let s = taskcore::summary(&tasks);
            println!("total={} open={} done={}", s.total, s.open, s.done);
        }
        _ => return Err("未知命令或参数错误".into()),
    };
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(2)
    }
}
