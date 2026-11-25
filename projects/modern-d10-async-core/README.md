# D10-A · 异步核心教学实验

入口：[D10-A 中文讲义](../../course/BOOK_D10_ASYNC_CORE.md)。

在完成 D8 与基础 Future 经验后学习。把 learner 整体复制到个人目录；固定 Rust 1.100.0 / Edition 2024，无第三方依赖。reference 只进入完整源码包，不交付给无答案学员包。六步任务观察真实 Poll、Waker、组合与取消；src/runtime.rs 是提供的单根教学 executor，不是 Tokio，也没有网络、I/O reactor 或抢占能力。

D10-B 将补真实 Tokio task / channel / Stream / select / spawn_blocking；AsyncBoard、C1、C2 保留为应用载体。当前只采用独立 CLI，尚未接入网页评分。
