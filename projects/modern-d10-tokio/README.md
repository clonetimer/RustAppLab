# D10-B · Tokio 与 Stream 异步工程实验室

先完成 [D10-A](../../course/BOOK_D10_ASYNC_CORE.md)，再读 [D10-B 中文讲义](../../course/BOOK_D10_TOKIO_STREAM.md)。

- learner：无答案学员工程，复制到个人目录再学习。
- reference：完整源码包的参考实现和两个编译反例，不进入学员包。
- 版本：Rust 1.100.0 / Edition 2024，Tokio 1.48.0、futures-util 0.3.34，依赖固定 Cargo.lock（复用已验收的 C1 依赖版本）。
- 首次联网准备只在个人目录运行 cargo fetch --locked；之后 cargo test --locked --offline。也可取本次 CI 的 D10-B-learner-offline.zip，安装工具链后在空 Cargo 缓存中 cargo test --frozen。
- 本课不启动公网服务，不重复开发 C1 下载器或 C2 工单服务；网页工作台尚未接入，当前为独立 CLI 选修。

维护者从完整源码根目录执行 python scripts/modern_d10b_check.py --output d10b-evidence，包含真实 vendor、离线学员包和迁移路径验证，首次准备依赖需网络。
