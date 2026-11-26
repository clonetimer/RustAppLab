# 我的 SQLite HTTPS 工程

安装 Rust 1.100.0 及系统 C/C++ 构建工具，取得通过当前 CI 的 E1-B-learner-offline.zip 后解压到新目录。Bundled SQLite 随 vendor 构建，不需要用户提前安装数据库服务器。平台中的模板依赖完整仓库的 E1-A/C2，不能直接脱离仓库使用。

依次只改 src/lib.rs 中 TODO(1)–TODO(4)：

    cargo test --frozen --test contracts step1_
    cargo test --frozen --test contracts step2_
    cargo test --frozen --test contracts step3_
    cargo test --frozen --test contracts step4_
    cargo test --frozen --all-targets
    cargo run --frozen --example local_demo

第 1 步实现批量事务（包含 audit），第 2 步关闭与冲突处理，第 3 步队列满与关闭错误，第 4 步固定字段 JSON 访问记录与环形容量。第 5 合同真实 HTTPS、授权、并发写、关闭后重新打开 SQLite。初始四个合同应失败，不能修改固定测试获得通过。

演示预期 HTTPS+SQLite verified; denied=403; created=201; closed=200; reopened=1; audit=2，随后输出真实 JSON Lines；不显示密钥、标题或原 URL。实际结果须自己运行确认。

手动体验：cargo run --frozen -- init <新SQLite路径>；生成私有 localhost 证书 cargo run --frozen --example local_cert -- <新私有目录>；设置两枚不同随机 64-hex E1_READ_TOKEN 和 E1_WRITE_TOKEN，cargo run --frozen -- serve <db> <cert.der> <key.pkcs8.der> <port>。仅在当前客户端显式信任 cert.pem，不关闭 TLS 校验；Ctrl+C 停止并等待已接收的 DB 工作。不要粘贴真实密钥或数据到日志。

记录 LEARNING_LOG.md：改变批量内容、测试中途失败与恢复、改变队列容量，解释为何失败事务与被拒绝请求都不能改变持久化记录；客户端断开不等于已接收工作回滚。讲义见 course/BOOK_E1_SQLITE.md。
