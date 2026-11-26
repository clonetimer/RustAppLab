# FaultScope 学员操作

先复制到个人目录。安装 Rust 1.100.0（首次需要网络），其余依赖为空，可 frozen 离线构建。只完成 src/lib.rs 的四个 TODO；不修改固定 synthetic、tests 或 examples。逐个执行：

- cargo test --frozen --test contracts step1_
- cargo test --frozen --test contracts step2_
- cargo test --frozen --test contracts step3_
- cargo test --frozen --test contracts step4_
- cargo test --frozen --all-targets
- cargo run --frozen --release --example faultscope

最后将自己的训练/留出结果及真实计时写入 LEARNING_LOG.md。再在个人实验副本改变训练轮数或反转标签，预测并比较结果，不改原始合同。测试通过不等同真实学员掌握或实际故障诊断性能。
