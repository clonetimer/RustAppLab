use std::{ffi::OsStr, io::Write, path::PathBuf, process::ExitCode};

fn run() -> Result<(), String> {
    let mut args = std::env::args_os().skip(1);
    let command = args.next();
    let path = args.next();
    if command.as_deref() != Some(OsStr::new("export-json"))
        || path.is_none()
        || args.next().is_some()
    {
        return Err("用法：taskpocket-json export-json <db>".into());
    }
    let path = PathBuf::from(path.ok_or("缺少数据库路径")?);
    let tasks = taskcore::load(&path)?;
    let json = taskpocket_json::to_json(&tasks)?;
    writeln!(std::io::stdout().lock(), "{json}")
        .map_err(|error| format!("无法写入 JSON 输出：{error}"))
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
