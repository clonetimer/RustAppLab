# 我的现代标准库实验

先安装 Rust 1.100.0，再复制本目录作为自己的工程。在 src/lib.rs 从 TODO(1) 开始修改，每步运行 cargo test --locked --offline --test contracts step1_（依次换成 step2_ … step16_）；第 8 步还需实现 TODO(8b)。初始测试应失败，成功后再继续，不改合同测试。

完成后运行 cargo test --locked --offline --all-targets 和 cargo run --locked --offline --example telemetry -- 7 bad 120 0 。应报告 good=3; bad=1; outliers=[120]; changes=[-7]。先预测，再换输入（空列表、负值、最大整数、重复 token、边界）验证。最后在 LEARNING_LOG.md 记录自己的改动、实际结果与原因。讲义见 course/BOOK_D9_STDLIB_CARGO.md。
