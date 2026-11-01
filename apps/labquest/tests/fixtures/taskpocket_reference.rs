//! Maintainer-only golden implementation used to validate the grader itself.
//! SPOILER: Learners should finish TaskPocket before inspecting this fixture.
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::Path;

struct Task {
    id: u64,
    done: bool,
    title: String,
}

fn load(db: &Path) -> Result<Vec<Task>, String> {
    let content = match fs::read_to_string(db) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error.to_string()),
    };
    let mut tasks = Vec::new();
    let mut seen = HashSet::new();
    for line in content.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() != 3 {
            return Err("invalid TSV record".into());
        }
        let id: u64 = parts[0].parse().map_err(|_| "invalid id")?;
        if id == 0 || !seen.insert(id) {
            return Err("duplicate or zero id".into());
        }
        let done = match parts[1] {
            "OPEN" => false,
            "DONE" => true,
            _ => return Err("invalid status".into()),
        };
        if parts[2].trim().is_empty() || parts[2].contains('\r') {
            return Err("invalid title".into());
        }
        tasks.push(Task {
            id,
            done,
            title: parts[2].to_string(),
        });
    }
    tasks.sort_by_key(|task| task.id);
    Ok(tasks)
}

fn save(db: &Path, tasks: &[Task]) -> Result<(), String> {
    let mut content = String::new();
    for task in tasks {
        content.push_str(&format!(
            "{}\t{}\t{}\n",
            task.id,
            if task.done { "DONE" } else { "OPEN" },
            task.title
        ));
    }
    fs::write(db, content).map_err(|e| e.to_string())
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    let (cmd, db) = match args.as_slice() {
        [cmd, db] if cmd == "list" || cmd == "stats" => (cmd.as_str(), db),
        [cmd, db, _] if cmd == "add" || cmd == "done" => (cmd.as_str(), db),
        _ => return Err("usage: taskpocket add/list/done/stats".into()),
    };
    let path = Path::new(db);
    let mut tasks = load(path)?;
    match cmd {
        "list" => {
            for task in tasks {
                println!(
                    "{}\t{}\t{}",
                    task.id,
                    if task.done { "DONE" } else { "OPEN" },
                    task.title
                );
            }
        }
        "stats" => {
            let done = tasks.iter().filter(|t| t.done).count();
            println!("total={} open={} done={done}", tasks.len(), tasks.len() - done);
        }
        "add" => {
            let title = args[2].trim();
            if title.is_empty() || title.contains(['\t', '\r', '\n']) {
                return Err("invalid title".into());
            }
            let id = tasks.iter().map(|task| task.id).max().unwrap_or(0) + 1;
            tasks.push(Task {
                id,
                done: false,
                title: title.to_owned(),
            });
            save(path, &tasks)?;
            println!("added={id}");
        }
        "done" => {
            let id: u64 = args[2].parse().map_err(|_| "invalid id")?;
            if id == 0 {
                return Err("invalid id".into());
            }
            let task = tasks.iter_mut().find(|t| t.id == id).ok_or("missing id")?;
            task.done = true;
            save(path, &tasks)?;
            println!("done={id}");
        }
        _ => unreachable!(),
    }
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(2);
    }
}
