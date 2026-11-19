# C1 · 用 Tokio / Reqwest 实现异步下载队列

本阶段交付一个真正发出 HTTP 请求的 Rust 下载器：限制同时下载数量，处理超时和取消，全部成功后才发布输出目录。它承接同步 NetFetch 和 AsyncBoard 的练习。

## 前置知识与工程区别

先完成 [NetFetch](../../course/studio/netfetch.md) 与 [AsyncBoard](../../course/studio/asyncboard.md)，能读懂所有权、`Result` 和测试。刚完成 FirstCard 时，从 [基础后续指南](../../course/AFTER_FIRSTCARD.md) 继续，不必提前安装异步依赖。

- `learner/`：三个核心 TODO；平台会生成你自己的独立工程。
- `reference/`：完整实现和相同合同测试，供观察目标行为。
- C1 使用 Edition 2021、**Rust 1.85.0**；基础平台仍支持 Rust 1.74，不需要更改默认工具链。
- 首次准备需要网络；依赖准备好后检查使用 `--locked --offline`。源码目录不等于自带 vendor 的离线包。

## 创建工程并完成第一个检查点

在 **RustAppLab 平台根目录**执行；安装工具链和 prepare 会联网，目录名应未使用：

```powershell
rustup toolchain install 1.85.0 --profile minimal
cargo run --locked --offline -p labquest -- advanced-c1 start "my-async-download"
cargo run --locked --offline -p labquest -- advanced-c1 prepare "my-async-download"
cargo run --locked --offline -p labquest -- advanced-c1 lesson "my-async-download"
cargo run --locked --offline -p labquest -- advanced-c1 check "my-async-download" 1
```

`prepare` 只抓取锁定依赖、验证固定参考程序，不执行学员源码；外层 `--offline` 不禁止显式 prepare 内部联网。日志在 `.c1-prepare`，准备成功还不是学习成绩。初始 TODO 的业务检查失败是预期结果。

打开 **自己工程的 `src/lib.rs`**，一次实现一个函数。`src/main.rs` 提供 CLI / 演示服务器，`src/model.rs` 提供类型：

| 阶段 | 你实现的函数 | 应能证明的行为 |
|---|---|---|
| 1 | `validate_jobs` | 拒绝非法任务、资源预算、文件名及非回环 URL |
| 2 | `fetch_one` | 异步读取真实字节，处理状态码、大小上限、超时和取消 |
| 3 | `download_batch` | 有界并发、结果顺序稳定、成功发布、失败或取消清理 |

在 **学员工程目录**可单独运行可见测试：

```powershell
cargo +1.85.0 test --locked --offline --test contracts c1_1_
cargo +1.85.0 test --locked --offline --test contracts c1_2_
cargo +1.85.0 test --locked --offline --test contracts c1_3_
```

回到平台根目录，将 `advanced-c1 check` 的阶段依次改成 `2`、`3`。`all` 会累计检查三个阶段及文档测试；修改本地测试不能替代平台固定合同。原始检查输出在学员工程 `.c1-reports`。

## 完成后，真实运行自己的程序

三个阶段通过后，在 **学员工程目录**执行：

```powershell
cargo +1.85.0 run --locked --offline -- demo "my-first-download"
```

程序启动本机演示服务器，下载三个文件；第一个请求故意较慢，终端仍按输入顺序显示文件名和字节数。仅在整批成功后出现输出目录。重跑请换新目录名，已有目录会被拒绝。

要先体验完整参考软件，可在 **平台根目录**运行以下两条；第一条联网：

```powershell
cargo +1.85.0 fetch --locked --manifest-path projects/advanced-c1/reference/Cargo.toml
cargo +1.85.0 run --locked --offline --manifest-path projects/advanced-c1/reference/Cargo.toml -- demo "my-c1-reference-demo"
```

## 自查、个人实验与下一步

按 [完整 C1 讲义](../../course/ADVANCED_C1.md) 用自己的回环 HTTP 数据比较并发 1 / 2，并做一次取消或慢正文实验。先写预测，再核对输出目录、已有文件和错误；不能只用总耗时声称并发正确。

你应能解释连接超时、总超时、正文空闲超时的区别，指出哪个部分拥有暂存目录，并为一个失败分支补充回归测试。保存自己的输入、命令、实际输出和解释；参考 demo 成功只说明参考程序运行正常。

回到 **平台根目录**查看当前源码对应的记录：

```powershell
cargo run --locked --offline -p labquest -- advanced-c1 next "my-async-download"
```

修改源码后重跑检查。完成个人实验，再去 [C2 Axum 工单服务](../advanced-c2/README.md)，把异步客户端知识用于理解服务端路由、共享状态与持久化。

## 离线包与使用边界

`C1-reference-offline.zip` 与 `C1-learner-offline.zip` 分别带参考代码或 TODO、锁文件、vendor、`.cargo/config.toml` 和许可证，不含编译器。先备好 Rust 1.85.0 与本机编译工具链，解压后在包内运行 `cargo +1.85.0 test --frozen --all-targets`；学员包初始业务测试应失败，不要再次 start 覆盖它。

平台使用离线包依赖的 `RUSTAPPLAB_C1_VENDOR` 配置见讲义。维护脚本 `scripts/c1_check.py`、`scripts/package_c1.py` 需要 Python 3.11+，普通学员无需运行它们。

仅支持数字回环 HTTP，不提供公网 / TLS、代理、重定向、重试或断电原子性。网络 I/O 是异步的，本地有界文件写入仍用同步标准库；取消不能撤销远端已发生的业务。检查只用于自己的可信代码。
