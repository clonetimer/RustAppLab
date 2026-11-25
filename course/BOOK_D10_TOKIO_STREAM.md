# D10-B · 在真实 Tokio 中处理异步数据流

[现代化路线](MODERN_RUST_ROADMAP.md) · [D10-A 原理课](BOOK_D10_ASYNC_CORE.md) · [D10-B 工程入口](../projects/modern-d10-tokio/README.md)

完成 D10-A 后进入。D10-A 自制教学 executor 只驱动一个 root；本课使用真实 Tokio 1.53.3、futures-util 0.3.34，固定 Rust 1.100.0 和 Cargo.lock。首次在个人副本运行 cargo fetch --locked（需要访问依赖源），之后 cargo test --locked --offline。离线包 D10-B-learner-offline.zip 包含 vendor 和校验清单，工具链已安装时无需 registry 缓存，运行 cargo test --frozen。不要单独改动全局工具链；旧网络课包按 [E1-0 安全迁移说明](NETWORK_SECURITY_BASELINE.md) 获取同提交的新依赖锁与 vendor 后重验。

## 7 步实际学习任务

| 任务 | 要写出的能力 | 观察与迁移 |
|---|---|---|
| 1 join_samples | 两个异步定时器同时推进 | Tokio 暂停虚拟时钟，30/50 ms join 期望只经过 50 ms，串行经过 80 ms；不是 OS 性能基准 |
| 2 start_producer | spawn 持有数据，容量 1–64 的 mpsc 队列 | 容量 1 时生产者等待，关闭接收者后停止，返回实际成功发送数 |
| 3 run_bounded | buffer_unordered 限制在途 Future，再按原始索引恢复顺序 | 不用墙钟猜并发：测试记录真实进入/退出与峰值并发，观察 2 个在途、完成乱序、输出有序 |
| 4 ReceiverStream / merge_positive | 实现真实 futures_util::Stream 的 poll_next，再用 StreamExt 合并与过滤 | Pending、真实 Waker、Some、EOF；源内顺序保留，跨源全局顺序不承诺 |
| 5 race_or_timeout / abort_and_join | select! 处理取消信号、任务结果、定时器，另对 spawned task abort 后 await | 显式取消优先，信号 sender 关闭不等于取消；用 Drop 计数检查资源释放 |
| 6 blocking_work | spawn_blocking 执行同步工作，异步侧可继续运行 | 真实线程 ID、双向通知，无需靠 sleep 时差推断；阻塞 panic 成为 JoinError |
| 7 spawn_read | 返回 impl Future + Send 的静态 Trait 接口，并将源与 key 移入 task | 真实多线程 runtime；正例、缺失值和缺少 Send 约束的编译反例 |

每步只改 src/lib.rs，执行 cargo test --locked --offline --test contracts step1_（替换数字）。初始 7 个合同故意失败；参考答案注入仅由维护者在临时副本执行。第 4、5 步各含两个 TODO。全部通过后 cargo test --locked --offline --all-targets，运行 cargo run --locked --offline --example async_workbench。在个人记录中增加不同容量、输入顺序、超时或取消场景，解释 API 与测试边界。公开合同不能强制指定语法；独立理解另验。

## 原理与工程边界

- Future 可在一个任务中交错推进；join 不必创建新 OS 线程。Tokio spawn 拥有独立 task，通常要求 Future: Send + 'static，async move 转移捕获但不自动产生 Send。
- 有界 mpsc 限制已入队消息数量，不限制整个应用内存；背压是 send().await 等待容量，不是丢弃消息。接收端关闭后 send 返回错误。
- buffer_unordered 在一批同构 Future 类型中保持有限在途任务；异构任务可使用 D10-A 的 Pin<Box<dyn Future>>。恢复原始索引与完成顺序无关。其取消范围仅包括仍由外层 Future 持有的工作。
- Stream 与 Future 不同：poll_next 可以多次返回 Some(item)，Pending 必须安排唤醒，None 表示结束。这里包装真正 Tokio Receiver，使用真实 futures-util StreamExt；不是把 Vec 改名为 Stream。merge 的跨源顺序依调度变化。
- select! 在本例用 biased 声明优先级；没有一般公平保证。drop 未完成 Future 不会撤销已完成副作用；drop JoinHandle 会 detach，abort 后 await 才核验终止。spawn_blocking 已开始的同步任务不能靠 abort 强制停止；需要协议协作，超时不等于撤销。
- RPITIT 显式 Send 约束使泛型调用者能 spawn 返回的 Future；裸 async fn in trait 不自动保证。这个 Trait 不宣称 dyn-compatible；需要动态分发时另用 boxed future 等明确接口。

## 新版 Book 第 17 章对应证据

| 固定章节 | 可定位的课程任务 |
|---|---|
| 17.1 Futures and Async Syntax | D10-A 1/4（惰性、竞速）；D10-B 1/5（真实 Tokio）；C1 HTTP 应用 |
| 17.2 Concurrency with Async | D10-A 2/5；D10-B 1/2/7（spawn、消息、async move） |
| 17.3 Any Number of Futures | D10-A 3/4（异构）；D10-B 3/5（有界、select）；C1 超时取消 |
| 17.4 Streams | D10-B 4（真实 Stream、StreamExt、合并、有限队列）；不是仅出现网络 chunk |
| 17.5 Traits for Async | D10-A 1/5/6（Future/Poll/Waker/Pin）；D10-B 4/7（Stream trait、Send） |
| 17.6 Futures, Tasks, Threads | D10-B 2/5/6/7（task、thread、阻塞池、取消）；C2 单写者 |

这是六节代表性教学目标映射，不宣称逐句穷尽 Book 示例。固定 [Book 源码](https://github.com/rust-lang/book/blob/1500248d8f230566e4ec9f27fcbb8fe9e2898ab1/src/ch17-00-async-await.md)。D10-A 既有双平台 PASS；D10-B 同提交专项仍须 CI。

## 真实反例和验收

完整 source 包的 reference/probes 包含 stream_pin 与 trait_send 的 red/green Rust 源。学员不必把错误程序放入 src。维护者脚本把它们放进临时项目的 example 并调用真实 Cargo 编译，分别核验 E0277 与 future cannot be sent between threads safely，再运行修复程序。另用实际暂停时钟、真实 Waker 和同步通知验证状态，不把一次 ready-only poll 或墙钟速度当作调度证据。

从完整平台根目录运行 python scripts/modern_d10b_check.py --output d10b-evidence。首次 Cargo vendor 必须联网；生成只含 TODO 的离线学员 ZIP、SHA-256、真实空缓存中文路径重放、7 步 RED/GREEN、Debug/Release、CLI、诊断日志。源码快照前后相等才 PASS。未安装编译器和系统构建工具仍需自行准备。不自动发布新 RC，不纳入旧 C1/C2 兼容工作区；网页入口和真人试用待后续。

参考：[Tokio 教程](https://tokio.rs/tokio/tutorial) · [Tokio 1.53.3](https://docs.rs/tokio/1.53.3/tokio/) · [futures-util 0.3.34](https://docs.rs/futures-util/0.3.34/futures_util/) · [Pin](https://doc.rust-lang.org/std/pin/index.html)。
