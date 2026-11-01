# NetFetch · 本机 HTTP 下载器

**本课的中心思想**：先让一段数据从真实 TCP 服务到达 Rust 程序，再决定它是不是成功的 HTTP 响应，最后在不覆盖旧文件的前提下保存。这是网络客户端的基础层，不是已经实现的异步 Tokio 下载队列。

## 适合什么时候学？

已经完成 TextScope、BatchMeter，理解 `Result`、借用、文件写入和基本线程知识之后开始。零基础仍请按 UnitLab → QuizDeck 顺序，不要跳过前面的课程。

## 先看完成后的软件是什么

在完整项目根目录执行：

```powershell
cargo run --offline -p netfetch -- demo my-first-download
```

会创建 `my-first-download/downloads.tsv`、`files/intro.txt`、`files/notes.txt` 和 `README.md`。打开两个 txt，你看到的是程序通过本机 HTTP 服务真正下载的内容，并非预先复制的样例。演示服务器仅监听 `127.0.0.1`，用完自动退出，不需要公网、账号或证书。请使用不存在的目录，避免覆盖旧作品。

你也可以让本机其他 HTTP 服务生成页面后，用 NetFetch 单独抓取。例如先运行 `cargo run --offline -p localhub -- demo localhub-playground`，再启动它的服务器，然后另一个终端运行：

```powershell
cargo run --offline -p netfetch -- fetch http://127.0.0.1:8765/api/health saved-health.json
```

这是命令行联网的实际用途：保存本机工具的状态快照。严格限制：本课程的 NetFetch 仅支持回环 HTTP/1.1，读取最大 2 MiB，不支持公网 HTTPS、chunked、重定向或断点续传。错误要给出明确原因；**不能把它当作通用互联网下载器**。

## 建立自己的工程

在完整平台根目录运行：

```powershell
cargo run --offline -p labquest -- studio start netfetch my-netfetch
```

打开 `my-netfetch/START.html`，编辑 `my-netfetch/src/lib.rs`，先只找 TODO(1)。`src/main.rs` 是已有 CLI 外壳，暂时不用改。你也可以在学员包的 `Studio/netfetch` 目录独立练习。

### 第 1 步：解析 URL，不连接网络

目标函数：`parse_url(text: &str) -> Result<LocalUrl, String>`。从 `http://localhost:8080/hello?x=1` 拆分为主机、端口和路径；不写 TCP、不写磁盘。合法主机只限 `127.0.0.1`、`localhost`、`[::1]`。拒绝 `https://`、域名、0/溢出端口、换行、反斜杠和未编码的空格。理解 `strip_prefix`、`split_once`/`find`、`Option` 与 `Result`。

先预测：`http://localhost` 的路径是什么？答 `/`。错误输入应返回 `Err`，不应该 `panic!`。

```powershell
cargo test --offline --test milestones step1_
```

完成后，在完整平台根目录执行 `cargo run --offline -p labquest -- studio check my-netfetch 1`。第 2、3 步尚未实现不妨碍本步通过。

### 第 2 步：把原始字节解释为 HTTP 响应

目标函数：`parse_response(bytes: &[u8]) -> Result<Vec<u8>, String>`。认识 `HTTP/1.1 200 OK`、`\r\n`、`Content-Length`、头部与正文分界符 `\r\n\r\n`。**Content-Length 是字节数，不是 Unicode 字符数。** 实际收 5 字节但声明 12，必须拒绝，不能保存截断文件。重复长度头、chunked 和 302 重定向本版均应报错。

不要自己发明复杂的全功能 HTTP 解析器；此处是边界清晰的教学用子集，生产场景应该使用经维护的 HTTP 客户端库。

```powershell
cargo test --offline --test milestones step2_
```

### 第 3 步：真实 socket → 正确响应 → 新文件

目标函数：`download(url: &str, output: &Path) -> Result<usize, String>`。先验证路径的父目录，再发出请求；对不存在的输出文件调用 `OpenOptions::create_new(true)`。它比“先检查 exists，然后普通 create”更能避免覆盖竞态。写入后同步到文件，任何写入错误都不得把不完整文件当作成功。网络超时、非 200 响应、超过 2 MiB、错误 URL 时不应产生输出文件。

`request()` 和 `batch()` 的基础设施已经提供，学员把下载和可靠保存接起来。批量工具在暂存目录下载全部成功后才发布正式目录；如果一个任务失败，不发布半成品。但它**不保证断电后的目录持久性**。

```powershell
cargo test --offline --test milestones step3_
cargo test --offline
cargo run --offline -- demo my-real-download
```

### 自己动手做网络排障

先故意填写 `https://example.com`，记录明确的错误；再填写本机 HTTP 地址。修改已存在的目标文件名，观察 `create_new` 拒绝覆盖。在另一份程序中比较 `Content-Length` 与 UTF-8 中文字符数。完成后把运行命令、真实下载文件、失败现象、学习到的知识记录到 `LEARNING_LOG.md`。

下一步请学 **ServiceDesk**：前者扮演 HTTP 客户端，后者扮演 HTTP 服务端。等取得 Tokio/Reqwest/Axum 的实际依赖后，再对比同步与 async/await、并发任务取消和背压；本课不能被记成“异步网络已完成”。
