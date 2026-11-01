//! CLI 参数输入是应用的第一道边界：本项目显式解析所有支持的选项。
use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Help,
    Demo {
        root: PathBuf,
    },
    Serve {
        root: PathBuf,
        log: Option<PathBuf>,
        port: u16,
        max_requests: Option<usize>,
    },
    Missions {
        id: Option<String>,
    },
}

pub fn parse(args: &[OsString]) -> Result<Command, String> {
    let Some(first) = args.first() else {
        return Ok(Command::Help);
    };
    let action = first.to_str().ok_or("命令名称必须是有效 UTF-8")?;
    match action {
        "help" | "--help" | "-h" if args.len() == 1 => Ok(Command::Help),
        "demo" if args.len() <= 2 => {
            let root = args
                .get(1)
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("localhub-playground"));
            Ok(Command::Demo { root })
        }
        "serve" => parse_serve(&args[1..]),
        "missions" if args.len() <= 2 => Ok(Command::Missions {
            id: args
                .get(1)
                .map(|item| {
                    item.to_str()
                        .ok_or("关卡编号必须是 UTF-8")
                        .map(str::to_owned)
                })
                .transpose()?,
        }),
        _ => Err(format!(
            "未知命令或多余参数：{action}；使用 localhub help 查看用法"
        )),
    }
}

fn parse_serve(args: &[OsString]) -> Result<Command, String> {
    let mut root = None;
    let mut log = None;
    let mut port = None;
    let mut max_requests = None;
    let mut i = 0;
    while i < args.len() {
        let flag = args[i].to_str().ok_or("serve 的选项必须是 UTF-8")?;
        let value = args
            .get(i + 1)
            .ok_or_else(|| format!("选项 {flag} 缺少值"))?;
        match flag {
            "--root" => {
                if root.replace(PathBuf::from(value)).is_some() {
                    return Err("--root 不能重复".into());
                }
            }
            "--log" => {
                if log.replace(PathBuf::from(value)).is_some() {
                    return Err("--log 不能重复".into());
                }
            }
            "--port" => {
                let parsed = value
                    .to_str()
                    .ok_or("端口必须为数字")?
                    .parse::<u16>()
                    .map_err(|_| "--port 必须在 0..=65535 之间".to_string())?;
                if port.replace(parsed).is_some() {
                    return Err("--port 不能重复".into());
                }
            }
            "--max-requests" => {
                let parsed = value
                    .to_str()
                    .ok_or("请求数必须为数字")?
                    .parse::<usize>()
                    .map_err(|_| "--max-requests 必须为正整数".to_string())?;
                if parsed == 0 || parsed > 100_000 {
                    return Err("--max-requests 必须在 1..=100000 之间".into());
                }
                if max_requests.replace(parsed).is_some() {
                    return Err("--max-requests 不能重复".into());
                }
            }
            _ => return Err(format!("不支持的 serve 选项：{flag}")),
        }
        i += 2;
    }
    Ok(Command::Serve {
        root: root.ok_or("serve 必须明确传入 --root <目录>，不会默认开放工作目录")?,
        log,
        port: port.unwrap_or(8765),
        max_requests,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args(parts: &[&str]) -> Vec<OsString> {
        parts.iter().map(OsString::from).collect()
    }

    #[test]
    fn parses_serve_and_rejects_ambiguous_input() {
        assert_eq!(
            parse(&args(&[
                "serve",
                "--root",
                "example",
                "--port",
                "0",
                "--max-requests",
                "2"
            ])),
            Ok(Command::Serve {
                root: PathBuf::from("example"),
                log: None,
                port: 0,
                max_requests: Some(2),
            })
        );
        for options in [
            vec!["serve"],
            vec!["serve", "--port", "8765"],
            vec!["serve", "--root"],
            vec!["serve", "--root", "x", "--port", "65536"],
            vec!["serve", "--root", "x", "--max-requests", "0"],
            vec!["serve", "--root", "x", "--root", "y"],
            vec!["serve", "--root", "x", "--extra", "y"],
        ] {
            assert!(parse(&args(&options)).is_err(), "{options:?}");
        }
    }
}
