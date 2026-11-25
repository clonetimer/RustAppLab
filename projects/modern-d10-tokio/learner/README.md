# 我的 Tokio 异步工作台

先安装 Rust 1.100.0，再将整个项目复制到新目录，不修改原模板。进入个人目录执行 rustc --version。普通源码包首次运行 cargo fetch --locked；之后可离线。若从 D10-B-learner-offline.zip 解压，已有 vendor 和 .cargo/config.toml，直接 cargo test --frozen。

在 src/lib.rs 完成 7 步（9 个 TODO），每次运行 cargo test --locked --offline --test contracts step1_，依次改成 step2_ 至 step7_。第 4、5 步各有一个子任务。初始合同应失败，不修改合同或删掉测试。

| 步骤 | 实现 |
|---|---|
| 1 | 两个定时 Future 的真实 Tokio join |
| 2 | 容量受限的 mpsc producer，接收者关闭后停止 |
| 3 | 动态任务限流，保持原始输入顺序 |
| 4 | ReceiverStream 和 merge_positive（StreamExt） |
| 5 | 显式取消、超时和 abort 后 join |
| 6 | spawn_blocking 与异步任务之间的协作 |
| 7 | 带 Send 返回 Future 的异步 Trait |

完成后运行 cargo test --locked --offline --all-targets 与 cargo run --locked --offline --example async_workbench。应观察到：

```text
joined=(7, 9)
positive=[2, 3, 4, 5]; sent=3
ordered=[9, 1, 4]
race=Cancelled
blocking=42
sample=Some(31)
```

以上是合同期望，实际以你的输出为准。先预测后运行，记录在 LEARNING_LOG.md。改变输入、容量、取消顺序，补一条自己的边界测试；解释何时真的发生 Pending、为什么异步不等于多线程、为什么 drop JoinHandle 不等于 abort。

公开合同只证明可观察行为，不证明你用了指定语法或已经独立掌握。阅读官方 Tokio 及 futures-util 文档；完整课程平台另有 course/BOOK_D10_TOKIO_STREAM.md。
