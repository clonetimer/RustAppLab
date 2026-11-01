# 第 18 关 · 为什么恢复前必须拒绝覆盖？

**中心问题**：软件能恢复文件，不代表它可以随意覆盖目标。用户可能把恢复目录误指向一个仍在开发的工作区；任何已有文件都不应在没有明确策略的情况下被覆盖。

## 可观察实验

```sh
cargo run -p safesync -- demo my-sfs-18
cargo run -p safesync -- restore my-sfs-18/backup my-sfs-18/restored
```

第二条命令应报“目标已存在”，因为 `demo` 已经创建过 `restored/`。用新目录名重试：

```sh
cargo run -p safesync -- restore my-sfs-18/backup my-sfs-18/restored-2
```

此时成功。再试把备份目标设在源目录内部：

```sh
cargo run -p safesync -- backup my-sfs-18/source my-sfs-18/source/backup-inside
```

预期失败，且 `backup-inside/` 不出现。否则备份过程可能不断把自己的临时文件再次读入，造成递归污染。

## 对应的工程知识

`src/ops.rs` 中 `Staging` 实现了一个很小的 RAII 资源管理器：临时目录创建后，即使某一步 `?` 早退，`Drop` 也会尝试删除未提交的临时目录；全部验证完成才通过 `fs::rename` 提交新目录。这是**有限的原子提交策略**，不是通用数据库事务。断电后的目录持久化、并发敌对进程与权限恢复仍不保证。

`src/tree.rs` 中 `relative_path` 拒绝 `../`、绝对路径和模糊分隔符；`verify` 会拒绝目录里出现未登记的文件、损坏记录和符号链接。试想：如果直接把清单里的每个字符串 `join` 到恢复目录，会出现什么路径穿越问题？

## 开发挑战：先预演，再恢复

新增 `restore <backup> <全新目标> --dry-run`：只校验备份、展示将恢复的文件/目录总数，**不能创建目标目录**。期望输出包含 `Dry-run`，失败时不能留下部分结果。基础版本暂不支持该参数。

```sh
cargo test -p safesync --test challenges mission_18_restore_dry_run -- --ignored --exact
```

从 `cli.rs` 解析参数开始，再在 `lib.rs` 增加干运行分支；优先复用 `ops::verify`，不要复制完整的目录扫描逻辑。

完成条件：能解释“验证后恢复”和“直接覆盖”区别，挑战由红变绿。下一关：[`19_release_recovery.md`](19_release_recovery.md)。
