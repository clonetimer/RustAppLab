# 我的异步核心练习

先安装 Rust 1.100.0，再复制本目录到自己的新目录。进入副本运行 rustc --version。依次完成 src/lib.rs 的 TODO(1) 至 TODO(6)。不修改 src/runtime.rs 或 tests/contracts.rs。

每步运行 cargo test --locked --offline --test contracts step1_（按编号替换）。初始预期失败，不能只改测试通过。完成后运行 cargo test --locked --offline --all-targets 和 cargo run --locked --offline --example async_workbench。

先预测：join 何时返回、哪个子 future 先 Ready、为什么完成后不能重复 poll、竞速的 loser 是否仍然运行、Waker 由谁触发。写下你的实际输出与原因，再换不同输入与等待顺序。在 LEARNING_LOG.md 保留新测试和解释。单根 executor 只用于可信的短时间非阻塞 poll，不是生产运行时。

完整中文讲义在 course/BOOK_D10_ASYNC_CORE.md。公开合同验证可观察行为，不代表真人已经掌握异步。
