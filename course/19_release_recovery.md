# 第 19 关 · 让备份工具真正进入日常开发流程

**中心问题**：每次手动备份都不难，难的是重复执行、不完整输出和自动化中的错误处理。一个脚本如何判断“已有相同备份”“备份已损坏”以及“需要生成新快照”？

## 真实场景：迭代与版本化快照

```sh
cargo run -p safesync -- demo my-sfs-19
cargo run -p safesync -- backup my-sfs-19/source my-sfs-19/backup
```

这次重复执行会输出 `Unchanged`（如果你没有修改原始样例），不会再次写入已有目录。现在修改 `my-sfs-19/source/README.txt`，再执行同样的 `backup`：应失败，不允许默默把旧快照改成新数据。

想保留两个版本，应使用两个**不同的备份目录**：

```sh
cargo run -p safesync -- backup my-sfs-19/source my-sfs-19/backup-v2
cargo run -p safesync -- verify my-sfs-19/backup-v2
cargo run -p safesync -- restore my-sfs-19/backup-v2 my-sfs-19/recovery-v2
```

这就是基于不可覆盖快照的最小版本管理策略。它尚不具备增量备份、压缩、去重、自动计划和定期保留策略。

## 开发挑战：让脚本正确读取输出

新增 `verify <备份目录> --json`，成功时 stdout 输出单行机器可读 JSON，含 `"verified":true`、`"files":N`、`"directories":N`、`"bytes":N`，失败时返回非零退出码，错误仍打印 stderr。默认版本**未实现**。

```sh
cargo test -p safesync --test challenges mission_19_verify_json -- --ignored --exact
```

建议不要新建一个仅服务此挑战的“统计系统”：已有 `ops::Summary` 足够表达这些字段。用普通标准库构造 JSON 时，只序列化固定键和整数，避免手写任意字符串插值。

## 发布前检查

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
cargo build --workspace --release
```

**完整验收包括**：普通目录、空目录、二进制文件、中文文件名、单字节损坏、符号链接拒绝、重复执行、已存在恢复目标、非法清单与释放临时目录。学员挑战是单独的 `--ignored --exact`，不能把基础测试全绿当成已经实现 `--json`。

进阶思考：若未来要支持自动备份调度、多平台文件权限、长周期大文件、增量版本和被恶意篡改的备份仓库，哪些部分必须重新设计？这些尚不是 0.4.0 已解决的问题。
