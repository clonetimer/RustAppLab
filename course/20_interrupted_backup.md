# 第 20 关 · 从进程中断恢复一份已完整写入的备份

**本关中心问题**：备份写完并校验成功，进程却在最后提交前退出。磁盘上的 `.part` 应当被忽略、删除，还是可以复用？决定权不能只依赖文件名，必须用内容和操作边界做判断。

本关使用 **SafeSync 的 debug 故障注入**。只用 `demo` 创建的练习目录操作，**不要把下面的环境变量留在真实开发终端里，也不要用生产数据测试**。这是一种可重复的异常退出模拟，`exit(86)` 故意跳过 Rust 的 `Drop`，但不是拔电源、文件系统断电或抗恶意篡改验证。

## 1. 先准备完整样例

在 Rust AppLab 项目根目录：

```sh
cargo run -p safesync -- demo class20
cargo run -p safesync -- verify class20/backup
```

若 `class20` 已存在，换一个从未用过的目录名，并同步替换后续命令。不要删除既有资料目录。

## 2. 模拟「已经写完但尚未提交」

Linux / macOS 的 shell：

```sh
RUST_APPLAB_SAFESYNC_FAULT=backup_after_verify cargo run -p safesync -- backup class20/source class20/recovered-snapshot
cargo run -p safesync -- stages class20
```

PowerShell：

```powershell
$env:RUST_APPLAB_SAFESYNC_FAULT = 'backup_after_verify'
cargo run -p safesync -- backup class20/source class20/recovered-snapshot
Remove-Item Env:RUST_APPLAB_SAFESYNC_FAULT
cargo run -p safesync -- stages class20
```

故障注入的 `backup` **预期以退出码 86 结束**，目标 `recovered-snapshot` 尚不存在。`stages` 将列出形如 `class20/.sfs~B~recovered-snapshot~<PID>~<时间戳>~<序号>.part` 的目录，且应标记为 `READY`。在自己的输出中复制这个完整暂存路径，代入下面的 `STAGE_PATH`，而不是直接复制尖括号占位符。

```sh
cargo run -p safesync -- recover STAGE_PATH class20/recovered-snapshot
cargo run -p safesync -- verify class20/recovered-snapshot
cargo run -p safesync -- stages class20
```

观察：恢复成功后原 `.part` 路径消失，目标目录出现；无需重新读取原始源文件。**`recover` 只提交完整且与目标名及父目录匹配的备份；不是通用的“强制重命名”。**

## 3. 模拟「第一份文件刚复制完就退出」

```sh
RUST_APPLAB_SAFESYNC_FAULT=backup_after_first_file cargo run -p safesync -- backup class20/source class20/unfinished
cargo run -p safesync -- stages class20
```

PowerShell 的注入方式与上一节相同，只需更换故障点。输出应包括 `INCOMPLETE`，此时不能使用 `recover` 提交。直接在**没有故障环境变量**的终端重试即可：

```sh
cargo run -p safesync -- backup class20/source class20/unfinished
cargo run -p safesync -- verify class20/unfinished
```

该重试会生成新的安全暂存目录并完成目标；旧的不完整 `.part` 仍保留供调查。**程序不会自动删除未知暂存数据**。清理工作不属于本课的自动化命令，只能由使用者确认无运行中操作且备份数据不需要后手动处理。

## 4. 阅读对应源码（先读接口，再读实现）

- `apps/safesync/src/cli.rs`：为什么 `stages` 只接受父目录，`recover` 要同时提供暂存目录和最终目标？
- `apps/safesync/src/ops.rs`：`stages()` 的三种状态、`recover()` 的路径关系/校验、`Staging::Drop` 的正常失败清理。
- `apps/safesync/tests/recovery.rs`：测试用真实子进程退出码 86 和文件系统断言，而不只是调用一个假的 `abort()` 函数。

## 5. 动手验收

```sh
cargo test -p safesync --test recovery
cargo test --workspace --all-targets
```

请解释两个具体行为：

1. 为什么「退出码 86 后有 `.part`」不能被直接解释成损坏或可恢复？什么证据决定 `READY`？
2. 为什么恢复时要求**同一父目录、目标名匹配、目标尚不存在**，而且拒绝 `--force`？

**已知边界**：本关不提供分块断点续传、自动孤儿目录回收或断电持久性。其他进程在检查后篡改目录可能仍有竞态；当前模型是本机非敌对单进程文件操作。
