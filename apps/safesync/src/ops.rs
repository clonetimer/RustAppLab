//! SafeSync 的保守恢复原则：只写入全新目标路径；复制后复验再提交。
//! 同盘目录 rename 提供有限的提交原子性；不承诺 fsync 父目录、ACL、并发恶意变更防护。
use crate::manifest::{self, MANIFEST};
use crate::sha256::{hex, Sha256};
use crate::tree::{self, Entry, Kind};
#[cfg(debug_assertions)]
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const PAYLOAD: &str = "data";

/// 使用与 0.5.0 不同的前缀，防止把旧版无类型暂存误当成新版备份暂存。
/// `~` 分隔符与数字结尾一起校验，不从前缀猜测用户原始目标名。
#[derive(Clone, Copy)]
enum StagePurpose {
    Backup,
    Restore,
}

impl StagePurpose {
    fn marker(self) -> &'static str {
        match self {
            Self::Backup => "B",
            Self::Restore => "R",
        }
    }
}

/// 仅在 debug 构建启用，专供课程测试模拟进程在提交边界前突然退出。
/// exit(86) 不运行 Drop；保留暂存目录以观察进程中断，而非模拟真实断电或 fsync 语义。
#[cfg(debug_assertions)]
fn debug_fault(point: &str) {
    if std::env::var_os("RUST_APPLAB_SAFESYNC_FAULT").as_deref() == Some(OsStr::new(point)) {
        eprintln!("SafeSync debug fault injection: {point}; process exits with 86");
        std::process::exit(86);
    }
}

#[cfg(not(debug_assertions))]
fn debug_fault(_: &str) {}

/// 注入普通 I/O 错误以验证 RAII 清理；与 exit(86) 的中断测试区分。
#[cfg(debug_assertions)]
fn debug_io_error(point: &str) -> io::Result<()> {
    if std::env::var_os("RUST_APPLAB_SAFESYNC_FAULT").as_deref() == Some(OsStr::new(point)) {
        return Err(io::Error::other(format!(
            "SafeSync debug injected I/O error: {point}"
        )));
    }
    Ok(())
}

#[cfg(not(debug_assertions))]
fn debug_io_error(_: &str) -> io::Result<()> {
    Ok(())
}

fn refused(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.into())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Summary {
    pub files: usize,
    pub directories: usize,
    pub bytes: u64,
}

impl Summary {
    fn from_entries(entries: &[Entry]) -> Self {
        let mut result = Self {
            files: 0,
            directories: 0,
            bytes: 0,
        };
        for entry in entries {
            match entry.kind {
                Kind::Directory => result.directories += 1,
                Kind::File { bytes, .. } => {
                    result.files += 1;
                    result.bytes = result.bytes.saturating_add(bytes);
                }
            }
        }
        result
    }
}

/// 仅接受已存在的父目录 + 尚不存在的单个目标名，拒绝所有覆盖行为。
fn target_exists(path: &Path) -> io::Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

fn target_parent(target: &Path) -> io::Result<PathBuf> {
    if target.as_os_str().is_empty() || target.file_name().is_none() {
        return Err(refused("目标必须是尚不存在的目录路径"));
    }
    if target_exists(target)? {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("目标已存在，绝不覆盖：{}", target.display()),
        ));
    }
    let parent = target
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    tree::require_real_dir(parent)?;
    fs::canonicalize(parent)
}

fn ensure_not_inside(parent: &Path, protected: &Path) -> io::Result<()> {
    let root = fs::canonicalize(protected)?;
    if parent.starts_with(&root) {
        return Err(refused(
            "目标不能位于源目录/备份内部，否则可能递归复制或污染备份",
        ));
    }
    Ok(())
}

struct Staging {
    path: PathBuf,
    committed: bool,
}

impl Staging {
    fn create(parent: &Path, target: &Path, purpose: StagePurpose) -> io::Result<Self> {
        let name = target
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| refused("目标目录名必须为 UTF-8"))?;
        tree::check_component(name)?;
        for attempt in 0..16u32 {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| refused("系统时钟异常"))?
                .as_nanos();
            let path = parent.join(format!(
                ".sfs~{}~{name}~{}~{now}~{attempt}.part",
                purpose.marker(),
                std::process::id()
            ));
            match fs::create_dir(&path) {
                Ok(()) => {
                    return Ok(Self {
                        path,
                        committed: false,
                    })
                }
                Err(err) if err.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(err) => return Err(err),
            }
        }
        Err(refused("无法创建唯一的临时目录"))
    }

    fn commit(mut self, target: &Path) -> io::Result<()> {
        // 并发操作目标路径仍存在检查后竞态；项目明确只支持单进程操作、非敌对文件系统。
        if target_exists(target)? {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "提交前目标已被其他程序创建",
            ));
        }
        fs::rename(&self.path, target)?;
        self.committed = true;
        Ok(())
    }
}

