# E1-A · 真实 HTTPS 与身份授权

[中文讲义](../../course/BOOK_E1_TLS_AUTH.md)。复用 C2 的固定参考业务和持久化，不重写 ServiceDesk。Rust 1.100.0；Tokio 1.53.3、Axum 0.8.8、rustls 0.23.46、tokio-rustls 0.26.6、subtle 2.6.1；测试用 rcgen 0.14.10。

当前源码候选第一次新增 TLS 依赖，完整传递锁由独立 CI 的 resolve job 实际生成、RustSec 审计并保存 SHA-256，Windows/Linux 消费同一锁文件。未把尚未由 Cargo 验证的传递图伪造为 Cargo.lock；后续将冻结通过验收的锁。普通学员使用通过验收的独立 E1-A-learner-offline.zip，而不是直接复制本目录的半成品依赖图。完整源码中的 learner 依赖 ../../advanced-c2/reference；独立包中固定已学 C2 作为提供的 support/c2，与 E1-A 自己的解答分离。普通无答案 learner-kit 只提供 E1 模板和导航，不混入 C2 答案。

完成四步后 cargo test --frozen --all-targets，cargo run --frozen --example local_demo。只监听回环；不接触公网、真实凭证或系统全局信任库。浏览器界面、用户账号生命周期、数据库/可观测性、生产部署不在 E1-A。
