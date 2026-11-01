//! 一条命令制造可观察的演示：准备源文件、创建备份、验证、恢复并再次核验。
use crate::ops;
use std::fs;
use std::io;
use std::path::Path;

pub fn create(root: &Path) -> io::Result<()> {
    if fs::symlink_metadata(root).is_ok() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "演示目录已经存在；请使用新名称，拒绝覆盖",
        ));
    }
    fs::create_dir(root)?;
    let source = root.join("source");
    fs::create_dir(&source)?;
    fs::create_dir_all(source.join("docs/空目录"))?;
    fs::create_dir_all(source.join("logs"))?;
    fs::write(
        source.join("README.txt"),
        "SafeSync 演示\n这是需要备份的文件。\n",
    )?;
    fs::write(
        source.join("docs/任务清单.txt"),
        "1. 备份\n2. 验证\n3. 恢复\n",
    )?;
    fs::write(source.join("logs/app.log"), "INFO start\nWARN slow disk\n")?;
    fs::write(source.join("binary.dat"), [0, 1, 2, 0, 255, 0, 17])?;
    let (status, summary) = ops::backup(&source, &root.join("backup"))?;
    debug_assert_eq!(status, ops::BackupStatus::Created);
    ops::verify(&root.join("backup"))?;
    ops::restore(&root.join("backup"), &root.join("restored"))?;
    println!("SafeSync 演示已完成：{}", root.display());
    println!("源目录：{}", source.display());
    println!("备份目录：{}", root.join("backup").display());
    println!("恢复目录：{}", root.join("restored").display());
    println!(
        "已校验：{} 文件 / {} 子目录 / {} 字节",
        summary.files, summary.directories, summary.bytes
    );
    println!("挑战：修改 backup/data/README.txt，再执行 verify 观察失败（仅在演示目录进行）。");
    Ok(())
}
