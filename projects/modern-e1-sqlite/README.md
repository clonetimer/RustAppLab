# E1-B · 真实 SQLite 事务与可观测性

先完成 [E1-A HTTPS](../../course/BOOK_E1_TLS_AUTH.md) 和 C2，再读 [本课中文讲义](../../course/BOOK_E1_SQLITE.md)。保持 Rust 1.100.0，新增固定 Rusqlite 0.43.0（bundled SQLite），复用 E1-A TLS/权限、原 servicedesk::apply 业务规则；不重写认证或 HTTP 底层。

learner 提供四个 TODO 和五个实际合同。完整源码 reference 是解答；独立离线学员包供应已经学过的 E1-A/C2 支持源码与 vendor，不含 E1-B 答案。普通学员套件仅提供模板和讲义，需要另取通过 CI 的 E1-B-learner-offline.zip。真实 Cargo resolve/audit 产物已冻结为相同 learner/reference Cargo.lock（108 registry 包；SHA-256 `20adcb21ee432c82a590fd0da3cbec857adbe572bdad2471698c8e0b07f140db`）；两个平台消费同一锁哈希。未通过验收前不是可发布包。

通过后 cargo test --frozen --all-targets 和 cargo run --frozen --example local_demo。仅本地教学，不对公网开放。真正 SQLite 文件、原子业务及审计事务、有限队列、结构化访问记录和优雅退出；不宣称分布式事务、强一致跨进程或生产级监控。
