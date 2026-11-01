# ServiceDesk · 本地工单网页

**本课的中心思想**：把一个 Rust 业务状态机连接到本机 HTTP API 和浏览器。用户创建工单，程序保存操作，刷新后数据仍在。学习“业务数据、持久化、协议、界面”的分层，而不是把所有逻辑堆进一个服务器函数。

## 先运行参考软件看目标

适合已经完成 LedgerLite、TermGarden、MiniKV，并学过 NetFetch 的学员。第一次学习 Rust 不必从本课开始。

在完整 Rust AppLab 根目录：

```powershell
cargo run --offline -p servicedesk -- demo my-service-demo
cargo run --offline -p servicedesk -- serve my-service-demo/tickets.log 8766
```

保持终端运行，在本机浏览器打开 `http://127.0.0.1:8766/`。新增一个“修复第一个网络 Bug”工单，再关闭它，刷新网页，确认状态保留。用 Ctrl+C 停止服务。`tickets.log` 是追加式数据日志，`tickets.json` 是初始演示快照而不是实时数据库。

这里实际实现的是**标准库同步串行 HTTP 服务器**，只绑定 127.0.0.1，没有 HTTPS、认证、多人并发控制、数据库迁移、后台任务或公网部署能力；不能把它冒充 Axum 或生产 Web 服务。安全保护包括限定 Host、Origin、写入专用请求头、限制请求长度、对页面内容用 textContent 渲染。它们降低本地误用风险，不是完整安全审计。

## 从一行业务判断开始，而不是先学 Web 框架

在完整项目根目录：

```powershell
cargo run --offline -p labquest -- studio start servicedesk my-servicedesk
```

浏览器打开学员目录的 `START.html`。编辑 `src/lib.rs` 中的 TODO，`src/http.rs` 和 `samples/index.html` 已提供。只有完成当前步骤再往后走。

### 第 1 步：验证工单标题（纯函数）

实现 `validate_title(text: &str) -> Result<String, String>`：去掉外层普通空格；标题 1–120 个字符；拒绝控制字符，包括换行和制表符。为什么用 `chars().count()` 而不是 `len()`？中文在 UTF-8 中可能占多个字节。为什么错误使用 `Result` 而不是 `unwrap()`？因为用户输错是正常业务场景。

```powershell
cargo test --offline --test milestones step1_
```

### 第 2 步：添加、关闭与防止状态错误

实现 `apply(tickets: &mut Vec<Ticket>, action: Action)`。`Action::Add` 生成新编号、存入未完成工单；`Action::Close(id)` 修改已有工单；不存在或已经关闭的编号应返回 `Err`，数据不应发生错误变化。

对照 `&mut Vec<Ticket>` 与 `&[Ticket]`，解释为什么写入函数需要可变借用，而输出报告只需只读借用。持久化 `persist()` 已提供，但它先调用你的 `apply()`：业务逻辑错误也会导致日志错误。

```powershell
cargo test --offline --test milestones step2_
```

### 第 3 步：面向浏览器的 JSON

实现 `render_json(tickets: &[Ticket]) -> String`，产生形如 `{"tickets":[{"id":1,"title":"我的任务","done":false}],"count":1}` 的 JSON。注意引号、反斜杠、中文、`<img>`、`&` 等特殊字符；浏览器不应该把用户输入解释为 HTML。已经给出 `json_string()` 辅助函数；学员重点是把有类型的工单组合成正确的 JSON。生产环境应使用 serde_json 等成熟库。

完整的 HTTP 处理外壳在 `src/http.rs`，学员不必首先自行编写解析器。`GET /` 是网页，`GET /api/tickets` 返回 JSON，`POST /api/tickets` 创建，`POST /api/close` 关闭。浏览器的 `fetch` 发送 URL 编码表单。服务使用单进程追加日志；损坏记录会让后续操作拒绝写入，而非悄悄丢失数据。

```powershell
cargo test --offline --test milestones step3_
cargo test --offline
cargo run --offline -- demo my-own-service
cargo run --offline -- serve my-own-service/tickets.log 8766
```

### 做一个属于自己的工单系统

至少实际创建 3 张工单，关闭其中 1 张；刷新页面并重新启动服务，确认数据仍在。更改一份本地**副本**的日志末尾制造不完整记录，观察读取失败，避免损坏真实学习数据。用文字解释哪些代码是领域状态，哪些代码是 HTTP 协议，哪些代码是持久化。还需要记录你输入过的至少一个无效标题，以及程序如何处理它。

本课程只证明同步 HTTP 基础。下一阶段等待依赖包实际到位后，再用 Tokio + Axum 改造，学习 async/await、任务取消、超时、连接池与多请求并发。
