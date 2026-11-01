# Tokio 异步网络的下一阶段开发准入（0.16.0 起）

当前 AsyncBoard 已完成 Rust `async/await` + 教学执行器，**不是 Tokio 网络 I/O**。NetFetch、ServiceDesk 同步网络基础版的课程与实际端到端结果都保留。

## 真正的进阶目标（尚未完成，不能按已通过宣传）

1. 获得 Tokio、Reqwest、Axum、Serde 等真实 crate 发行包，锁定 Cargo.lock，记录来源和哈希；维护者需要使用可信 crates.io 镜像或官方仓库。不要求零基础学员自行下载全部依赖。
2. 为 NetFetch 设计异步队列：Tokio TCP/Reqwest HTTP/TLS、连接超时、总超时、最大并发、取消后文件清理、重试策略，和固定服务端进行真实对比测试。
3. 为 ServiceDesk 设计 Axum 服务：路由、JSON、状态共享、持久化、优雅关闭、并发客户端压力测试；不能把标准库同步 HTTP 源码改名即标记完成。
4. Windows MSVC 原生、Linux/macOS、离线 vendor 包和最低 Rust 版本必须独立验证。库升级需更新证据，不能推断兼容。

## 学员必会的基础

先理解 `Future` 的惰性、`.await` 和 `Poll::Pending`，再学 Tokio 的 `spawn` 与 `JoinHandle`，最后学习具体 HTTP 协议、取消与背压。生产代码不应复制 `apps/asyncboard/src/runtime.rs` 作为通用网络运行时。官方参考：https://tokio.rs/tokio/tutorial 及 https://rust-lang.github.io/async-book/。
