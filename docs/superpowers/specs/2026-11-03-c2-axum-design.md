# C2 设计：ServiceDesk 的 Axum 异步工单服务

主干基线：C1 已合并的 \`9371c87df40f9d84ac17fb33b12577a4eb4b66d3\`。本阶段只推进 C2，不改变同步 ServiceDesk、C1、B4 的既有 ABI 与根工作区 Rust 1.74 边界。

## 用户意图与设计取舍

按现有课程路线，本阶段要让学习者真正使用 Tokio/Axum 完成工单服务的路由、JSON、共享状态、持久化、优雅关闭和并发客户端验证。两个实现选项：(1) 改写既有同步服务端，迁移成本低但回归风险高；(2) 独立工作区复用既有 servicedesk 业务逻辑和日志格式，迁移代价稍高但对照学习清楚、保持旧课程稳定。采用选项(2)，因为用户持续要求逐段推进与保留既有功能。

## 边界

C2 使用独立 Rust 1.85 / Edition 2021 工作区：Axum 0.8.8、Tokio 1.48.0、Serde/serde_json 与 Reqwest 测试客户端。既有 servicedesk（Rust1.74、零外部依赖）作为 path 依赖复用；离线包复制该固定业务库至 support 目录，避免安装后引用原仓库绝对路径。

服务只监听字面 IP \`127.0.0.1\`；HTTP/1.1，不启用公网、TLS、认证账号、CORS、WebSocket 或多进程共同写入。Web 表单写操作要求自定义 \`X-RustAppLab-Client: 1\` 和 \`application/json\`，利用浏览器跨域预检阻止普通跨域写入；它不是通用鉴权系统。JSON 请求正文硬限 2048 字节；单标题 1~120 个 Unicode 字符（复用既有规范）；工单上限1000条，日志上限2MiB。输入错误与不存在/冲突/存储不可用映射为确定的 HTTP 状态码，不回显系统私密路径。输出 JSON，由浏览器用 textContent 显示。

## 接口

\`GET /\` 为 HTML 页面，\`GET /assets/client.js\` 为独立脚本，\`GET /health\` 返回 JSON。
\`GET /api/tickets\` -> \`{"tickets":[{"id":1,"title":"...","done":false}],"count":1}\`；
\`POST /api/tickets\` -> body \`{"title":"..."}\`，201 + 工单对象；
\`POST /api/tickets/{id}/close\` -> 200 + 已关闭工单。缺失自定义头=403；重复关闭=409；不存在=404；非法标题=422；正文超限=413；日志错误=503。未知路径返回404，静态资源不会被映射为数据 API。

CLI \`netdesk-c2 init <new-log-file>\` 创建旧格式数据库并拒绝覆盖；\`netdesk-c2 serve <existing-log-file> [--port 3000]\` 绑定回环 IP，Ctrl+C 发出布尔 watch 取消并让服务有界地完成正在处理的请求，退出时停止接收新连接。调用方未来内的正常异步关闭受支持；不承诺断电持久化、多进程写并发或强行结束进程时的远端语义。

## 并发与数据一致性

每个 \`AppState\` 是有界 \`tokio::sync::mpsc\` Sender。独立阻塞线程持有单份内存快照和唯一磁盘写者，通过 \`blocking_recv\` 处理 List/Create/Close；网络 handler 通过 oneshot 等待回应，不在 Tokio 核心线程执行同步磁盘 I/O。写入先用旧 servicedesk::persist() 同步日志成功，再更新内存快照并回复；因此同一运行中并发新增不会丢失更新。日志读取遇坏记录显式拒绝启动，不能悄悄修复或清空。未读完的工作在请求取消后仍可能完成持久化，API 不承诺取消回滚。

## 教学、分发与验收

三步学员函数：\`start_store\`（单写者状态与持久化）、\`router\`（Axum 路由与状态）、\`serve\`（真正 TCP HTTP + graceful shutdown）。学员可修改 lib.rs，也可拆分自己的模块；固定 model/handler/CLI 作为教学基础设施。参考与学员分别打包，真实 Cargo 生成 lock，不编造哈希。与 C1 一致：固定测试、三阶段累计报告、源码绑定成绩、准备依赖与评分分离；支持 Windows/Linux 空缓存与搬移目录验证。根工作区的基础 CI、真实浏览器、Rust1.74、C1 仍为准入门槛。只有全部门禁成功才合并和打 checkpoint；不宣称 Stable 或本机 Windows10 验收。

资料：Axum 0.8.8官方文档 https://docs.rs/axum/0.8.8/axum/；Tokio Join/Channels https://tokio.rs/tokio/tutorial/channels；Serde https://serde.rs/；Cargo vendor https://doc.rust-lang.org/cargo/commands/cargo-vendor.html。