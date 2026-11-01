# 参考项目借鉴记录（2026-10-28）

本项目教学方案参考下列公开仓库的 README、入口说明或项目介绍。主要借鉴**教学组织方式和产品交付理念**，不复制参考项目的源码。

| 项目 | 已核对的公开证据 | 对 Rust AppLab 的取舍 |
|---|---|---|
| [MiniMCU](https://github.com/clonetimer/MiniMCU) | `README_START_HERE.md` 规定 15 分钟 Guided Demo、分阶段课程、读源码/跟踪/CTest 的顺序 | 保留“首用入口 + 分阶段 + 自动验收”；不移植硬件模拟内容 |
| [MiniMPU](https://github.com/clonetimer/MiniMPU) | README 介绍五步先导流程及 GUI 页面，侧重先学软件使用再学 MPU 原理 | 保留“先软件体验后原理”，让首个可见结果是 HTML 报告 |
| [Rustlings](https://github.com/rust-lang/rustlings) | 小练习通过编译器和自动测试即时反馈 | 借用 RED→GREEN 关卡，但关卡必须扩展产品的真实功能 |
| [Rust CLI Book](https://github.com/rust-cli/book) | 面向命令行应用的工程学习资源 | 第一产品选择 CLI，减少初学阶段 GUI 框架复杂度 |
| [mini-redis](https://github.com/tokio-rs/mini-redis) | 利用简化 Redis 实例讲授 Tokio 异步应用 | 网络/并发留在 LocalHub 阶段，避免抢占入门主线 |
| [Build Your Own X](https://github.com/codecrafters-io/build-your-own-x) | 汇集大量逐步构建软件的教程 | 借鉴由作品驱动的学习方式，但限制每一阶段的功能边界 |

核心设计结论：**学员不应该在体验一个真实结果之前，被迫阅读大量语言理论或框架代码。** 每个关卡先定义可观察的产品行为，再说明其中的 Rust 知识，随后用测试验证学员是否真正改变了软件。