impl Drop for Staging {
    fn drop(&mut self) {
        if !self.committed {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

fn copy_entries(
    source: &Path,
    target: &Path,
    entries: &[Entry],
    failpoint: &str,
) -> io::Result<()> {
    let mut copied_files = 0;
    for entry in entries {
        let path = tree::relative_path(&entry.path)?;
        let from = source.join(&path);
        let to = target.join(&path);
        let meta = fs::symlink_metadata(&from)?;
        if meta.file_type().is_symlink() {
            return Err(refused("复制时遇到符号链接，源文件可能已被修改"));
        }
        match &entry.kind {
            Kind::Directory => {
                if !meta.is_dir() {
                    return Err(refused("复制时目录类型发生变化"));
                }
                fs::create_dir(&to)?;
            }
            Kind::File { bytes, sha256 } => {
                if !meta.is_file() {
                    return Err(refused("复制时文件类型发生变化"));
                }
                let mut reader = File::open(&from)?;
                if !reader.metadata()?.is_file() {
                    return Err(refused("复制时文件不是普通文件"));
                }
                let mut writer = File::options().write(true).create_new(true).open(&to)?;
                // 在复制过程中直接核对源文件的大小和摘要，避免把扫描后已发生
                // 变化的数据先写满整个暂存目录才发现错误。写入目标后还会独立重验。
                let mut digest = Sha256::new();
                let mut size = 0u64;
                let mut buffer = [0u8; 64 * 1024];
                loop {
                    let n = reader.read(&mut buffer)?;
                    if n == 0 {
                        break;
                    }
                    writer.write_all(&buffer[..n])?;
                    digest.update(&buffer[..n]);
                    size = size.checked_add(n as u64).ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidData, "复制文件长度溢出")
                    })?;
                    if copied_files == 0 && failpoint == "backup_after_first_file" {
                        debug_io_error("backup_mid_file_io_error")?;
                    }
                }
                writer.flush()?;
                writer.sync_all()?;
                if size != *bytes || hex(&digest.finalize()) != *sha256 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("源文件在扫描/复制期间发生变化：{}", entry.path),
                    ));
                }
                copied_files += 1;
                if copied_files == 1 {
                    debug_fault(failpoint);
                    if failpoint == "backup_after_first_file" {
                        debug_io_error("backup_after_first_file_error")?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn compare(expected: &[Entry], actual: &[Entry]) -> io::Result<()> {
    if expected == actual {
        return Ok(());
    }
    let reason = expected
        .iter()
        .zip(actual)
        .find(|(a, b)| *a != *b)
        .map(|(a, b)| format!("首个差异：预期 {}，实际 {}", a.path, b.path))
        .unwrap_or_else(|| {
            format!(
                "数量不符：预期 {} 项，实际 {} 项",
                expected.len(),
                actual.len()
            )
        });
    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        format!("文件清单校验失败：{reason}"),
    ))
}

pub fn verify(backup: &Path) -> io::Result<Summary> {
    tree::require_real_dir(backup)?;
    let mut names = fs::read_dir(backup)?
        .map(|item| item.map(|entry| entry.file_name()))
        .collect::<io::Result<Vec<_>>>()?;
    names.sort();
    let mut expected_names: Vec<std::ffi::OsString> = vec![PAYLOAD.into(), MANIFEST.into()];
    expected_names.sort();
    if names != expected_names {
        return Err(refused(
            "备份根目录只能包含 data/ 和 manifest.sfs，不允许额外文件",
        ));
    }
    let manifest_entries = manifest::load(&backup.join(MANIFEST))?;
    let actual = tree::scan(&backup.join(PAYLOAD))?;
    compare(&manifest_entries, &actual)?;
    Ok(Summary::from_entries(&actual))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackupStatus {
    Created,
    Unchanged,
}

pub fn backup(source: &Path, target: &Path) -> io::Result<(BackupStatus, Summary)> {
    backup_with_hook(source, target, || {})
}

// 测试钩子仅用于单元测试中确定性制造“复制完成后源数据又变化”的竞争；
// 生产 CLI 始终传递空操作，不包含任意可注入的文件修改命令。
fn backup_with_hook<F: FnOnce()>(
    source: &Path,
    target: &Path,
    after_stage_verified: F,
) -> io::Result<(BackupStatus, Summary)> {
    tree::require_real_dir(source)?;
    // 在扫描源目录之前判断目标父目录，以免扫描时将尚未备份的临时文件算入源。
    // 目标父目录不存在时由 target_parent 返回明确错误。
    let parent = target
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if parent.exists() {
        ensure_not_inside(&fs::canonicalize(parent)?, source)?;
    }
    let expected = tree::scan(source)?;
    if target_exists(target)? {
        // 幂等语义：只有备份本身完整且源内容完全一致，才报告已存在。
        let verified = verify(target)?;
        let existing = manifest::load(&target.join(MANIFEST))?;
        compare(&existing, &expected)?;
        return Ok((BackupStatus::Unchanged, verified));
    }
    let parent = target_parent(target)?;
    ensure_not_inside(&parent, source)?;
    let stage = Staging::create(&parent, target, StagePurpose::Backup)?;
    let data = stage.path.join(PAYLOAD);
    fs::create_dir(&data)?;
    copy_entries(source, &data, &expected, "backup_after_first_file")?;
    compare(&expected, &tree::scan(&data)?)?;
    manifest::save(&stage.path.join(MANIFEST), &expected)?;
    verify(&stage.path)?;
    after_stage_verified();
    // 这只是非敌对单进程场景中的保守最终检查：扫描和 rename 并非原子事务，
    // 不声称提供可冻结全目录的原子快照或抵御反复修改并复原的写入者。
    compare(&expected, &tree::scan(source)?)?;
    debug_fault("backup_after_verify");
    stage.commit(target)?;
    Ok((BackupStatus::Created, Summary::from_entries(&expected)))
}

pub fn restore(backup_dir: &Path, target: &Path) -> io::Result<Summary> {
    // 先验证源备份；损坏、缺失或有额外文件则不创建目标。
    let summary = verify(backup_dir)?;
    let parent = target_parent(target)?;
    ensure_not_inside(&parent, backup_dir)?;
    let entries = manifest::load(&backup_dir.join(MANIFEST))?;
    let stage = Staging::create(&parent, target, StagePurpose::Restore)?;
    copy_entries(
        &backup_dir.join(PAYLOAD),
        &stage.path,
        &entries,
        "restore_after_first_file",
    )?;
    compare(&entries, &tree::scan(&stage.path)?)?;
    debug_fault("restore_after_verify");
    stage.commit(target)?;
    Ok(summary)
}

/// 暂存目录是一个操作证据，而不是清理许可。任何命令都不自动删除旧暂存目录。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StageStatus {
    Ready(Summary),
    Incomplete,
    /// 已知是恢复操作的暂存；即使内容恰好是合法备份，也不能将其恢复为备份。
    RestoreStage,
    /// 0.5.0 未记录操作类型的暂存；必须明确指定 recover-legacy 才允许提交。
    LegacyReady(Summary),
    Unsafe,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageInfo {
    pub path: PathBuf,
    pub status: StageStatus,
}

/// `stages` 不能仅凭一个看起来正确的前缀就把目录声明为 READY。
/// 从右向左解析数字字段，以允许目标目录名中使用 `-` 或 `~`。
fn well_formed_stage_name(name: &str, purpose: Option<StagePurpose>) -> bool {
    let (prefix, delimiter) = match purpose {
        Some(StagePurpose::Backup) => (".sfs~B~", '~'),
        Some(StagePurpose::Restore) => (".sfs~R~", '~'),
        None => (".sfs-", '-'),
    };
    let Some(content) = name
        .strip_prefix(prefix)
        .and_then(|rest| rest.strip_suffix(".part"))
    else {
        return false;
    };
    let mut parts = content.rsplitn(4, delimiter);
    let (Some(attempt), Some(nanos), Some(pid), Some(target)) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return false;
    };
    tree::check_component(target).is_ok()
        && [attempt, nanos, pid]
            .into_iter()
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
}

/// 只列举直接子项：不会递归搜索系统或跟随暂存目录的符号链接。
pub fn stages(parent: &Path) -> io::Result<Vec<StageInfo>> {
    tree::require_real_dir(parent)?;
    let mut results = Vec::new();
    for item in fs::read_dir(parent)? {
        let item = item?;
        let name = item.file_name();
        let Some(text) = name.to_str() else { continue };
        if !(text.starts_with(".sfs-") || text.starts_with(".sfs~")) || !text.ends_with(".part") {
            continue;
        }
        let ty = item.file_type()?;
        let status = if ty.is_symlink() || !ty.is_dir() {
            StageStatus::Unsafe
        } else if text.starts_with(".sfs~R~")
            && well_formed_stage_name(text, Some(StagePurpose::Restore))
        {
            StageStatus::RestoreStage
        } else if text.starts_with(".sfs-") && well_formed_stage_name(text, None) {
            match verify(&item.path()) {
                Ok(summary) => StageStatus::LegacyReady(summary),
                Err(_) => StageStatus::Incomplete,
            }
        } else if !well_formed_stage_name(text, Some(StagePurpose::Backup)) {
            StageStatus::Unsafe
        } else {
            match verify(&item.path()) {
                Ok(summary) => StageStatus::Ready(summary),
                Err(_) => StageStatus::Incomplete,
            }
        };
        results.push(StageInfo {
            path: item.path(),
            status,
        });
    }
    results.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(results)
}

fn stage_name_matches_target(stage_name: &str, target_name: &str, legacy: bool) -> bool {
    let prefix = if legacy {
        format!(".sfs-{target_name}-")
    } else {
        format!(".sfs~B~{target_name}~")
    };
    let Some(record) = stage_name
        .strip_prefix(&prefix)
        .and_then(|tail| tail.strip_suffix(".part"))
    else {
        return false;
    };
    let fields = record
        .split(if legacy { '-' } else { '~' })
        .collect::<Vec<_>>();
    fields.len() == 3
        && fields
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
}

/// 恢复**完整的备份暂存目录**：只在同父目录且目标名匹配时进行提交。
/// 复制中途被中断（无 manifest）的暂存目录不能在此处强行视为完整备份。
/// 操作不会删除暂存内容；若校验失败，调用方可重新运行 backup 到新的目标。
pub fn recover(stage: &Path, target: &Path) -> io::Result<Summary> {
    recover_checked(stage, target, false)
}

/// 显式处理 0.5.0 遗留暂存。旧版暂存不带操作类型，因此只能在用户已亲自
/// 确认来源后使用；不允许普通 `recover` 静默地提升未知用途的旧目录。
pub fn recover_legacy(stage: &Path, target: &Path) -> io::Result<Summary> {
    recover_checked(stage, target, true)
}

fn recover_checked(stage: &Path, target: &Path, legacy: bool) -> io::Result<Summary> {
    tree::require_real_dir(stage)?;
    let parent = target_parent(target)?;
    let stage_parent = stage
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if fs::canonicalize(stage_parent)? != parent {
        return Err(refused("恢复暂存目录与目标必须位于同一个真实父目录"));
    }
    let target_name = target
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| refused("目标目录名必须是 UTF-8"))?;
    tree::check_component(target_name)?;
    let stage_name = stage
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| refused("暂存目录名必须是 UTF-8"))?;
    if !stage_name_matches_target(stage_name, target_name, legacy) {
        return Err(refused(
            "暂存目录类型/名称与目标不匹配；旧版暂存需人工确认后使用 recover-legacy",
        ));
    }
    let summary = verify(stage)?;
    // 同步检查再提交，遵守与普通 backup 一致的“非敌对、单进程”前提。
    if target_exists(target)? {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "恢复前目标被创建，拒绝覆盖",
        ));
    }
    fs::rename(stage, target)?;
    Ok(summary)
}

#[cfg(test)]
mod consistency_tests {
    use super::*;

    #[test]
    fn source_changed_after_stage_verified_is_not_committed() {
        let folder = std::env::temp_dir().join(format!(
            "sfs-consistency-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let source = folder.join("source");
        let target = folder.join("snapshot");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("state.txt"), b"before").unwrap();
        let result = backup_with_hook(&source, &target, || {
            fs::write(source.join("state.txt"), b"after!").unwrap();
        });
        assert!(result.is_err(), "最终源文件扫描必须发现后续修改");
        assert!(!target.exists(), "不一致快照绝不能提交");
        assert_eq!(fs::read(source.join("state.txt")).unwrap(), b"after!");
        assert!(stages(&folder).unwrap().is_empty(), "普通错误应清理暂存");
        fs::remove_dir_all(folder).unwrap();
    }
}
