# D5 · 多线程 Web Server

**对照原书**：[rust.bootcss.com 第 20 章](https://rust.bootcss.com/ch20-00-final-project-a-web-server.html)，对应当前英文版 Rust Book 的 [第 21 章 Final Project: Building a Multithreaded Web Server](https://doc.rust-lang.org/book/ch21-00-final-project-a-web-server.html)。本课覆盖原书 20.1–20.3 三节。

配套 [独立可运行 Workspace](../projects/book-d5-threaded-web-server/README.md)，Edition 2021、Rust 1.74、零第三方依赖、publish = false，不修改根工作区 17 个包。

## 学习目标

完成本课后，你应能：

- 使用 TcpListener 接收连接，读取 HTTP 请求行并构造响应。
- 解释单线程服务器为什么会被慢请求阻塞。
- 使用 Job、mpsc、Arc、Mutex 和 Worker 构建固定大小线程池。
- 解释 Receiver 为什么需要被 Mutex 包装，以及为什么锁必须在执行 Job 前释放。
- 使用消息枚举区分 NewJob 与 Terminate。
- 通过 Drop 发送终止消息并 join 所有线程，保证线程池退出前完成已接收任务。

## 0 · 运行实验

在 RustAppLab 源码根目录执行：

~~~powershell
cargo fmt --manifest-path projects/book-d5-threaded-web-server/Cargo.toml --all -- --check
cargo clippy --locked --offline --manifest-path projects/book-d5-threaded-web-server/Cargo.toml --workspace --all-targets -- -D warnings
cargo test --locked --offline --manifest-path projects/book-d5-threaded-web-server/Cargo.toml --workspace --all-targets
cargo test --locked --offline --manifest-path projects/book-d5-threaded-web-server/Cargo.toml --workspace --doc
~~~

运行单线程服务器（请求数为 1）：

~~~powershell
cargo run --locked --offline --manifest-path projects/book-d5-threaded-web-server/web-server/Cargo.toml -- single 7878 1
~~~

另开终端执行：

~~~powershell
curl.exe -i http://127.0.0.1:7878/
~~~

运行线程池服务器（4 个 Worker，接受 2 个连接后有序关闭）：

~~~powershell
cargo run --locked --offline --manifest-path projects/book-d5-threaded-web-server/web-server/Cargo.toml -- pool 7878 4 2
~~~

用两个终端分别访问 / 与 /sleep。/sleep 会阻塞当前处理线程 150 ms；线程池允许另一个 Worker 同时处理其他连接。服务器仅绑定 127.0.0.1，不能从外部网络访问。

## 20.1 · 单线程 Web Server

serve_single 使用 TcpListener.incoming() 顺序接收连接，每个连接都在当前线程完成请求解析和响应。示例只实现 GET、HTTP/1.0/1.1、根路径、/sleep 和 404 路由；请求行限制为 1024 字节，读取超时 2 秒，响应显式携带 Content-Length 与 Connection: close。

**观察实验**：

1. 先发送 /sleep，再发送 /。
2. 比较两个请求的完成顺序。
3. 解释为什么同一线程必须先完成慢请求，才能处理后一个连接。

这不是生产 HTTP 服务器：不实现请求体、持久连接、chunked 编码、TLS、虚拟主机或公网访问。这里的重点是 TCP 连接、请求行、响应格式与阻塞模型。

## 20.2 · 多线程 Web Server

ThreadPool 固定创建 N 个 Worker。execute 将 Box<dyn FnOnce() + Send + 'static> 作为 Job 发送到 mpsc 队列；每个 Worker 从共享 Receiver 取出一个 Message，再执行 Job。

std::sync::mpsc::Receiver 不能直接跨线程共享，因此用 Arc<Mutex<Receiver<Message>>> 管理唯一接收端。锁只用于取消息，拿到 Job 后立即释放；如果在执行整个 Job 期间持锁，其他 Worker 就无法同时取任务，线程池会退化成串行执行。

**观察实验**：

- 查看 thread_pool.rs 中 Job 类型别名、Message 枚举和 Worker 循环。
- 运行 two_workers_can_start_jobs_before_either_is_released，理解并发不是依靠 sleep 时间估算，而是两个 Worker 都进入任务后再统一释放。
- 将线程池大小改成 1，观察并发测试为什么不能满足双 Worker 前置条件。

**练习**：新增一个 /status 路由，返回固定状态文本；为 GET、未知路径和不支持的方法分别增加响应测试。

## 20.3 · 优雅关闭

线程池析构时执行两个步骤：

1. 向消息队列发送与 Worker 数量相同的 Terminate。
2. 逐个调用 JoinHandle::join，等待正在执行的任务结束并回收线程。

由于任务消息先于终止消息入队，Worker 会先处理已接收的任务，再读到 Terminate 并退出。Drop 不会在 Worker 仍执行任务时提前返回。测试 drop_drains_queued_jobs_and_joins_workers 验证队列中的任务全部完成；HTTP 集成测试则在处理完指定连接后返回并完成线程池关闭。

本实验用正整数请求数限制服务器运行时间，确保可以观察正常退出路径。强制终止进程不等同于优雅关闭。

**练习**：

- 修改线程池大小和请求数量，确认所有请求均有响应。
- 暂停一个 Job，观察 drop 必须等待它完成。
- 尝试让 Worker 收到 Terminate 后立即退出，再观察尚未处理的任务会发生什么；不要把任务丢弃误称为优雅关闭。

## 自动验收与边界

| 原书内容 | 可复现证据 |
|---|---|
| 20.1 单线程服务器 | 本地 TCP 请求、200/400/404/405 响应 |
| 20.2 多线程服务器 | 双 Worker 并发启动、共享消息接收端、真实 HTTP 双连接 |
| 20.3 优雅关闭 | 队列任务排空、Terminate、join、有限请求后的服务器返回 |
| 安全与兼容 | 127.0.0.1 绑定、请求行大小上限、读取超时、Rust 1.74、零第三方依赖 |

测试证明参考实现满足这些行为，不代表学员已经掌握。完成学习还需要自行增加路由、修改线程池实现并补充测试。

参考资料：
- [The Single-Threaded Web Server](https://doc.rust-lang.org/book/ch21-01-single-threaded.html)
- [Turning Our Single-Threaded Server into a Multithreaded Server](https://doc.rust-lang.org/book/ch21-02-multithreaded.html)
- [Graceful Shutdown and Cleanup](https://doc.rust-lang.org/book/ch21-03-graceful-shutdown-and-cleanup.html)
