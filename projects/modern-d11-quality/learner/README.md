# 我的帧处理质量实验

安装 rustup toolchain install 1.100.0 --profile minimal --component rustfmt --component clippy，然后复制整个 learner 目录到个人目录。按 src/lib.rs 的 TODO(1)–TODO(4) 修改。每一步运行 cargo test --locked --offline --test contracts step1_（替换编号）。初始四项测试应失败。

全部完成后：

```text
cargo test --locked --offline --all-targets
cargo run --locked --offline --example frame -- "温度"
cargo run --locked --offline --example frame -- "温度" corrupt
cargo run --locked --offline --release --example measure -- 1000
```

第一条运行命令应返回 decoded_bytes=6；损坏帧应拒绝并非零退出。基准打印两个算法五次测量的中位数，不能只凭一台机器的结果宣称哪个永远更快。修改输入、阈值和基准轮数，写一条新的属性测试并记录原因。公开合同仅核验行为，不代替个人理解。

断点实验：cargo build --locked --offline --example debug_probe。Ubuntu 安装 GDB 后对实际二进制执行 break debug_probe::diagnose_frame、run、info args、bt、continue。Windows 可使用支持 Rust 的已安装 IDE 调试器，在 examples/debug_probe.rs 的 diagnose_frame 函数内设断点，观察 expected=42、actual=43；自动 CI 不证明你的 IDE 已可用。

完整讲义见平台 course/BOOK_D11_QUALITY.md；格式化及 Clippy 实验位于同级 tools-practice。保留 LEARNING_LOG.md。
