//! Provided CLI shell for the advanced ownership lab. State changes remain in the library.
use std::{
    env,
    fs::{self, OpenOptions},
    io::{self, BufRead, Write},
    path::Path,
    sync::Arc,
};
use termgarden::workbench::{spawn_report, Workspace};
use termgarden::{apply_event, parse_command, storage, BoardView, CompactView};

fn new_file(path: &Path, text: &str) -> Result<(), String> {
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    f.write_all(text.as_bytes()).map_err(|e| e.to_string())
}
struct PendingReport {
    dir: std::path::PathBuf,
    snapshot: Arc<termgarden::workbench::Snapshot>,
    board: std::thread::JoinHandle<String>,
    compact: std::thread::JoinHandle<String>,
}
impl PendingReport {
    fn start(ws: &Workspace, dir: &Path) -> Result<Self, String> {
        let snapshot = Arc::new(ws.snapshot().map_err(|e| e.to_string())?);
        fs::create_dir(dir).map_err(|e| format!("请使用全新报告目录：{e}"))?;
        let board = spawn_report(Arc::clone(&snapshot), BoardView);
        let compact = spawn_report(Arc::clone(&snapshot), CompactView);
        println!(
            "报告已排队：revision={}；可以继续编辑，reports查看，wait等待写入。",
            snapshot.revision
        );
        Ok(Self {
            dir: dir.to_owned(),
            snapshot,
            board,
            compact,
        })
    }
    fn ready(&self) -> bool {
        self.board.is_finished() && self.compact.is_finished()
    }
    fn finish(self) -> Result<(), String> {
        // Join both even when one panics; never label a partial export a success.
        let board = self.board.join();
        let compact = self.compact.join();
        new_file(
            &self.dir.join("board.txt"),
            &board.map_err(|_| "board worker panicked")?,
        )?;
        new_file(
            &self.dir.join("summary.txt"),
            &compact.map_err(|_| "compact worker panicked")?,
        )?;
        new_file(
            &self.dir.join("snapshot.tasks"),
            &storage::encode(&self.snapshot.garden)?,
        )?;
        println!(
            "报告已保存：{} / revision={}",
            self.dir.display(),
            self.snapshot.revision
        );
        Ok(())
    }
}
fn reports(ws: &Workspace, dir: &Path) -> Result<(), String> {
    PendingReport::start(ws, dir)?.finish()
}
fn collect(jobs: &mut Vec<PendingReport>, wait: bool) -> Result<(), String> {
    let mut i = 0;
    let mut errors = Vec::new();
    while i < jobs.len() {
        if wait || jobs[i].ready() {
            if let Err(error) = jobs.remove(i).finish() {
                errors.push(error);
            }
        } else {
            i += 1;
        }
    }
    println!("待收取报告：{}", jobs.len());
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}
fn demo(dir: &Path) -> Result<(), String> {
    fs::create_dir(dir).map_err(|e| format!("demo 只创建新目录：{e}"))?;
    let ws = Workspace::new(termgarden::Garden::default()).map_err(|e| e.to_string())?;
    let view = ws.view(Box::new(BoardView));
    for title in ["完成所有权设计", "编写个人回归测试", "交付自己的工作区"] {
        ws.edit(|g| apply_event(g, termgarden::Event::Add(title.into())))
            .map_err(|e| e.to_string())?;
    }
    let before = ws.snapshot().map_err(|e| e.to_string())?;
    let mut numbered = 0;
    let renamed = ws
        .rename_where(
            |t| !t.done,
            |t| {
                numbered += 1;
                format!("{numbered}. {}", t.title)
            },
        )
        .map_err(|e| e.to_string())?;
    let mut journal = format!(
        "renamed={renamed}\n{}",
        view.draw().map_err(|e| e.to_string())?
    );
    ws.undo().map_err(|e| e.to_string())?;
    let restored = ws.snapshot().map_err(|e| e.to_string())?;
    if restored.garden != before.garden {
        return Err("撤销未还原任务".into());
    }
    journal.push_str("undo_restored=true\n");
    let failed = ws.rename_where(|_| true, |_| "\n".into());
    if failed.is_ok() || ws.snapshot().map_err(|e| e.to_string())? != restored {
        return Err("非法批量编辑没有被完整拒绝".into());
    }
    journal.push_str("invalid_batch_unchanged=true\n");
    reports(&ws, &dir.join("reports"))?;
    storage::save(&dir.join("garden.tasks"), &restored.garden)?;
    drop(ws);
    if !matches!(view.draw(), Err(termgarden::workbench::EditError::Closed)) {
        return Err("非拥有视图错误地延长了工作区生命周期".into());
    }
    journal.push_str("view_after_close=Closed\n");
    new_file(&dir.join("SESSION.txt"), &journal)?;
    println!("{journal}\n产物：{}", dir.display());
    Ok(())
}
fn play(path: &Path) -> Result<(), String> {
    let ws = Workspace::new(storage::load(path)?).map_err(|e| e.to_string())?;
    let view = ws.view(Box::new(BoardView));
    let mut jobs = Vec::new();
    println!("add 标题 / n / p / x / f / prefix 前缀 / undo / report 新目录 / reports / wait / save / q\n仅 save 写回任务文件，退出未保存的编辑不会落盘；撤销历史仅保留本次会话最近32次编辑。");
    println!("{}", view.draw().map_err(|e| e.to_string())?);
    for line in io::stdin().lock().lines() {
        let line = line.map_err(|e| e.to_string())?;
        let command = line.trim();
        if command == "q" || command == "quit" {
            break;
        }
        let result: Result<(), String> = if command == "undo" {
            ws.undo().map_err(|e| e.to_string())
        } else if command == "save" {
            storage::save(path, &ws.snapshot().map_err(|e| e.to_string())?.garden)
        } else if let Some(prefix) = command.strip_prefix("prefix ") {
            ws.rename_where(|t| !t.done, |t| format!("{prefix}{}", t.title))
                .map(|_| ())
                .map_err(|e| e.to_string())
        } else if command == "reports" {
            collect(&mut jobs, false)
        } else if command == "wait" {
            collect(&mut jobs, true)
        } else if let Some(dir) = command.strip_prefix("report ") {
            if jobs.len() >= 4 {
                Err("最多4份待收取报告，先执行reports或wait".into())
            } else {
                PendingReport::start(&ws, Path::new(dir)).map(|job| jobs.push(job))
            }
        } else {
            parse_command(command).and_then(|e| {
                ws.edit(|g| apply_event(g, e))
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            })
        };
        if let Err(error) = result {
            eprintln!("未执行：{error}");
        }
        println!("{}", view.draw().map_err(|e| e.to_string())?);
    }
    collect(&mut jobs, true)
}
fn main() {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let result = match args.first().and_then(|s| s.to_str()) {
        Some("demo") if args.len() == 2 => demo(Path::new(&args[1])),
        Some("play") if args.len() == 2 => play(Path::new(&args[1])),
        None | Some("help" | "--help") => {
            println!(
                "TermGarden 进阶工作区\ndemo <新目录>\nplay <任务文件>\n只操作自己的可信练习数据。"
            );
            Ok(())
        }
        _ => Err("用法：demo <新目录> / play <任务文件>".into()),
    };
    if let Err(error) = result {
        eprintln!("工作区错误：{error}");
        std::process::exit(2);
    }
}
