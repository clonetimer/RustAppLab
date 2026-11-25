# E1-A · 用真正的 TLS 保护现有 C2 网络服务

[现代化路线](MODERN_RUST_ROADMAP.md) · [E1-0 安全基线](NETWORK_SECURITY_BASELINE.md) · [本课工程](../projects/modern-e1-tls/README.md)

先完成 C2 与 D10。保持 C2 的 Axum JSON 业务、单写者持久化、状态接口不变，在独立 Rust 1.100.0 工程上增加 TLS 与访问策略。用户取得通过 CI 的 E1-A-learner-offline.zip 后复制到个人新目录。首轮 Ubuntu resolve job 从 C2 锁实际解析 101 个 registry 包并通过固定 RustSec 数据库审计；现将原始产物 SHA-256 `44ef895eb533fcf092d77f52c2488513d9dddb1b8faa0ad034d4cf7615d2c3fd` 字节一致冻结为两份 Cargo.lock。后续只允许 locked fetch 和重审计；两平台验证同锁及真正 vendor 空缓存。

| 任务 | 实现 | 实际验收与迁移 |
|---|---|---|
| 1 认证 | authenticate 验证唯一 Authorization，Bearer 大小写无关，严格 64 hex，两个固定长度值使用 subtle 比较 | 缺失、错误、重复、错误 scheme；有效 Reader/Writer；不输出凭证 |
| 2 授权 | decide 默认拒绝未列出的路由；仅 GET /health 公开；读写角色区分 | 401 与 403；reader 禁止 POST；浏览器 Origin 写入拒绝；新增自己的错误路径 |
| 3 TLS 配置 | rustls ring provider，TLS 1.3，DER 证书和 PKCS8 密钥，HTTP/1.1 ALPN | 错误 DER、证书和密钥不配对拒绝；不关闭客户端验证 |
| 4 握手 | tokio-rustls TlsAcceptor，限制握手时间 | 真正 TLS1.3 协商；空闲 socket 超时；不回退明文 |
| 5 集成 | 使用固定 C2 router 和 actor | 可信/不可信证书、错误主机名、明文拒绝、并发握手、实际 HTTP 状态、拒绝写入不改日志、允许写入及重启持久化 |

## 学习方法

每次只改 src/lib.rs 对应 TODO，执行 cargo test --frozen --test contracts step1_（替换编号）。全部完成后 cargo test --frozen --all-targets 和 cargo run --frozen --example local_demo。演示使用随机临时证书与凭证，通过真实 TLS client 请求本机服务；不通过 stdout 模拟成功。对照实际 401/403/201/200 和持久化。公开合同不能证明独立理解，记录自己的额外输入与解释。

本地证书直接由本次客户端显式信任，SAN 为 localhost。错误信任和错误主机名必须导致握手失败；不把自签证书加入系统全局信任库。TLS 保护传输，不自动识别用户；Bearer token 代表持有者权限，不是完整账号、会话、OAuth 或密码存储。常数时间比较限定相同长度的两枚已配置 token，不宣称整个服务完全侧信道安全。不要将真实 token 写进日志、CLI 参数、仓库或网页 localStorage。环境变量只是教学配置，不是生产密钥管理。

保留 C2 旧 HTTP 浏览器写入校验。E1-A 只开放 GET /health、GET/POST /api/tickets、POST /api/tickets/{id}/close；所有其他路由拒绝，不开启原 HTML 页面或跨源策略。写入还要求原 C2 X-RustAppLab-Client: 1 且不带 Origin；此 header 本身不是身份认证。拒绝请求不得修改日志。每次只允许最多 16 个在途握手、每次 2 秒超时；这不是整体 HTTP 连接限制、生产级 DoS 防护或强制关闭已运行请求。E1-B 增加更完整的持久化、事务、结构化日志、资源预算和关闭恢复。

## 验收与离线交付

维护者使用 scripts/modern_e1a_check.py。resolve 阶段验证仓库已冻结的传递锁并重审计 TLS 图，若存在已知 vulnerability 或 unsound 警告则不能交付。Windows/Linux 用同一哈希的锁进行 cargo vendor --locked；参考 Debug/Release、四步 RED/GREEN、实际 HTTPS 集成、命令行演示、独立学员 ZIP 搬移与空缓存 frozen 验收。提供的 support/c2 是前置课固定代码，而不是 E1 答案；全局 learner-kit 不包含 C2 解答。CI 只在临时副本生成锁、证书、凭证和测试文件，源码不改。固定数据库无实时/系统依赖认证。

参考：[rustls 0.23.46](https://docs.rs/crate/rustls/0.23.46) · [tokio-rustls 0.26.6](https://docs.rs/crate/tokio-rustls/0.26.6) · [Axum](https://docs.rs/axum/0.8.8/axum/) · [rcgen 0.14.10](https://docs.rs/crate/rcgen/0.14.10)。

独立 CLI 先执行 `cargo run --frozen -- init <新工单日志>`，再配置 TLS/凭证并 serve；init 沿用 C2 非覆盖式创建，serve 不自动创建或覆盖文件。首轮参考合同失败是集成夹具没有 init；修复后保留原五项合同并增加未初始化启动拒绝与 CLI 不覆盖验证。
