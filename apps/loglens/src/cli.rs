//! 不使用第三方命令行库，便于学员读懂 CLI 的完整控制流。
use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Help,
    Demo {
        root: PathBuf,
    },
    Analyze {
        input: PathBuf,
        strict: bool,
    },
    Compare {
        before: PathBuf,
        after: PathBuf,
        strict: bool,
    },
    Report {
        input: PathBuf,
        output: PathBuf,
        strict: bool,
        force: bool,
        top: usize,
    },
    Missions {
        id: Option<String>,
    },
}

pub fn parse(args: &[OsString]) -> Result<Command, String> {
    let Some(first) = args.first() else {
        return Ok(Command::Help);
    };
    let action = first.to_str().ok_or("命令名称应使用 UTF-8")?;
    let rest = &args[1..];
    match action {
        "help" | "-h" | "--help" => {
            if !rest.is_empty() {
                return Err("help 不接受额外参数".into());
            }
            Ok(Command::Help)
        }
        "demo" => {
            if rest.len() > 1 {
                return Err("用法：demo [新目录]".into());
            }
            let root = rest.first().map_or_else(
                || PathBuf::from("loglens-playground"),
                |item| PathBuf::from(item.as_os_str()),
            );
            Ok(Command::Demo { root })
        }
        "analyze" => {
            let (paths, strict) = paths_and_strict(rest)?;
            if paths.len() != 1 {
                return Err("用法：analyze <日志文件> [--strict]".into());
            }
            Ok(Command::Analyze {
                input: paths[0].clone(),
                strict,
            })
        }
        "compare" => {
            let (paths, strict) = paths_and_strict(rest)?;
            if paths.len() != 2 {
                return Err("用法：compare <旧日志> <新日志> [--strict]".into());
            }
            Ok(Command::Compare {
                before: paths[0].clone(),
                after: paths[1].clone(),
                strict,
            })
        }
        "report" => parse_report(rest),
        "missions" => {
            if rest.len() > 1 {
                return Err("用法：missions [08-11]".into());
            }
            let id = rest
                .first()
                .map(|item| item.to_str().ok_or("任务编号必须是 UTF-8"))
                .transpose()?
                .map(str::to_owned);
            Ok(Command::Missions { id })
        }
        _ => Err(format!("未知命令：{action}")),
    }
}

fn paths_and_strict(rest: &[OsString]) -> Result<(Vec<PathBuf>, bool), String> {
    let mut paths = Vec::new();
    let mut strict = false;
    for item in rest {
        match item.to_str() {
            Some("--strict") => strict = true,
            Some(raw) if raw.starts_with('-') => return Err(format!("不支持的选项：{raw}")),
            _ => paths.push(PathBuf::from(item.as_os_str())),
        }
    }
    Ok((paths, strict))
}

fn parse_report(rest: &[OsString]) -> Result<Command, String> {
    let mut input = None;
    let mut output = PathBuf::from("loglens-report.md");
    let mut strict = false;
    let mut force = false;
    let mut top = 10_usize;
    let mut index = 0;
    while index < rest.len() {
        match rest[index].to_str() {
            Some("--strict") => strict = true,
            Some("--force") => force = true,
            Some("-o") | Some("--output") => {
                index += 1;
                let value = rest.get(index).ok_or("--output 后缺少路径")?;
                if value.to_str().is_some_and(|value| value.starts_with('-')) {
                    return Err("--output 后缺少路径".into());
                }
                output = PathBuf::from(value.as_os_str());
            }
            Some("--top") => {
                index += 1;
                let raw = rest
                    .get(index)
                    .and_then(|item| item.to_str())
                    .ok_or("--top 后必须是 1..=100 的整数")?;
                top = raw.parse::<usize>().map_err(|_| "--top 必须为整数")?;
                if !(1..=100).contains(&top) {
                    return Err("--top 必须在 1..=100 之间".into());
                }
            }
            Some(raw) if raw.starts_with('-') => return Err(format!("不支持的选项：{raw}")),
            _ => {
                let path = PathBuf::from(rest[index].as_os_str());
                if input.replace(path).is_some() {
                    return Err("report 只能分析一个日志文件".into());
                }
            }
        }
        index += 1;
    }
    Ok(Command::Report {
        input: input
            .ok_or("用法：report <日志文件> [-o 输出.md] [--top N] [--strict] [--force]")?,
        output,
        strict,
        force,
        top,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(items: &[&str]) -> Vec<OsString> {
        items.iter().map(OsString::from).collect()
    }

    #[test]
    fn report_flags_are_parsed_with_validation() {
        assert_eq!(
            parse(&args(&[
                "report", "a.log", "--top", "3", "-o", "b.md", "--strict"
            ])),
            Ok(Command::Report {
                input: PathBuf::from("a.log"),
                output: PathBuf::from("b.md"),
                strict: true,
                force: false,
                top: 3,
            })
        );
        assert!(parse(&args(&["report", "a.log", "--top", "0"])).is_err());
        assert!(parse(&args(&["report", "a.log", "-o", "--force"])).is_err());
        assert!(parse(&args(&["compare", "one.log"])).is_err());
    }
}
