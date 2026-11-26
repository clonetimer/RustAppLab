# 我的嵌入式监测器

先安装 Rust 1.100.0；取得验收通过的 E3-learner-offline.zip 并解压到自己的新目录。Windows/Linux 均可学习普通主机合同，不需要实物开发板。先完成 src/lib.rs 的 TODO(1)、TODO(1b)、TODO(2)、TODO(3)、TODO(4)，每步运行：

    cargo test --frozen --test contracts step1_
    cargo test --frozen --test contracts step2_
    cargo test --frozen --test contracts step3_
    cargo test --frozen --test contracts step4_
    cargo test --frozen --all-targets
    cargo run --frozen --example host_demo

第 5 项合同是同一核心的离散调度/控制集成。主机脚本不是硬件时序证据。保留原合同并做自己的边界测试。

ARM 实验需先显式安装固定目标（首次联网）：

    rustup target add thumbv7m-none-eabi --toolchain 1.100.0
    cargo build --frozen --release --target thumbv7m-none-eabi --features firmware --bin firmware

Ubuntu 装 qemu-system-arm 后执行真实模拟 CPU：

    qemu-system-arm -M lm3s6965evb -cpu cortex-m3 -nographic -monitor none -serial none -semihosting-config enable=on,target=native -kernel target/thumbv7m-none-eabi/release/firmware

预期出现真实 timer_irq_count、四条 sample 记录、simulated_interlock=latched output_command=low 与 E3_QEMU_PASS。若无法退出，终止 QEMU 并排查实际 IRQ/编译日志。不要用真实开发板直接烧录这个内存布局；真实传感器、电气连接、输出回读和时序需单独工程验证。不能以模拟 output_command=low 证明真实电气安全。

参考完整平台 course/BOOK_E3_EMBEDDED.md，记录 LEARNING_LOG.md。
