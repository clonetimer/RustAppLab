# AsyncBoard · 离线提醒工作台（0.16.0）

本软件使用真正的 `async fn`、`.await`、`Future`、`Poll`、`Waker` 和单独的定时器驱动线程。任务是并发等待而不是逐个 `sleep`。可查看 `RESULTS.tsv` 中的完成顺序。

**不是 Tokio/Reqwest/Axum**：这里没有非阻塞网络 I/O、事件反应器、多线程任务池或 TLS。为了零基础和离线教学，核心执行器封装在 `src/runtime.rs`，学员可以先学 `src/lib.rs` 的三个任务，再学习底层原理。

```powershell
cargo run --offline -p asyncboard -- demo my-async-demo
cargo run --offline -p asyncboard -- preview my-async-demo/jobs.tsv
cargo run --offline -p asyncboard -- run my-async-demo/jobs.tsv my-async-demo/SECOND.tsv
```

每个 ID 只能包含 ASCII 字母、数字、下划线、短横线；延时上限 30 秒；一次最多 128 条。不会覆盖已有报告，避免误改数据。
