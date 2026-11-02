# 参考项目借鉴记录

## 2026-11-02：Rust Book 中文站与官方资料

本轮实际阅读 [rust.bootcss.com](https://rust.bootcss.com/)，并用 [官方 Rust Book](https://doc.rust-lang.org/book/) 和 Cargo 文档核对行为。中文站首页写明 Rust 1.37.0 / Edition 2018；本项目保持 Rust 2021 与最低 Rust 1.74.0，按主题建立对照。

| 参考内容 | 本项目落实 |
|---|---|
| [引用与借用](https://rust.bootcss.com/ch04-02-references-and-borrowing.html)、[Result](https://rust.bootcss.com/ch09-02-recoverable-errors-with-result.html) | `learn reading [工程]` 将现有作品与相关概念、思考实验、实际检查入口相连 |
| [测试组织](https://rust.bootcss.com/ch11-03-test-organization.html)、[官方对应章节](https://doc.rust-lang.org/book/ch11-03-test-organization.html) | 进阶 B 验证公开接口、CLI list、独立报告进程；用真实错误变体确认检查器发现回归 |
| [Cargo 工作空间](https://rust.bootcss.com/ch14-03-cargo-workspaces.html)、[官方 Workspace](https://doc.rust-lang.org/cargo/reference/workspaces.html) | 显式测试独立 reference 工作区，将默认/strict feature、Debug/Release、no-default 纳入工程检查 |
| [Cargo test](https://doc.rust-lang.org/cargo/commands/cargo-test.html)、[文档测试](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html) | 所有目标测试与 doctest 分别执行，新增可运行 API 示例，零测试、ignored 和失败均按门槛判断 |
| [现代 async](https://doc.rust-lang.org/book/ch17-00-async-await.html) | AsyncBoard 导航同时提供线程基础和当前异步资料，标注现有教学执行器范围 |

课程解释和思考实验由本项目编写，参考链接用于进一步阅读。学习说明见 [对照学习指南](course/BOOK_GUIDE.md)。

## 2026-10-28：原有参考项目

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
