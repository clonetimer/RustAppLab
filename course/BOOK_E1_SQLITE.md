# E1-B · 从工单日志到真实数据库事务与结构化观测

[现代化路线](MODERN_RUST_ROADMAP.md) · [E1-A HTTPS](BOOK_E1_TLS_AUTH.md) · [工程入口](../projects/modern-e1-sqlite/README.md)

本阶段复用原 C2 的 servicedesk::apply（标题、数量、编号、关闭冲突）与 E1-A TLS 1.3/角色策略。只替换持久化和观测层。使用 Rust 1.100.0、Rusqlite 0.43.0 bundled SQLite、已验证的 TLS/Axum/Tokio 直接版本。先完成 E1-A，取得通过验收的独立学员包；普通学员套件仅含模板。新增数据库依赖完整锁由真实 Cargo 从 E1-A 冻结锁出发解析，RustSec 同快照审核后双平台消费同一哈希；冻结前不作为发布版本。

| 任务 | 学员实现 | 行为验收 |
|---|---|---|
| 1 create_batch | 1–16 条请求、BEGIN IMMEDIATE、复用域规则、参数绑定与 audit | 中文与引号数据、错误中途回滚、audit 插入失败导致业务回滚、重启后读取 |
| 2 close | 业务关闭规则、UPDATE 一条、audit 与状态一同提交 | 不存在、重复关闭冲突、故意 audit 失败后未关闭、重新打开 |
| 3 enqueue | 有界 Tokio mpsc 的 try_send | 实际满队列 Busy，已关闭 Closed，禁止假成功/静默丢弃 |
| 4 AccessLog::record | 固定事件字段、路线类别、容量与丢弃计数 | 只保留有限最近事件；状态、耗时、请求编号；不保存 raw URL、标题、Authorization |
| 5 集成 | 固定提供 E1-A TLS 与授权包装、单 DB actor、真实请求 | 错误信任拒绝，401/403 不写库；合法 POST/close，多个并发请求、允许明确 503，停止后重开文件及真实 audit |

## 数据、权限与事务

初始 schema 有 application_id 和 user_version，init 使用 create_new、禁止覆盖；open 不自动创建，要求本地可信普通文件和正确 schema 版本。SQLite foreign_keys 开启、同步级别 FULL、busy_timeout 一秒。事务中先按现有域规则计算，再使用参数绑定写 tickets 和 audit_events；两者同事务提交，任一错误自动回滚。测试使用真实 SQLite trigger 让 audit 写入失败，不能用 mock 假造回滚。用户标题不会拼接成 SQL，合法引号内容按数据存储。事务教学不等于断电/恶意目录/多进程性能保证，不能把旧 C2 文本日志直接当 SQLite 打开；数据迁移不在当前任务。

每步在个人包执行 cargo test --frozen --test contracts step1_（替换编号）。完成全部后 cargo test --frozen --all-targets，再 cargo run --frozen --example local_demo。实际 demo 通过本地 HTTPS 创建/关闭工单并重开数据库，打印不含凭证或标题的 JSON Lines。补自己的迁移测试，例如 3 个合法标题后跟空标题、重复关闭、队列满后的重试策略。不要直接重试已接收的写请求并假定幂等。

## 资源控制与观测

固定单写者 spawn_blocking 持有 Connection；mpsc 容量 8，try_send 满时立即给出 503（busy），不会假装执行。已入队任务即使 HTTP 客户端断开，也可能提交；客户端超时不证明数据库回滚。丢弃全部 Backend 发送端后等待 actor 完成已接收工作；TLS 仍由 E1-A 限制 16 个在途握手与两秒期限。尚无整体连接上限、全局速率限制、幂等键、强制终止正在执行请求等生产保障。

持久业务 audit_events 只含序号、操作类型、工单 ID，与业务事务一同提交。HTTP 访问日志是另一种信息：outer middleware 记录每个已完成请求的固定方法、路线类别、HTTP 状态、耗时及进程内序号；容量 1–256，超过容量丢最旧并计数，输出 JSON Lines。拒绝请求也可观测，但没有业务 audit 行。日志不保存完整 URL、请求体、标题、凭证或任意错误字符串；不把临时内存日志冒称远程持久日志/分布式追踪。请求编号由服务端生成，重启重置；不宣称跨进程唯一。只读/写入凭证仍为教学固定角色，不是账号生命周期管理。

## 验收

维护者用 scripts/modern_e1b_check.py：resolve 从 E1-A 已冻结锁真实 Cargo 解析并审计（vulnerability/unsound 拒绝）；replay 使用同一 Cargo.lock SHA-256，准备 vendor，真实 Debug/Release、四步 RED/GREEN、集成及 CLI、ZIP 中文路径空缓存 frozen。源与学员测试保持一致；独立 E1-B 包的 support/e1a 提供前置完成源码而非 E1-B 答案，普通 learner-kit 不暴露前置答案。实际 CI 未通过前不宣称完成。网页、用户试用、跨进程持久化策略、正式生产审计另验。

参考：[Rusqlite 0.43.0](https://docs.rs/rusqlite/0.43.0/rusqlite/) · [SQLite 事务](https://www.sqlite.org/lang_transaction.html) · [SQLite 外键](https://www.sqlite.org/foreignkeys.html) · [Tokio mpsc](https://docs.rs/tokio/1.53.3/tokio/sync/mpsc/index.html)。
