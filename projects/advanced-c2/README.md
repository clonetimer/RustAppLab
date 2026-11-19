# C2 · 用 Axum 提供可持久化的异步工单服务

本阶段把已有 ServiceDesk 的业务规则接到 Axum / Tokio：浏览器可以新建、查看和关闭工单，多个请求通过一个持久化写者更新同一份日志，关闭服务时等待在途请求结束。

## 什么时候学习、要准备什么

先理解 [同步 ServiceDesk](../../course/studio/servicedesk.md) 的业务与日志，并完成 [C1 异步下载](../advanced-c1/README.md)。刚完成 FirstCard 时，先看 [基础后续指南](../../course/AFTER_FIRSTCARD.md)。

- `learner/` 保留三个核心 TODO，`reference/` 是完整参考实现；两者不是你的个人完成记录。
- C2 独立使用 **Rust 1.85.0 / Edition 2021**，不改变基础平台 Rust 1.74。
- 首次 prepare 联网获取锁定依赖；之后检查 locked/offline。源代码目录与包含 vendor 的发行离线包不同。
- 平台创建的工程带 `support/servicedesk` 业务库副本；直接阅读仓库模板时，其 path 依赖指向 `apps/servicedesk`。

## 从自己的第一个 TODO 开始

在 **RustAppLab 平台根目录**运行，使用尚不存在的目录名：

```powershell
rustup toolchain install 1.85.0 --profile minimal
cargo run --locked --offline -p labquest -- advanced-c2 start "my-axum-servicedesk"
cargo run --locked --offline -p labquest -- advanced-c2 prepare "my-axum-servicedesk"
cargo run --locked --offline -p labquest -- advanced-c2 lesson "my-axum-servicedesk"
cargo run --locked --offline -p labquest -- advanced-c2 check "my-axum-servicedesk" 1
```

工具链安装及 prepare 会联网。prepare 只执行固定参考程序，不运行学员源码；外层 `--offline` 仅限制平台启动。初次 TODO 检查失败是学习起点，环境准备成功不是课程通过。

只需实现 **自己工程 `src/lib.rs`** 的三个函数，也可拆分额外 Rust 模块。`model.rs`、`handlers.rs`、CLI 与网页资源是已提供的基础设施。

| 阶段 | 核心函数 | 应能验证的行为 |
|---|---|---|
| C2-1 | `start_store` | 验证旧日志，用容量 64 的队列与阻塞工作线程串行写盘，失败后拒绝陈旧状态 |
| C2-2 | `router` | 注册网页、静态资源、健康检查及工单 API，限制 JSON 正文为 2048 字节 |
| C2-3 | `serve` | 实际服务 TCP listener，收到关闭信号后等待活动请求完成 |

在 **学员工程目录**运行对应可见测试：

```powershell
cargo +1.85.0 test --locked --offline --test contracts c2_1_
cargo +1.85.0 test --locked --offline --test contracts c2_2_
cargo +1.85.0 test --locked --offline --test contracts c2_3_
```

回到平台根目录，把 `advanced-c2 check` 的阶段依次改为 `2`、`3` 或 `all`，执行累计检查。平台使用固定业务库、handler、锁文件和合同；修改可见测试不等于通过验收。查看学员工程 `.c2-prepare`、`.c2-reports` 的诊断。

## 三个阶段通过后，使用自己的工单服务

在 **学员工程目录**运行，首次选择尚不存在的日志文件：

```powershell
cargo +1.85.0 run --locked --offline -- init "my-tickets.db"
cargo +1.85.0 run --locked --offline -- serve "my-tickets.db" --port 3000
```

打开 [本机工单页面](http://127.0.0.1:3000/)，添加两条自己的练习任务并关闭其中一条。按 Ctrl+C 停服，再只执行 `serve`；重启后应仍有两条工单，其中一条完成。不要重复 `init`；它只创建新日志。

想先观察参考软件，可在 **平台根目录**执行（第一条联网）：

```powershell
cargo +1.85.0 fetch --locked --manifest-path projects/advanced-c2/reference/Cargo.toml
cargo +1.85.0 run --locked --offline --manifest-path projects/advanced-c2/reference/Cargo.toml -- init "my-c2-reference.db"
cargo +1.85.0 run --locked --offline --manifest-path projects/advanced-c2/reference/Cargo.toml -- serve "my-c2-reference.db" --port 3000
```

先停止占用同一端口的旧服务；参考程序的运行记录不替代自己的实现。

## 学会了什么、完成后去哪里

你应能画出“请求 → handler → 队列 → 单写者 → 日志 → 响应”的流程，解释为什么仅共享内存不足以证明持久化正确，以及为什么 HTTP 取消不等于撤销已落盘的数据。

按 [C2 完整讲义](../../course/ADVANCED_C2.md) 做一个个人并发或存储失败实验，并补一条回归测试；保存请求、响应、重启后数据与解释。存储失败应返回 503，后续读取不能假装旧快照仍然有效；非法标题仍按输入错误处理。

在 **平台根目录**运行：

```powershell
cargo run --locked --offline -p labquest -- advanced-c2 next "my-axum-servicedesk"
```

源码变动后重新检查。C2 是 [当前课程地图](../../course/CURRICULUM.md) 的已有终点；下一步完成自己的并发、持久化和关闭实验，整理作品交付。[异步网络后续说明](../../course/ASYNC_NETWORK_NEXT.md) 区分现有能力与未来方向，不需要寻找新的 C3 关卡。

## 离线包与技术边界

`C2-reference-offline.zip` / `C2-learner-offline.zip` 带锁定依赖、vendor、隐藏 Cargo 配置、业务库副本、网页和讲义，不含 Rust / MSVC。解压后在包内运行 `cargo +1.85.0 test --frozen --all-targets`；学员包初始业务测试应失败，已有工程无需再次 start。固定离线评分的 `RUSTAPPLAB_C2_VENDOR` 配置见讲义。

仅绑定 `127.0.0.1` HTTP；本机写请求标记不是账号认证。启动时同步验证日志，请求期间单个阻塞工作线程串行写盘。优雅关闭等待在途请求，没有强制截止时间；不保证公网 / TLS、多进程写入或断电原子性。平台检查只执行自己的可信 Rust 代码，不是恶意程序沙箱。
