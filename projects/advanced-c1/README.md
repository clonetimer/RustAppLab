# C1 · 真实异步下载

`reference/` 是 Tokio / Reqwest 完整参考软件，`learner/` 是三个核心 TODO 的学员工作区。两者使用相同 Cargo.lock 和合同测试；各自独立于根工作区，使用 Rust1.85.0。基础平台继续支持1.74。

完整命令、参数、三级提示、离线包与边界见 [C1 中文讲义](../../course/ADVANCED_C1.md)。使用平台的 `advanced-c1 start/prepare/check/next/lesson` 入口；维护者用 `scripts/c1_check.py` 和 `scripts/package_c1.py` 生成源代码绑定的验证记录，维护脚本需要 Python3.11 及以上。

回环 HTTP 不代表公网/TLS 或 Axum 课程已完成。教学检查只用于可信学员代码。
