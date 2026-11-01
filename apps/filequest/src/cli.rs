//! 命令行入口：通过 enum 表达不同的业务操作。
use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Help,
    Demo {
        root: PathBuf,
    },
    Scan {
        root: PathBuf,
        all: bool,
    },
    Find {
        root: PathBuf,
        query: String,
        all: bool,
    },
    Report {
        root: PathBuf,
        output: PathBuf,
        all: bool,
        force: bool,
        limit: usize,
    },
    Missions {
        id: Option<String>,
    },
}

/// 刻意使用标准库手写简单解析器：第 01 课会解释为什么这里适合 `match`。
/// 成熟应用可替换为 clap，见后续课程路线。
pub fn parse(args: &[OsString]) -> Result<Command, String> {
    let Some(first) = args.first() else {
        return Ok(Command::Help);
    };
    let Some(action) = first.to_str() else {
        return Err("命令名必须是有效 UTF-8 文本".into());
    };
    let rest = &args[1..];
    match action {
        "help" | "-h" | "--help" => Ok(Command::Help),
        "demo" => {
            if rest.len() > 1 {
                return Err("用法：demo [目录]".into());
            }
            let root = rest.first().map_or_else(
                || PathBuf::from("filequest-playground"),
                |value| PathBuf::from(value.as_os_str()),
            );
            Ok(Command::Demo { root })
        }
        "scan" => {
            let (positionals, all) = parse_simple_path(rest)?;
            if positionals.len() != 1 {
                return Err("用法：scan <目录> [--all]".into());
            }
            Ok(Command::Scan {
                root: positionals[0].clone(),
                all,
            })
        }
        "find" => {
            let (positionals, all) = parse_simple_path(rest)?;
            if positionals.len() != 2 {
                return Err("用法：find <目录> <文件名关键词> [--all]".into());
            }
            let query = positionals[1]
                .to_str()
                .ok_or("关键词必须为有效 UTF-8 文本")?
                .to_owned();
            if query.is_empty() {
                return Err("关键词不能为空".into());
            }
            Ok(Command::Find {
                root: positionals[0].clone(),
                query,
                all,
            })
        }
        "report" => parse_report(rest),
        "missions" => {
            if rest.len() > 1 {
                return Err("用法：missions [00-07]".into());
            }
            let id = match rest.first() {
                Some(value) => Some(
                    value
                        .to_str()
                        .ok_or("任务编号必须为有效 UTF-8 文本")?
                        .to_string(),
                ),
                None => None,
            };
            Ok(Command::Missions { id })
        }
        _ => Err(format!("未知命令：{action}")),
    }
}

fn parse_simple_path(rest: &[OsString]) -> Result<(Vec<PathBuf>, bool), String> {
    let mut all = false;
    let mut paths = Vec::new();
    for item in rest {
        match item.to_str() {
            Some("--all") => all = true,
            Some(value) if value.starts_with('-') => {
                return Err(format!("不支持的选项：{value}"));
            }
            _ => paths.push(PathBuf::from(item.as_os_str())),
        }
    }
    Ok((paths, all))
}

fn parse_report(rest: &[OsString]) -> Result<Command, String> {
    let mut root = None;
    let mut output = PathBuf::from("filequest-report.html");
    let mut all = false;
    let mut force = false;
    let mut limit = 400_usize;
    let mut index = 0;
    while index < rest.len() {
        match rest[index].to_str() {
            Some("--all") => all = true,
            Some("--force") => force = true,
            Some("-o") | Some("--output") => {
                index += 1;
                let value = rest.get(index).ok_or("--output 后必须提供文件路径")?;
                if value.to_str().is_some_and(|text| text.starts_with('-')) {
                    return Err("--output 后必须提供文件路径，不能是另一个选项".into());
                }
                output = PathBuf::from(value.as_os_str());
            }
            Some("--limit") => {
                index += 1;
                let raw = rest
                    .get(index)
                    .and_then(|value| value.to_str())
                    .ok_or("--limit 后必须提供 1..=10000 的整数")?;
                limit = raw.parse::<usize>().map_err(|_| "--limit 必须是整数")?;
                if !(1..=10_000).contains(&limit) {
                    return Err("--limit 的允许范围为 1..=10000".into());
                }
            }
            Some(option) if option.starts_with('-') => {
                return Err(format!("不支持的选项：{option}"));
            }
            _ => {
                if root
                    .replace(PathBuf::from(rest[index].as_os_str()))
                    .is_some()
                {
                    return Err("report 只能指定一个扫描目录".into());
                }
            }
        }
        index += 1;
    }
    Ok(Command::Report {
        root: root.ok_or("用法：report <目录> [-o 输出.html] [--limit N] [--force] [--all]")?,
        output,
        all,
        force,
        limit,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args(items: &[&str]) -> Vec<OsString> {
        items.iter().map(OsString::from).collect()
    }

    #[test]
    fn parses_scan_with_all() {
        assert_eq!(
            parse(&args(&["scan", "project", "--all"])),
            Ok(Command::Scan {
                root: PathBuf::from("project"),
                all: true,
            })
        );
    }

    #[test]
    fn rejects_invalid_report_limit() {
        assert!(parse(&args(&["report", ".", "--limit", "0"])).is_err());
    }

    #[test]
    fn rejects_missing_output() {
        assert!(parse(&args(&["report", ".", "-o"])).is_err());
        assert!(parse(&args(&["report", ".", "-o", "--force"])).is_err());
    }
}
