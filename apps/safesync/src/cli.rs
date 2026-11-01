//! CLI 不推测用户的覆盖意图：任何已有路径都不能作为恢复目标。
use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Help,
    Missions,
    Demo(PathBuf),
    Backup { source: PathBuf, target: PathBuf },
    Verify(PathBuf),
    Stages(PathBuf),
    Recover { stage: PathBuf, target: PathBuf },
    RecoverLegacy { stage: PathBuf, target: PathBuf },
    Restore { backup: PathBuf, target: PathBuf },
}

pub const HELP: &str = r#"SafeSync 0.6.0 · 本地快照备份 / SHA-256 校验 / 安全恢复
用法（在 Rust AppLab 项目根目录运行）：
  cargo run -p safesync -- demo [演示目录]
  cargo run -p safesync -- backup <源目录> <全新备份目录>
  cargo run -p safesync -- verify <备份目录>
  cargo run -p safesync -- stages <暂存目录所在父目录>
  cargo run -p safesync -- recover <完整的暂存目录> <原定全新目标目录>
  cargo run -p safesync -- recover-legacy <0.5.0 暂存目录> <原定全新目标目录>
  cargo run -p safesync -- restore <备份目录> <全新恢复目录>
  cargo run -p safesync -- missions
  cargo run -p safesync -- help

特性：保留空目录；SHA-256 逐文件比对；拒绝符号链接、特殊文件及覆盖；
      复制后再次扫描源文件；新暂存区分备份(B)与恢复(R)，仅完整备份可 recover。
      旧版暂存缺少用途标记，recover-legacy 需要先人工核实来源。
范围：离线单用户、普通文件。不是实时同步、加密备份或抗恶意篡改存储。
"#;

pub fn parse(args: &[OsString]) -> Result<Command, String> {
    let words: Vec<&str> = args
        .iter()
        .map(|word| word.to_str().unwrap_or(""))
        .collect();
    let args_path = |index: usize| PathBuf::from(&args[index]);
    match words.as_slice() {
        [] | ["help"] | ["--help"] | ["-h"] => Ok(Command::Help),
        ["missions"] => Ok(Command::Missions),
        ["demo"] => Ok(Command::Demo("safesync-playground".into())),
        ["demo", _] => Ok(Command::Demo(args_path(1))),
        ["backup", _, _] => Ok(Command::Backup {
            source: args_path(1),
            target: args_path(2),
        }),
        ["verify", _] => Ok(Command::Verify(args_path(1))),
        ["stages", _] => Ok(Command::Stages(args_path(1))),
        ["recover", _, _] => Ok(Command::Recover {
            stage: args_path(1),
            target: args_path(2),
        }),
        ["recover-legacy", _, _] => Ok(Command::RecoverLegacy {
            stage: args_path(1),
            target: args_path(2),
        }),
        ["restore", _, _] => Ok(Command::Restore {
            backup: args_path(1),
            target: args_path(2),
        }),
        _ => Err("参数格式错误。运行 safesync help 查看支持命令".to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_commands_without_ambiguous_options() {
        let input = |args: &[&str]| args.iter().map(OsString::from).collect::<Vec<_>>();
        assert_eq!(
            parse(&input(&["backup", "src", "bak"])),
            Ok(Command::Backup {
                source: "src".into(),
                target: "bak".into(),
            })
        );
        assert!(parse(&input(&["restore", "src"])).is_err());
        assert!(parse(&input(&["backup", "src", "bak", "--force"])).is_err());
        assert!(parse(&input(&["unknown"])).is_err());
        assert_eq!(
            parse(&input(&["recover", "old-stage", "new-target"])),
            Ok(Command::Recover {
                stage: "old-stage".into(),
                target: "new-target".into(),
            })
        );
        assert_eq!(
            parse(&input(&["recover-legacy", "old-stage", "new-target"])),
            Ok(Command::RecoverLegacy {
                stage: "old-stage".into(),
                target: "new-target".into(),
            })
        );
    }
}
