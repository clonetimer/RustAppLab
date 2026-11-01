# 第 23 关 · 暂存身份谜题：数据完整不代表操作正确

**本关中心问题**：一个临时目录里恰好有 `manifest.sfs` 和 `data/`，并且 SHA-256 全部正确，是否就能当作「未提交的备份」恢复？不能。必须先确认**它是哪个操作留下的**。

## 1. 不同用途的暂存目录

SafeSync 0.6.0 新建立的暂存目录以不同前缀记录用途（仅表示用途，不是身份认证）：

| 前缀 | 含义 | `stages` 显示 | 允许普通 `recover` |
|---|---|---|---|
| `.sfs~B~...part` | 未提交的备份 | `READY` 或 `INCOMPLETE` | 仅 `READY` 且路径匹配时允许 |
| `.sfs~R~...part` | 未提交的文件恢复 | `RESTORE` | 不允许，应重新运行 `restore` |
| `.sfs-...part` | 0.5.0 旧版，不含用途字段 | `LEGACY` 或 `INCOMPLETE` | 不允许；确认来源后可明确使用 `recover-legacy` |
| 非普通目录或无效类型 | 无法信任其用途 | `UNSAFE` | 不允许 |

**关键区别**：一个完整的恢复暂存目录可能**碰巧长得像完整备份目录**。判断依据必须包括暂存用途，而不仅是结构与哈希。

## 2. 运行反例：一份完整、但不能恢复的暂存

```sh
cargo test -p safesync complete_restore_stage_is_not_a_recoverable_backup_even_if_layout_matches -- --nocapture
```

阅读 `apps/safesync/tests/recovery.rs` 中该测试的操作顺序：先创建一份真实备份，再将一份包含合法 `data/ + manifest.sfs` 的目录当作普通资料备份，随后在 `restore` 已经完成复制但还没提交时退出。

验证三件事：

1. 这个恢复暂存的 `verify()` 可以返回成功，说明内容可构成合法备份。
2. `stages()` 仍将其标为 `RESTORE`，而非 `READY`。
3. 即使指定正确的父目录和目标，`recover()` 和 `recover_legacy()` 都会拒绝这份**新格式**的恢复暂存。

## 3. 旧版兼容：保留数据，但要求明确的意图

旧版 `.sfs-...part` 没有记录备份/恢复用途；仅凭目录内容**无法可靠推断真实来源**。为了保护 0.5.0 未提交的数据，同时阻止普通恢复路径静默接受它，0.6.0 将其显示为 `LEGACY`，并要求明确调用：

```sh
cargo run -p safesync -- stages <旧暂存所在父目录>
cargo run -p safesync -- recover-legacy <已人工确认来源的旧暂存路径> <原定全新备份目标>
```

尖括号是说明性占位符，**不要原样复制执行**；先确认：这是你用 0.5.0 `backup` 创建的暂存，进程已经结束，原定目标不存在，目标和暂存父目录相同、名称匹配。`recover-legacy` 仍会校验整个清单和所有文件，拒绝覆盖；但不能补造旧版缺失的用途记录，无法防止人为误判。

运行回归：

```sh
cargo test -p safesync older_untyped_stage_requires_explicit_legacy_recovery -- --nocapture
```

## 4. 给暂存做分诊：你会如何处理？

| 现场线索 | 推荐操作 |
|---|---|
| `READY`，目标不存在，确认备份进程已退出 | `recover` 后 `verify` |
| `INCOMPLETE`，缺清单或有损坏 | 保留证据、重新执行 `backup`，不要强行提交 |
| `RESTORE`，即使 `verify` 成功 | 重启 `restore` 到全新目标，勿用于 `recover` |
| `LEGACY`，无法确认原来是什么操作 | 暂停自动操作，确认来源后才决定是否使用 `recover-legacy` |
| `UNSAFE`，符号链接/非目录/异常格式 | 不跟随、不自动清除 |

`stages` 只读取状态，不提供「清理所有临时目录」命令。若同目录仍有进程运行，**不要手动移动、删除或改名其暂存**。

## 5. 验收与迁移思考

```sh
cargo test -p safesync --test recovery
cargo test --workspace --all-targets
```

用自己的话说明：「清单完整性」与「操作来源」为什么是两个不同的性质？再回答：为什么 0.6.0 需要显式的旧版兼容命令，而不能让新版 `recover` 静默接受全部 `.part`？

本关讨论的用途标签不是安全签名。面对恶意进程重命名目录、符号链接替换、断电持久性或多进程竞争，当前 Rust 教学软件**不保证安全恢复**，不能按生产级备份系统使用。
