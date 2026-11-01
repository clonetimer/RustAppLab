use std::{env, fs, path::PathBuf};

fn help() {
    println!("AsyncBoard · 离线 async/await 提醒工作台 (教学执行器，非 Tokio)\n\n  asyncboard demo <全新目录>\n  asyncboard preview <jobs.tsv>\n  asyncboard run <jobs.tsv> <全新报告.tsv>\n\n任务清单格式：ID<TAB>等待毫秒<TAB>说明；仅在本机计时，不发送网络请求。\nWindows 初学者请先从 Studio 分步练习开始，不必手写 Future/Waker。");
}
fn run() -> Result<(), String> {
    let mut args = env::args_os().skip(1);
    let cmd = args.next().unwrap_or_default();
    match cmd.to_str().unwrap_or("") {
        "help" | "--help" | "-h" if args.next().is_none() => {
            help();
            Ok(())
        }
        "demo" => {
            let root = PathBuf::from(args.next().ok_or("用法：asyncboard demo <新目录>")?);
            if args.next().is_some() {
                return Err("demo 只需要一个全新目录".into());
            }
            asyncboard::demo(&root)?;
            println!("已生成 {}，请打开 RESULTS.tsv", root.display());
            Ok(())
        }
        "preview" => {
            let file = PathBuf::from(args.next().ok_or("用法：asyncboard preview <jobs.tsv>")?);
            if args.next().is_some() {
                return Err("preview 参数过多".into());
            }
            let input = fs::read_to_string(file).map_err(|e| e.to_string())?;
            for job in asyncboard::parse_jobs(&input)? {
                println!("{} · {}ms · {}", job.id, job.delay_ms, job.note);
            }
            Ok(())
        }
        "run" => {
            let source = PathBuf::from(
                args.next()
                    .ok_or("用法：asyncboard run <jobs.tsv> <新报告.tsv>")?,
            );
            let output = PathBuf::from(args.next().ok_or("缺少报告路径")?);
            if args.next().is_some() {
                return Err("run 参数过多".into());
            }
            let input = fs::read_to_string(source).map_err(|e| e.to_string())?;
            let report = asyncboard::run_manifest(&input)?;
            asyncboard::write_new(&output, &report)?;
            println!("已保存 {}", output.display());
            Ok(())
        }
        _ => {
            help();
            Err("未知命令；请使用 help 查看用法".into())
        }
    }
}
fn main() {
    if let Err(e) = run() {
        eprintln!("AsyncBoard 错误：{e}");
        std::process::exit(2);
    }
}
