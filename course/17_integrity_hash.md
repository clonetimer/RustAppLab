# 第 17 关 · 文件明明在，为什么恢复仍然会失败？

**中心问题**：一个备份文件只被修改了一个字节，名字、长度甚至修改时间都可能没有明显变化。如何知道它已经不能作为可靠的恢复来源？

## 先做破坏实验（只修改生成的演示文件）

```sh
cargo run -p safesync -- demo my-sfs-17
cargo run -p safesync -- verify my-sfs-17/backup
```

现在打开 `my-sfs-17/backup/data/README.txt`，只把一个汉字或字母改掉，并保存。再次执行：

```sh
cargo run -p safesync -- verify my-sfs-17/backup
cargo run -p safesync -- restore my-sfs-17/backup my-sfs-17/should-fail
```

预期：**两条命令都应失败并返回非零退出码**；`should-fail/` 不会被创建。先不要“修复”这个行为：拒绝从损坏的备份中恢复才是正确功能。

## 学习对应的 Rust 知识

- `src/sha256.rs`：`Read::read` 如何以 64 KiB 为块计算摘要？为什么不能把整个大文件一次性加载进内存？
- `src/manifest.rs`：记录了哪些信息？路径为什么使用 UTF-8 字节的十六进制编码？
- `src/ops.rs`：`compare` 何时用于阻止提交？为什么校验不能只比对文件大小？

**重要限制**：SHA-256 只提供完整性检测，不代表文件来源可信。若有人能同时修改文件和清单，本工具没有数字签名来识别主动篡改。文件权限、ACL、时间戳和稀疏文件属性也不在本版备份范围。

## 开发挑战：安静的校验模式

真实自动化脚本经常不希望输出一长段成功信息，却需要一个准确的退出码。请添加：

```sh
cargo run -p safesync -- verify my-sfs-17/backup --quiet
```

期望：成功时 stdout 完全为空、退出码为 0；损坏时退出码非 0。**目前未实现**，先执行失败态测试：

```sh
cargo test -p safesync --test challenges mission_17_verify_quiet -- --ignored --exact
```

修改 `cli.rs` 的枚举与解析，以及 `lib.rs` 中 `Verify` 命令输出分支；不要删除 `verify` 中的 SHA-256 校验，也不能在错误时故意返回成功。

完成条件：该挑战由红变绿，并且 `cargo test -p safesync` 仍全部通过。下一关：[`18_safe_restore.md`](18_safe_restore.md)。
