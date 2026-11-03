# C2：基于 Axum 的异步 ServiceDesk 工单服务

C1 已学习真正的 Tokio/Reqwest 客户端；C2 对照同步版 ServiceDesk，完成 Axum JSON 路由、Tokio 共享状态、单写者持久化以及优雅关闭。独立工程使用 Rust 1.85.0，原平台 Rust 1.74、C1 与 B4 不变。

## 开始学习

在 RustAppLab 完整平台的根目录中执行：

~~~powershell
rustup toolchain install 1.85.0 --profile minimal
cargo run --locked --offline -p labquest -- advanced-c2 start "my-axum-servicedesk"
cargo run --locked --offline -p labquest -- advanced-c2 prepare "my-axum-servicedesk"
cargo run --locked --offline -p labquest -- advanced-c2 lesson "my-axum-servicedesk"
cargo run --locked --offline -p labquest -- advanced-c2 check "my-axum-servicedesk" 1
cargo run --locked --offline -p labquest -- advanced-c2 next "my-axum-servicedesk"
~~~

start 仅创建新目录，不覆盖已存在文件。prepare 首次显式联网获取锁定依赖，只编译固定参考程序，不运行学员 Cargo.toml、build.rs、测试或源码。之后 check 完全 locked/offline；next 与 lesson 只读且不执行学员程序。成绩记录与实际代码绑定，源代码变动后旧报告过期。

## 先运行参考服务

~~~powershell
cargo +1.85.0 fetch --locked --manifest-path projects/advanced-c2/reference/Cargo.toml
cargo +1.85.0 run --locked --offline --manifest-path projects/advanced-c2/reference/Cargo.toml -- init "tickets.db"
cargo +1.85.0 run --locked --offline --manifest-path projects/advanced-c2/reference/Cargo.toml -- serve "tickets.db" --port 3000
~~~

打开 http://127.0.0.1:3000/ 即可新建、查看、关闭工单。每次修改会先写入与旧 ServiceDesk 兼容的日志。init 只能创建新日志文件；第二次 serve 复用此日志，不要重复 init。Ctrl+C 触发等待正在处理请求完成的关闭流程。

## 三个累计检查点

| 阶段 | 由学员实现 | 期望行为 |
|---|---|---|
| C2-1 | start_store | 使用 servicedesk::load 验证日志；Tokio mpsc 队列容量 64；spawn_blocking 执行唯一持久化写者；通过 oneshot 返回结果，磁盘失败后拒绝陈旧数据 |
| C2-2 | router | 注册 GET /health、GET /、GET /assets/client.js、GET /assets/app.css、GET/POST /api/tickets、POST /api/tickets/{id}/close；配置 2048 字节 JSON 请求正文限制 |
| C2-3 | serve | axum::serve 真正绑定给定 TCP listener；watch true 后执行 graceful_shutdown 并等待活动请求完成，不能因 false sender 关闭而忙循环 |

学员工作区的 src/model.rs、src/handlers.rs、src/main.rs 是固定基础设施；业务域规则沿用原 servicedesk crate，核心练习集中在 src/lib.rs，也可以自行拆分模块。可见测试命令：

~~~powershell
cargo +1.85.0 test --locked --offline --test contracts c2_1_
cargo +1.85.0 test --locked --offline --test contracts c2_2_
cargo +1.85.0 test --locked --offline --test contracts c2_3_
~~~

初始 TODO 可编译但业务测试会失败，属于预期学习起点。平台评分使用固定锁、合同测试、业务库、handler 和 CLI，只接收学员实现代码，不以学员自行修改的测试替换合同。

## API 速查

GET /health 返回 JSON 的 ok=true。GET /api/tickets 返回 tickets 数组及 count。POST /api/tickets 请求 JSON 的 title，成功时 201 返回 id、title、done=false。POST /api/tickets/{id}/close 成功时 200 返回 done=true。写请求必须带 Content-Type: application/json 与 X-RustAppLab-Client: 1；这是本机浏览器跨源写防护，不是用户身份认证。

非法标题422、缺少写请求标记403、过大JSON正文413、编号不存在404、重复关闭409、存储错误503。标题最多 120 Unicode 字符且不可含控制字符；最多1000个工单、日志上限2MiB。若存储损坏，明确拒绝启动，不自动修复或重置。

## 三级提示

**提示 1：画出事件流。** HTTP 客户端 → Axum Router → Handler → AppState → mpsc Sender → 单写者任务 → 旧日志 → oneshot 响应。哪一层拥有磁盘句柄？两个同时 POST 如何保证编号唯一？

**提示 2：缩小操作。** 从既有 servicedesk::load 初始化内存状态；在 spawn_blocking 中用 blocking_recv 读取命令，单线程调用旧 persist，成功后更新快照。Router 通过 with_state(state) 共享状态，get/post 注册路由，DefaultBodyLimit::max(2048) 约束请求正文。

**提示 3：收尾与失败。** 写盘失败后不要继续返回内存旧状态或假成功。用 axum::serve(...).with_graceful_shutdown(...) 监听 watch true；发送端失联但 false 时应保持等待。用多线程 Tokio 模拟48个并发客户端，重启后再校验日志，证明没有丢失更新。

## 离线包与边界

交付 C2-reference-offline.zip 与 C2-learner-offline.zip。两份分别含参考代码或三个 TODO，含 Cargo.lock、vendor、隐藏 .cargo/config.toml、旧业务库副本、网页资源、合同测试和讲义；不含 Rust 编译器与 MSVC 工具链。解压后在工程根目录运行 cargo +1.85.0 test --frozen --all-targets。完整平台可以用 RUSTAPPLAB_C2_VENDOR 环境变量指向解压的 vendor，为学员源码做固定离线评分。学员包带工程标记，解压后直接接入平台，不必再次 start。

本阶段仅支持 127.0.0.1 HTTP，不提供 TLS、公网、账号认证、多进程同时写入、强杀或断电原子性。GitHub Windows Server 验证不替代本机 Windows 10 验收。数据保存成功后，即使调用方取消 HTTP future，也不保证撤销已发生的持久化副作用。

参考：[Axum 0.8.8](https://docs.rs/axum/0.8.8/axum/)；[Tokio Channels](https://tokio.rs/tokio/tutorial/channels)；[Serde](https://serde.rs/)；[Cargo vendor](https://doc.rust-lang.org/cargo/commands/cargo-vendor.html)。
