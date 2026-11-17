# D5 · 从 TCP 到多线程 Web Server

**对照原书**：[rust.bootcss.com 第 20 章：构建多线程 Web Server](https://rust.bootcss.com/ch20-00-final-project-a-web-server.html)，覆盖 20.1–20.3。配套 [独立参考 Workspace](../projects/book-d5-webserver/README.md)，不加入根 Workspace，不改变冻结 RC 的 17 个学员包。

## 学习目标

完成本课后，你应能：

- 区分 TCP 连接与 HTTP 请求/响应，解释请求行、头部、状态行、响应头和 body 的边界。
- 使用 TcpListener / TcpStream 接收连接、读取有限的请求头并返回正确的 HTTP 响应。
- 解释为什么固定大小线程池比“每个连接都 spawn 一个线程”更容易控制资源。
- 使用 mpsc、Arc<Mutex<Receiver<_>>>、FnOnce + Send + 'static 将任务交给 Worker。
- 通过 Drop 发送终止消息并 join 所有线程，理解关闭顺序与死锁风险。

## 0 · 运行真实实验

在 RustAppLab 源码根目录运行：

~~~powershell
cargo fmt --manifest-path projects/book-d5-webserver/Cargo.toml --all -- --check
cargo clippy --locked --offline --manifest-path projects/book-d5-webserver/Cargo.toml --workspace --all-targets -- -D warnings
cargo test --locked --offline --manifest-path projects/book-d5-webserver/Cargo.toml --workspace --all-targets
cargo test --locked --offline --manifest-path projects/book-d5-webserver/Cargo.toml --workspace --doc
cargo run --locked --offline --manifest-path projects/book-d5-webserver/webserver-lab/Cargo.toml -- 127.0.0.1:7878 4
~~~

打开浏览器访问：

- http://127.0.0.1:7878/：返回项目中的 HTML 页面。
- http://127.0.0.1:7878/sleep：故意等待 200 ms，用于观察慢请求。
- http://127.0.0.1:7878/missing：返回 404。

在运行服务的终端输入 stop 并回车。服务器停止接收新连接，线程池先完成队列中的工作，再退出 Worker 并 join。默认仅监听本机回环地址，不暴露到局域网。

## 20.1 · 单线程 Web Server：先理解字节流

TCP 是可靠的字节流传输，不会替应用划分 HTTP 消息。HTTP 定义了请求行、头部和消息体的格式。这个实验只实现必要子集：

~~~text
GET / HTTP/1.1\r\n
Host: localhost\r\n
\r\n
~~~

请求行由 method、request-target 和 HTTP version 组成；头部每行以 CRLF 分隔，空行标记头部结束。响应使用状态行、Content-Type、Content-Length、Connection 和 body。Content-Length 按字节数计算，不是字符数。

参考工程的 parse_request 和 read_request 分离了纯解析与 Socket I/O：

- 总头部上限 16 KiB，单行上限 4 KiB，最多 64 个头部字段。
- 只接受 HTTP/1.1 的 origin-form 请求目标；非法请求返回 400。
- 非空请求体与 Transfer-Encoding 被明确拒绝，不会假装支持 POST body。
- Socket 设置读取超时，避免客户端连接后永远不发送完整头部。
- / 返回 HTML，/sleep 提供一个有意延迟的处理路径，其他路径返回 404，非 GET 返回 405。

先运行 serve_sequential 的测试，理解一个连接完成前，单线程不会处理下一个连接。它接受有限数量连接，是教学基线，不是默认服务器入口。

**练习 1**：增加 /health 路由，返回纯文本 ok。分别测试正确请求、未知路径和非 GET 方法，确保状态码与响应体一致。

## 20.2 · 多线程 Server：固定大小 Worker 与消息队列

直接对每个连接调用 thread::spawn，会让线程数量随请求数增长。D5 使用固定大小线程池：

~~~text
TcpListener
    |
    v
accept loop -----> mpsc::Sender<Message>
                         |
                         v
                 shared Receiver
                  Arc<Mutex<_>>
                    /   |   \\
                 Worker Worker Worker
                    |     |     |
                   Job   Job   Job
~~~

关键类型关系：

- Job = Box<dyn FnOnce() + Send + 'static>：一个只能执行一次、可跨线程移动、且不借用当前栈帧的任务。
- Message::NewJob(Job)：把连接处理闭包放入队列。
- Message::Shutdown：通知 Worker 退出。
- Arc<Mutex<Receiver<Message>>>：标准库 MPSC Receiver 不能直接 Clone；Arc 共享接收端，Mutex 保证同一时刻只有一个 Worker 从队列取消息。
- Worker 取到 Job 后会先释放 Receiver 锁，再执行任务；慢请求不会在执行期间占住队列锁。

线程池拒绝 0 个 Worker。它不承诺生产级背压、任务取消或 panic 隔离；如果需要这些能力，应使用经过验证的运行时或线程池库，而不是继续堆叠教学代码。

**练习 2**：把 Worker 数从 1 改为 2、4，分别同时访问 /sleep 与 /。记录响应顺序与耗时；解释为什么不能仅凭一次运行的先后顺序判断并发是否正确。再新增一个测试，验证任务最大并发数不会超过 Worker 数。

## 20.3 · 优雅停机与清理：先发终止消息，再 join

线程池的 Drop 实现遵循两个阶段：

1. 向队列发送与 Worker 数量相同的 Shutdown 消息。
2. 再逐个 join Worker 线程。

不能在同一个循环里“发送一个 Shutdown，然后立刻 join 对应线程”。队列中的终止消息可能被另一个 Worker 取走；当前 join 就可能无限等待。先发送全部终止消息，再 join，才避免这种死锁结构。

服务器本身通过 shutdown channel 接收停止信号。退出 accept loop 后，ThreadPool 离开作用域；FIFO 队列中排在 Shutdown 之前的 Job 会先执行，Worker 随后退出。这里的关闭保证的是已接收任务的排空和线程回收，不是 HTTP keep-alive 会话管理，也不是操作系统信号处理框架。

**练习 3**：为 ThreadPool 增加一个完成计数器。提交多项任务后立即离开作用域，验证 Drop 返回时所有已提交任务均已完成。再画出“Job 入队 → Worker 取出 → Job 完成 → Shutdown → join”的时序图。

## 自动验收与边界

| 验收项 | 参考证据 |
|---|---|
| 请求解析 | 请求行、HTTP 版本、头部、编码、大小上限与 body 拒绝测试 |
| HTTP 响应 | 200/400/404/405、Content-Length 字节数与真实 TCP 交换 |
| 单线程基线 | 有限连接数的顺序服务测试 |
| 线程池 | 0-worker 拒绝、任务执行、固定并发与 Drop 排空测试 |
| 关闭顺序 | shutdown channel、先发送全部终止消息、再 join 的实现与集成测试 |
| CLI | 帮助、参数错误和退出码测试 |

本工程只支持教学所需的 HTTP/1.1 子集：没有 TLS、keep-alive、请求体、chunked encoding、路由参数、完整 RFC 校验或生产级资源限流。真实产品应使用成熟 HTTP 栈；本课的价值是看清它们之下的 TCP、任务调度和资源回收原理。

官方对照：[20.1 单线程 Web Server](https://rust.bootcss.com/ch20-01-single-threaded.html) · [20.2 多线程 Web Server](https://rust.bootcss.com/ch20-02-multithreaded.html) · [20.3 优雅停机与清理](https://rust.bootcss.com/ch20-03-graceful-shutdown-and-cleanup.html)。这些是旧中文 Rust Book 的章节编号；当前官方书籍可能调整编号，应按主题核对。
