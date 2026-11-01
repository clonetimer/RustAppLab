# 第 21 关 · 用可复现数据评估备份软件，而不是猜性能

**本关中心问题**：SafeSync 使用流式 SHA-256，是否代表大文件场景下足够快、足够省内存？必须通过真实数据和可解释指标回答，不能从源码结构推导吞吐量。

## 1. 构建 Release 后测量

在 Workspace 根目录：

```sh
cargo build --locked --offline --release -p safesync
python3 scripts/bench_safesync.py --files 64 --mib 32 --output my-safesync-64files.json
```

Windows 使用 `python`（如果系统未配置 `python3`）；脚本会自动识别 `target/release/safesync.exe`。基准程序**只创建独立临时目录**，写入确定性伪随机字节，之后按真实 CLI `backup → verify → restore` 操作；最后使用 Python 标准库 `hashlib.sha256` 独立核验源、备份和恢复的每一个文件。脚本执行结束自动释放临时目录。

> 如果设置了 `CARGO_TARGET_DIR`，要把编译得到的完整可执行文件路径传给 `--binary`；例如 `python3 scripts/bench_safesync.py --binary /path/to/target/release/safesync`。

观察输出中的 `all_content_equal`、`backup_seconds`、`verify_seconds`、`restore_seconds` 和三项 `*_mib_per_second`。**数据完整性是正确性条件；速度只是该机器的一次观测值。**

## 2. 比较不同规模

```sh
python3 scripts/bench_safesync.py --files 512 --mib 8 --output my-safesync-many-small.json
python3 scripts/bench_safesync.py --files 2 --mib 32 --output my-safesync-few-large.json
```

比较结果时先回答：小文件遍历、创建目录项和 `sync_all` 成本是否导致吞吐下降？较大文件场景的磁盘缓存和 CPU 计算各占多少？当前基准只统计整体耗时，**不足以独立分离 CPU / I/O / 文件系统缓存**。同机至少重复数轮并记录每轮结果，避免把单次差异解释成算法提升。

## 3. 可独立验收的三个条件

1. 真实 Release 二进制能够完成备份、校验与恢复，且外部 `hashlib` 复核成功。
2. 使用不同的文件数量或容量后，输出仍包含有意义的速度、样本规模和环境信息。
3. 测试日志与源码版本一一对应；跨操作系统尚未运行时，不能把 CI 工作流存在当成通过证据。

核心质量检查：

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --release --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
cargo build --workspace --release
```

该项目声明 `rust-version = 1.74` 作为最低目标版本，CI 增加独立 MSRV 工作流。**只有实际运行 Rust 1.74 编译和测试成功才能宣称 MSRV 已验证；在 Rust 1.99.1 上通过不能代替这项证据。**

## 4. 源码导读与下一步优化判断

- `apps/safesync/src/sha256.rs`：64 KiB 读取缓冲区解决了什么？
- `apps/safesync/src/tree.rs`：为什么先扫描并哈希，再复制后哈希可能重复读取数据？
- `apps/safesync/src/ops.rs`：为什么每个文件都调用 `sync_all`，换来了什么和付出了什么？
- `scripts/bench_safesync.py`：为什么使用独立 Python SHA-256，而不是只信任被测程序自己的 `verify`？

最后写一条你自己的改进提案：**先明确测试负载、目标指标与不允许破坏的安全语义，再决定是否值得增加并发或增量备份**。本关不要求把可能有争议的固定性能阈值写进通用 CI。
