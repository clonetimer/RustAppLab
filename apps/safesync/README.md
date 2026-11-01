# SafeSync：本地可验证快照备份

这是 Rust AppLab 0.6.0 的独立教学软件，不是 FileQuest 的子命令，也不是高可用备份服务。

## 15 分钟操作

在 Workspace 根目录：

```sh
cargo run -p safesync -- demo
cargo run -p safesync -- verify safesync-playground/backup
cargo run -p safesync -- restore safesync-playground/backup safesync-playground/another-restore
cargo run -p safesync -- missions
```

演示目录 `safesync-playground/` 是全新创建的专用练习数据，包含 `source/`、`backup/`、`restored/`。若它已存在，改用 `cargo run -p safesync -- demo my-new-demo`，不要删除已有工作目录。

自己的练习文件可使用：

```sh
cargo run -p safesync -- backup ./example-project ./my-snapshot-v1
cargo run -p safesync -- verify ./my-snapshot-v1
cargo run -p safesync -- restore ./my-snapshot-v1 ./example-project-recovery
```

**恢复目标必须是不存在的目录**，而且其父目录已经存在。已有文件、符号链接和目录均不会被覆盖。`backup` 对已经存在且完整、与源逐字节一致的快照返回 `Unchanged`；当源发生变化时应另选新的快照目录名。

## 备份结构与规则

```text
my-snapshot-v1/
├── manifest.sfs      # SAFESYNC 1 格式的完整性清单：相对路径、类型、文件长度、SHA-256
└── data/
    └── ...           # 备份的普通文件和空目录
```

`verify` 完整扫描 `data/`，拒绝多余/缺失文件、非法路径、损坏内容和符号链接。`restore` 先验证备份，复制到临时目录后再次比较，再以同父目录 `rename` 提交。不支持 `--force`，也不会自行删除旧快照。

## 0.5.0 新增：可审查的暂存目录与受控提交

当备份进程在临时目录写入后突然退出，可先在 **暂存目录的父目录** 运行：

```sh
cargo run -p safesync -- stages ./my-backups
```

新版本会将 `.sfs~B~<目标名>~<PID>~<时间>~<序号>.part` 等暂存标记为：`READY`（完整且已校验的备份）、`INCOMPLETE`（数据损坏或不完整）、`RESTORE`（恢复操作暂存）、`LEGACY`（旧版无类型标记的完整暂存）、`UNSAFE`（非目录、符号链接或非法暂存名称）。`stages` **不删除也不修改任何暂存内容**。

如果看到 `READY` 且确认原本要提交的目标仍不存在，可将列表中显示的**完整目录路径**代入：

```sh
cargo run -p safesync -- recover <完整暂存路径> <最初计划的新快照路径>
cargo run -p safesync -- verify <最初计划的新快照路径>
```

`recover` 要求真实父目录相同、目标名字严格对应、暂存通过 `verify`，并且目标不存在。复制中途退出、恢复过程退出或目录被损坏时都不能强制提交，需要在没有故障注入的情况下**重新执行 backup / restore**，可能留下供检查的旧 `.part`。

教程 [第 20 关](../../course/20_interrupted_backup.md) 使用仅限 debug 的测试环境变量触发进程 `exit(86)`，让学员实际复现两类故障。**它不等同于模拟断电**。依然不提供自动清理、断点续传或针对恶意并发文件操作的保护。

## 明确不支持的情形

- 只能备份普通文件和目录；拒绝符号链接、FIFO/socket、非 UTF-8 文件名、若干不适合跨平台移植的路径名称。
- **不保存时间戳、ACL、权限、扩展属性、硬链接关系、稀疏文件信息**；也没有文件锁。目标是还原文件字节与目录结构。
- **不能保证源目录在复制过程中静止**；写入者可能与扫描竞争，尽管复制后的 SHA-256 复验可以发现大部分内容不一致。面对恶意的文件系统并发修改，当前路径检查并非完整沙箱防护。
- SHA-256 检测非恶意损坏，不提供加密、签名、远端存储或抵御攻击者同时更改清单与数据的能力。
- 受最多 100000 目录项、64 层嵌套、2048 字节相对路径、8 MiB 清单上限约束；全量读取文件两次以上，适合小中型离线目录，不承诺大型数据集性能。
- 进程异常崩溃/断电可能残留 `.sfs~*.part` 或旧版 `.sfs-*.part` 临时目录；可用 `stages` 识别完整/不完整，**仅在确认没有正在运行的备份且已辨认目录所有者后手动清理**。恢复已完成的暂存也不提供父目录 fsync/断电保证，不支持不可信多进程同时写同一备份路径。
- 仓库测试已在 Linux x86_64 Rust 1.99.1 上运行；Windows/macOS 和 Rust 1.74 最低版本仍需独立复验。另有 32 MiB 实际基准及外部 SHA-256 校验，性能数值只对实测环境有效。

## 教学地图

- [第 16 关：从文件丢失开始](../../course/16_backup_first_snapshot.md)
- [第 17 关：检测损坏文件](../../course/17_integrity_hash.md)
- [第 18 关：安全恢复](../../course/18_safe_restore.md)
- [第 19 关：自动化与交付](../../course/19_release_recovery.md)
- [第 20 关：异常退出与受控恢复](../../course/20_interrupted_backup.md)
- [第 21 关：真实性能与发布证据](../../course/21_benchmark_release.md)
- [挑战提示](../../course/HINTS_16_19.md)

## 0.6.0 更新：源文件一致性和暂存用途

- `backup` 在复制每个文件时核对实际读取的长度和 SHA-256，并在提交正式目录前重新扫描源树。若有可观察的变化，返回错误并清理正常失败下的暂存；这不是文件系统原子快照，不能保证捕捉扫描后再次变化的源文件。
- 新暂存前缀为 `.sfs~B~<目标名>~...part`（备份）或 `.sfs~R~<目标名>~...part`（恢复），`stages` 区分 `READY`、`INCOMPLETE`、`RESTORE`、`LEGACY`、`UNSAFE`。**只有完整且类型为 B 的新暂存可以普通 `recover`**。
- 0.5.0 未区分用途的 `.sfs-...part` 即使校验正确也只显示 `LEGACY`，需人工核实确由旧版 `backup` 留下，再显式运行 `recover-legacy <旧暂存> <原定全新目标>`。该命令不会提供旧版清单没有的身份真实性保证。
- 教学实验见 [第 22 关：源文件变化](../../course/22_source_consistency_lab.md) 与 [第 23 关：暂存身份](../../course/23_stage_provenance.md)。
