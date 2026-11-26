# 我的 HTTPS 权限实验

优先取得 E1-A-learner-offline.zip，解压到新目录，先安装 Rust 1.100.0 和系统构建工具。此独立包含已学 C2 的固定支持代码、Cargo.lock 与 vendor；**不含 E1-A 参考解答**。若你只有平台中的 learner 模板，请取得完整 E1 包；该模板依赖完整仓库 C2，不能脱离路径直接运行。不要从旧 C1/C2 复制 vendor。

在 src/lib.rs 实现 TODO(1) 到 TODO(4)，依次运行：

    cargo test --frozen --test contracts step1_
    cargo test --frozen --test contracts step2_
    cargo test --frozen --test contracts step3_
    cargo test --frozen --test contracts step4_
    cargo test --frozen --all-targets

最后一项 step5 为真实 HTTPS 与 C2 持久化集成。初始四项应失败。保留测试，不以跳过证书校验或允许匿名写入换取通过。

    cargo run --frozen --example local_demo

预期实际输出 HTTPS verified; missing=401; reader_write=403; writer_create=201; reader_list=200。测试/演示在进程内临时生成证书及随机凭证，不打印密钥，不需要手工创建证书。

进一步体验：cargo run --frozen --example local_cert -- <新的私有目录>，只生成 localhost 教学证书与密钥。先执行 `cargo run --frozen -- init <新工单日志>`（拒绝覆盖已有文件）。服务 CLI cargo run --frozen -- serve <journal> <cert.der> <key.pkcs8.der> <port> 使用环境变量 E1_READ_TOKEN、E1_WRITE_TOKEN（两份不同的安全随机 32 字节十六进制值）；不要硬编码真实凭证或提交密钥。在自己可信的客户端仅对本次请求信任 cert.pem，使用 localhost 作为主机名；切勿使用 -k 或 disable certificate verification。Ctrl+C 优雅退出。真实账号、会话、轮换和撤销另学，不把两枚教学凭证当正式身份系统。

在 LEARNING_LOG.md 记录不同错误、状态码及为什么持久状态不变。完整讲义见 course/BOOK_E1_TLS_AUTH.md；网页接入和真人理解仍待验。
