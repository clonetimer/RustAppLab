//! SafeSync：从真实备份/故障恢复理解 Rust 的文件 I/O、Result、完整性校验和提交边界。
pub mod cli;
pub mod demo;
pub mod guide;
pub mod manifest;
pub mod ops;
pub mod sha256;
pub mod tree;

use std::ffi::OsString;
use std::io;

pub fn run(args: &[OsString]) -> io::Result<()> {
    let command =
        cli::parse(args).map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;
    match command {
        cli::Command::Help => print!("{}", cli::HELP),
        cli::Command::Missions => print!("{}", guide::MISSIONS),
        cli::Command::Demo(path) => demo::create(&path)?,
        cli::Command::Backup { source, target } => {
            let (status, summary) = ops::backup(&source, &target)?;
            println!(
                "Backup {status:?}: {} 文件、{} 目录、{} 字节 → {}",
                summary.files,
                summary.directories,
                summary.bytes,
                target.display()
            );
        }
        cli::Command::Stages(path) => {
            let found = ops::stages(&path)?;
            if found.is_empty() {
                println!("无 SafeSync 暂存目录：{}", path.display());
            }
            for info in found {
                match info.status {
                    ops::StageStatus::Ready(summary) => println!(
                        "READY: {} ({} 文件，{} 字节)；可用 recover 提交",
                        info.path.display(),
                        summary.files,
                        summary.bytes
                    ),
                    ops::StageStatus::Incomplete => println!(
                        "INCOMPLETE: {}；未通过校验，不能 recover，请另起备份",
                        info.path.display()
                    ),
                    ops::StageStatus::RestoreStage => println!(
                        "RESTORE: {}；恢复操作暂存不能 recover，请重新执行 restore",
                        info.path.display()
                    ),
                    ops::StageStatus::LegacyReady(summary) => println!(
                        "LEGACY: {} ({} 文件，{} 字节)；0.5.0 无用途标记，人工确认后才可 recover-legacy",
                        info.path.display(),
                        summary.files,
                        summary.bytes
                    ),
                    ops::StageStatus::Unsafe => {
                        println!("UNSAFE: {}；不是普通目录，拒绝使用", info.path.display())
                    }
                }
            }
        }
        cli::Command::Recover { stage, target } => {
            let summary = ops::recover(&stage, &target)?;
            println!(
                "Recover OK: {} 文件、{} 目录、{} 字节 → {}",
                summary.files,
                summary.directories,
                summary.bytes,
                target.display()
            );
        }
        cli::Command::RecoverLegacy { stage, target } => {
            let summary = ops::recover_legacy(&stage, &target)?;
            println!(
                "Recover legacy OK: {} 文件、{} 目录、{} 字节 → {}",
                summary.files,
                summary.directories,
                summary.bytes,
                target.display()
            );
        }
        cli::Command::Verify(path) => {
            let summary = ops::verify(&path)?;
            println!(
                "Verify OK: {} 文件、{} 目录、{} 字节",
                summary.files, summary.directories, summary.bytes
            );
        }
        cli::Command::Restore { backup, target } => {
            let summary = ops::restore(&backup, &target)?;
            println!(
                "Restore OK: {} 文件、{} 目录、{} 字节 → {}",
                summary.files,
                summary.directories,
                summary.bytes,
                target.display()
            );
        }
    }
    Ok(())
}
