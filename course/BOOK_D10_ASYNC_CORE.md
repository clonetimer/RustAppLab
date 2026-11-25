# D10-A · 从 Future 到实际唤醒

[现代化计划](MODERN_RUST_ROADMAP.md) · [新版 Book 覆盖矩阵](BOOK_MODERN_COVERAGE_D7.md) · [独立工程](../projects/modern-d10-async-core/README.md)

在 AsyncBoard 与 D9-A 的 async trait、async closure 之后学习。D9-A 的 ready-only 测试只用于立即完成的 Future；本课提供实际 Pending 与 Waker，使用标准库实现最小单根 executor。不是重复实现网络运行时。Rust 1.100.0 / Edition 2024；工具链安装完即可离线学习。复制 learner 整个目录到个人路径，编辑 src/lib.rs。

## 六个实际任务

| 步骤 | 实现 | 正常/异常合同与迁移 |
|---|---|---|
| 1 | YieldOnce：第一次 Pending，并唤醒自己；第二次返回值 | 手动 Poll 验证唤醒计数；异步体在 poll 前不执行；没有 wake 的 pending root 到期退出。改变值或让出次数。 |
| 2 | join_pair：同时推进两个 Future，保存各自完成结果 | 一侧先 Ready 后不能再 poll；两种不同输出类型；交换先完成的一侧，解释为何串行 await 不等于 join。 |
| 3 | join_many：异构 Future 通过 Pin<Box<dyn Future>> 统一接口 | 保留输入顺序、空数组、借用非静态数据；增加一个不同 async 块和一个慢任务。 |
| 4 | race_pair：先 Ready 的返回，约定左优先，同一作用域 loser 被 drop | 验证 loser 的 Drop 计数；同一轮同时 Ready；修改 tie 策略并解释公平性。 |
| 5 | SignalReceiver：在同一锁中读取值/关闭状态或保存最新 Waker | 真实 OS 线程发送、替换旧 Waker、sender drop、receiver drop 返还值；先 Pending 后外部唤醒，两次 root poll。 |
| 6 | pin_probe：Box::pin 固定 !Unpin 对象，移动句柄不移动对象 | 对比地址与实际 Future 输出；查看 E0277 反例；解释为何不需要 unsafe。 |

每次在个人工程执行 cargo test --locked --offline --test contracts step1_，更换 1–6。初始业务测试故意失败。完成后执行 cargo test --locked --offline --all-targets 及 cargo run --locked --offline --example async_workbench。预期 joined=(7, 9); polls=2，ordered=[8, 3, 5]，race=Right(2)，signal=Some(42)，pinned=31；实际结果以当前 CI/个人运行日志为准。

## 原理与边界

Future 是惰性的状态机。poll 返回 Pending 时必须建立将来可以继续推进的通知条件；Waker 是调度通知，不是返回值，也不意味着立刻同步 poll。提供的 runtime.rs 用容量 1 的同步消息通道合并通知，首先 poll，再等待 wake；通知在等待前发生也能保留。没有网络 reactor、定时器驱动、后台 task、工作窃取或抢占；超时无法中断阻塞的单次 poll。不得将本教学执行器用于公网生产服务。

join 保存已 Ready 的输出，动态 join 保留原始输入顺序，不代表任务完成顺序。这里的所有 futures 在同一 root 被轮询，不是多核并行。race 的取消只表示在作用域内 drop 未完成的 Future；不会回滚已发生的副作用，也不保证已经 spawn 的其他任务被取消。左优先只是显式教学策略，不保证公平。Signal 的状态读取与 Waker 注册在同一锁中，send/drop 取出 Waker 后才 wake，避免丢通知；该单值通道不是多生产者队列。

Pin 不意味着所有内存固定不动，约束的是被 pin 的 pointee；Unpin 类型允许安全移出，而 !Unpin 类型需遵守固定约束。Pin<Box<T>> 的拥有型句柄可以移动。跨线程任务通常要求 Send，运行时 spawn 通常还有 'static 约束；async move 移入捕获，但不自动使 Rc 成为 Send。

## 与新版 Rust Book 的六节对应

| 小节 | D10-A 提供 | D10-B 或既有课程继续承担 |
|---|---|---|
| 17.1 | 惰性、Pending、实际 Waker 与竞速 | C1 真实运行时/网络请求 |
| 17.2 | join、跨线程单值通知、async move 反例 | Tokio spawn、mpsc、多任务间消息 |
| 17.3 | 动态异构集合、join/race、协作式超时 | Tokio 有界并发、select、yield、超时 |
| 17.4 | 尚未提供标准 Stream 专项 | 真实 futures_core::Stream / StreamExt、有限队列 |
| 17.5 | Future、Pin/Unpin、Context、Poll、Waker、Send 诊断 | Stream trait 与异步 Trait 的综合 |
| 17.6 | 线程向 Future 发真实唤醒、协作式等待边界 | Tokio spawn_blocking、CPU 与 I/O 对照 |

D10-A 不是第 17 章全部覆盖完成。已有 C1/C2 不等同于系统化讲解；下一阶段补 D10-B。Book 基线为 [固定 Rust 1.100 发布快照](https://github.com/rust-lang/book/blob/1500248d8f230566e4ec9f27fcbb8fe9e2898ab1/src/ch17-00-async-await.md)。

## 真实编译反例与维护者检查

完整源包 reference/probes 有 4 组 red.rs/green.rs：异构 async 类型、非 Unpin、非 'static 借用与 !Send。不要将 red.rs 放进正式 crate 的 src。运行 rustc --edition=2024 --emit=metadata probes/pin_unpin/red.rs 可观察预期 E0277；脚本逐组核对实际诊断并运行修复版，成功不是由无关编译错误造成。

维护者在完整平台根目录运行 python scripts/modern_d10a_check.py --output d10a-evidence，执行独立 Rust 1.100.0 编译、六步 RED/GREEN、Debug/Release、CLI 及真实失败/修复；只在临时副本注入参考答案，不写回学员代码。学员无需 Python。真人理解、网页入口与完整主干 CI 均另验。

参考：[Future](https://doc.rust-lang.org/std/future/trait.Future.html) · [Waker](https://doc.rust-lang.org/std/task/struct.Waker.html) · [Pin](https://doc.rust-lang.org/std/pin/index.html) · [Tokio](https://tokio.rs/tokio/tutorial)。
