# E1-A · 真实 HTTPS 与身份授权

[中文讲义](../../course/BOOK_E1_TLS_AUTH.md)。复用 C2 的固定参考业务和持久化，不重写 ServiceDesk。Rust 1.100.0；Tokio 1.53.3、Axum 0.8.8、rustls 0.23.46、tokio-rustls 0.26.6、subtle 2.6.1；测试用 rcgen 0.14.10。

完整传递锁已从首轮通过 RustSec 的真实 CI 产物冻结，101 个 registry 包、SHA-256 `44ef895eb533fcf092d77f52c2488513d9dddb1b8faa0ad034d4cf7615d2c3fd`。Windows/Linux 必须重新执行 locked/frozen 与同一固定数据库审计。普通学员使用通过验收的独立 E1-A-learner-offline.zip。完整源码中的 learner 依赖 ../../advanced-c2/reference；独立包中固定已学 C2 作为提供的 support/c2，与 E1-A 自己的解答分离。普通无答案 learner-kit 只提供 E1 模板和导航，不混入 C2 答案。

完成四步后 cargo test --frozen --all-targets，cargo run --frozen --example local_demo。只监听回环；不接触公网、真实凭证或系统全局信任库。浏览器界面、用户账号生命周期、数据库/可观测性、生产部署不在 E1-A。
