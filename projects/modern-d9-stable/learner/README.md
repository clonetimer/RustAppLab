# 我的现代数据处理实验

首次安装：rustup toolchain install 1.100.0 --profile minimal --component rustfmt --component clippy 。进入个人副本，运行 rustc --version 确认工具链。

在 src/lib.rs 找到 TODO(1) 到 TODO(9)，一次只改一个。每步运行 cargo test --locked --offline --test contracts step1_（将数字换成对应步骤）。初始九项业务测试预期全部失败，不改测试去获得通过。全部完成后运行 cargo test --locked --offline --all-targets 和 cargo run --locked --offline --example batch -- 7 0 3 。期望 accepted=2; stored=[100, 7, 0, 3] 。再试 7 bad 3，观察拒绝和保留的原始数据。

建议保存一次自己的新输入、边界测试和逐项解释，见 LEARNING_LOG.md。完整讲义在 RustAppLab 的 course/BOOK_D9_STABLE_FEATURES.md。公开合同验证行为，并不检查你是否采用指定语法或独立理解；保留你的源码与解释供复核。
