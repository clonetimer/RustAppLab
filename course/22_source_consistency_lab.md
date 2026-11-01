# 第 22 关 · 抢救正在变化的文件：一份备份究竟何时可信？

**本关中心问题**：日志还在写、配置文件又被修改时，一次成功的 `copy` 是否意味着得到了一份一致的备份？本关不是比谁复制得快，而是让你亲手验证「拒绝提交比悄悄保存错误快照更重要」。

## 1. 五分钟准备：先拿到成功样例

在 Rust AppLab 项目根目录执行（使用不存在的新演示目录）：

```sh
cargo run -p safesync -- demo sfs-lab22
cargo run -p safesync -- verify sfs-lab22/backup
cargo run -p safesync -- backup sfs-lab22/source sfs-lab22/snapshot-2
cargo run -p safesync -- verify sfs-lab22/snapshot-2
```

观察两个路径的区别：`source/` 是会变化的数据；`snapshot-2/data/` 是正式提交的备份。两个快照均须通过 `verify`，且新目标不得预先存在。

## 2. 追踪三个保护点

打开 `apps/safesync/src/ops.rs`，从 `backup()` 开始沿调用顺序追踪：

1. **扫描前**：`tree::scan(source)` 得到预期目录结构、每份文件的大小和 SHA-256。
2. **复制中**：`copy_entries()` 每次从文件读取缓冲区时累计长度与 SHA-256；发现与预期不一致即报错，不再继续提交。
3. **提交前**：完整写入的暂存目录通过 `verify()`，再扫描源目录一次；内容和目录结构必须仍与最初的清单相同，才能提交。

**想一想**：复制后既校验了暂存，为什么还要复查源目录？因为数据在复制完成后、提交开始前也可能继续被另一个进程修改。

## 3. 用确定性回归测试制造一次文件竞态

不要试图靠手动修改一个 1 KiB 文件来「碰巧」撞中竞态。运行测试钩子，它会严格在**暂存已完整验证之后、最终源目录复查之前**修改一个练习文件：

```sh
cargo test -p safesync source_changed_after_stage_verified_is_not_committed -- --nocapture
```

在 `apps/safesync/src/ops.rs` 文件底部找到该测试，逐项核对断言：

- 修改后的原始文件仍然存在；
- 函数返回错误，而非 `BackupStatus::Created`；
- 正式目标 `snapshot` 不出现；
- 正常返回错误时 `Staging::Drop` 清理暂存目录。

本测试只改写临时目录里的测试文件，不会修改你的真实资料。

## 4. 模拟磁盘写入失败：为什么不能留下半份「成功」快照

在 Linux/macOS shell 运行（测试内部会启动独立子进程，并只对子进程设置故障环境变量）：

```sh
cargo test -p safesync simulated_mid_file_write_error_rolls_back_and_leaves_source_intact -- --nocapture
```

PowerShell 也可以直接运行这条测试，不需要设置环境变量：测试内部会启动独立子进程并注入错误。

```powershell
cargo test -p safesync simulated_mid_file_write_error_rolls_back_and_leaves_source_intact -- --nocapture
```

看 `apps/safesync/tests/recovery.rs` 的断言：256 KiB 测试文件在首个缓冲区写入后触发模拟 I/O 错误，目标目录不得存在、原文件字节不变、异常暂存必须清理。**这是可控 I/O 失败测试，不是实际将磁盘写满，更不是断电/掉电持久性测试。**

## 5. 本关实际交付与验收

产出一份简短的实验记录，包含：执行命令、错误发生在哪一步、`snapshot` 是否产生，以及一句你对「最终扫描仍不是原子快照」的解释。完整回归命令：

```sh
cargo test -p safesync --test recovery
cargo test -p safesync --lib
cargo test --workspace --all-targets
```

**边界判断题**：若另一个进程在最终扫描结束后、`rename` 前又改写源数据，当前实现能否保证发现？答案是**不能保证**。若工作负载要求严格一致性，应让写入者暂停、使用文件系统快照或由应用提供一致性导出；不能靠增加无限次数的扫描来假装拥有原子性。
