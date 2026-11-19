# Book D5 · 多线程 Web Server 参考实验

这是中文 Rust Book 2018 版第 20 章的**可选参考实验**：从标准库 TCP / HTTP 处理出发，比较单线程基线与固定大小线程池，再观察停止接收、清空队列和 join 的关闭过程。

## 前置与学习产物

先理解 `Result`、闭包、线程、消息通道和基本 TCP；可从 [BatchMeter](../../course/studio/batchmeter.md)、[同步 ServiceDesk](../../course/studio/servicedesk.md) 回看自己的实现。刚完成 FirstCard 时请先走 [基础后续指南](../../course/AFTER_FIRSTCARD.md)。

实验使用 Edition 2021、最低 Rust 1.74，零 crates.io 依赖、`publish = false`，独立于根 Workspace 和冻结 RC 范围。它提供完整参考实现，不是新的 TODO 学员工程。

完成个人练习后，应能解释一份 HTTP 响应的字节边界、工作线程怎样接收任务，以及关闭时为何要先等待已入队任务完成。详细讲义在 [D5 Web Server](../../course/BOOK_D5_WEB_SERVER.md)。

## 启动服务并观察三个请求

在 **源码根目录或学员包根目录**（能看到 `projects/`）运行：

```powershell
cargo run --locked --offline --manifest-path projects/book-d5-webserver/webserver-lab/Cargo.toml -- 127.0.0.1:7878 4
```

保留这个终端，打开浏览器：

| 地址 | 预期行为 |
|---|---|
| [首页](http://127.0.0.1:7878/) | 返回实验的 HTML 页面 |
| [慢请求](http://127.0.0.1:7878/sleep) | 等待约 200 ms 后返回响应 |
| [不存在的页面](http://127.0.0.1:7878/missing) | 返回 404 |

在**服务终端**输入 `stop` 并按 Enter。服务停止接收新连接，已排队任务完成，worker 被 join 后程序退出。端口被占用时，换为未占用的本机端口并同步修改浏览器地址。

## 源码怎样对应观察

| 文件 / 接口 | 负责什么 |
|---|---|
| `webserver-lab/src/main.rs` | 地址、worker 数量、stdin 停止命令 |
| `webserver-lab/src/lib.rs` 的解析与路由 | 读取有上限的请求头，生成 HTTP 响应 |
| `serve_sequential` | 有限连接数的单线程教学基线 |
| `ThreadPool`、`serve` | 固定 worker、任务通道、停止与清理 |
| `webserver-lab/tests/cli.rs` 及库内测试 | CLI、真实 TCP、并发上限、队列清理与关闭 |

`serve_sequential` 是库接口，不是额外的 CLI 子命令。比较实验按讲义和测试运行。

## 检查参考实现

先正常停止服务，再在 **同一个根目录**执行：

```powershell
cargo fmt --manifest-path projects/book-d5-webserver/Cargo.toml --all -- --check
cargo clippy --locked --offline --manifest-path projects/book-d5-webserver/Cargo.toml --workspace --all-targets -- -D warnings
cargo test --locked --offline --manifest-path projects/book-d5-webserver/Cargo.toml --workspace --all-targets
cargo test --locked --offline --manifest-path projects/book-d5-webserver/Cargo.toml --workspace --doc
```

这些检查验证参考实现；fmt / clippy 还需要本机相应工具组件。只看网页打开，不能证明线程池和关闭行为正确。

## 一次自己的收尾实验

复制整个实验到新的个人目录。在副本 `webserver-lab/src/lib.rs` 按讲义为线程池增加完成计数测试：提交多项任务后立即离开作用域，断言 Drop 返回时任务全部完成。先预测哪些线程仍在工作，画出入队、执行、停止消息和 join 的顺序。

在 **副本 Workspace 根目录**运行：

```powershell
cargo test --locked --offline --workspace --all-targets
cargo test --locked --offline --workspace --doc
```

保存自己的测试和解释：为什么不能持有同一个互斥锁执行整个慢任务？为什么不能只发关闭信号就认为线程已退出？比较一个与多个 worker 时，记录实际行为，不保证更多线程一定更快。

## 技术边界与下一步

这是教学 HTTP/1.1 子集：只处理 GET、无请求体、无 keep-alive、无 TLS，请求头有上限。不是生产 Web Server，不替代 ServiceDesk 或 Axum / Tokio。

完成同步线程池实验后，回到自己的网络作品核对错误和关闭路径。具备 AsyncBoard 基础时可读 [C1](../../course/ADVANCED_C1.md)，再按路线进入 [C2](../../course/ADVANCED_C2.md)，比较线程阻塞和异步等待的不同；单纯查词与工具则用 [D6 附录](../book-d6-appendices/README.md)。
