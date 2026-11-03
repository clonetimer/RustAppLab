# C1：用 Tokio / Reqwest 实现真实异步下载

C1 在原同步 NetFetch 和 AsyncBoard 之后学习，复用所有权、Result 和测试组织的知识。目标是亲手实现一个可取消的回环 HTTP 下载队列。C1 的三个检查点仍属于同一款软件，不是 C2/C3 课程。

## 安装与入口

基础平台继续使用 Rust 1.74；C1 是独立的 Rust 1.85.0 / Edition 2021 工作区。安装一次附加工具链，不需要更改默认版本：

```powershell
rustup toolchain install 1.85.0 --profile minimal
# 以下命令在 RustAppLab 平台根目录运行
cargo run --locked --offline -p labquest -- advanced-c1 start "my-async-download"
cargo run --locked --offline -p labquest -- advanced-c1 prepare "my-async-download"
cargo run --locked --offline -p labquest -- advanced-c1 lesson "my-async-download"
cargo run --locked --offline -p labquest -- advanced-c1 check "my-async-download" 1
cargo run --locked --offline -p labquest -- advanced-c1 next "my-async-download"
```

`start` 只创建全新目录，已有目录保持原样。`prepare` 明确联网执行固定 `cargo fetch --locked`，然后验证平台参考代码；它不运行学员源码、学员 Cargo.toml、build.rs、tests 或 .cargo 配置。外层 `cargo run --offline` 只限制启动平台；显式 prepare 内部的 fetch 允许联网。准备日志在 `.c1-prepare`，不是成绩。依赖已准备后，检查全部使用 locked/offline。

`learn lesson/reading/next/check <工程>` 也支持 C1。`next/lesson/reading` 不执行代码、不下载依赖、不写成绩；新源码或固定合同改变后旧记录失效。查看 `.c1-reports` 中的实际 Cargo 输出定位错误。缺失依赖或 Rust1.85.0 是环境问题，不会生成新的失败成绩。

## 先体验参考软件

在平台根目录：

```powershell
cargo +1.85.0 fetch --locked --manifest-path projects/advanced-c1/reference/Cargo.toml
cargo +1.85.0 run --locked --offline --manifest-path projects/advanced-c1/reference/Cargo.toml -- demo "my-first-download"
```

demo 在本机随机端口启动教学服务器，下载三个 UTF-8 文件，故意让第一个请求较慢。终端依然按输入顺序打印文件名与字节数；输出目录出现于全部成功之后。输出目录必须不存在。这个服务器只服务固定演示内容，不是 Axum 服务端。

使用自己的回环 HTTP 服务时：

```powershell
cargo +1.85.0 run --locked --offline --manifest-path projects/advanced-c1/reference/Cargo.toml -- download "my-downloads" --concurrency 2 --timeout-ms 5000 --idle-ms 2000 --connect-ms 2000 --max-bytes 1048576 http://127.0.0.1:8080/a http://127.0.0.1:8080/b
```

正文按原始字节保存为 001.bin、002.bin；扩展名不代表编码。Ctrl+C 取消本批次。CLI 仅在整个批次成功后写 stdout；错误写 stderr 并退出非零。Windows 中文/空格目录必须加引号。

## 三个检查点与修改位置

学员修改自己工程的 `src/lib.rs`，可自行拆分额外 Rust 模块。`src/main.rs` 是提供的 CLI/演示服务器，`src/model.rs` 是类型定义；它们不是完整核心答案。

| 检查点 | 接口 | 需要证明的行为 |
|---|---|---|
| 1：输入与资源预算 | `validate_jobs(&[Job], Limits)` | 1–64 个任务；1–8 路并发；正文上限 1–8 MiB；三类超时均在 (0,60秒]；跨平台文件名与回环 URL |
| 2：单请求 | `fetch_one(Job, Limits, watch::Receiver<bool>)` | 真实异步字节读取；状态码、正文上限、完整响应/正文空闲超时、取消 |
| 3：队列与发布 | `download_batch(Vec<Job>, &Path, Limits, watch::Receiver<bool>)` | 有界投递、稳定排序、全部成功后发布，失败/取消清理，调用方丢弃 future 后清理 |

在学员工程目录运行固定可见测试：

```powershell
cargo +1.85.0 test --locked --offline --test contracts c1_1_
cargo +1.85.0 test --locked --offline --test contracts c1_2_
cargo +1.85.0 test --locked --offline --test contracts c1_3_
```

回到平台根目录依次 `advanced-c1 check <工程> 1`、`2`、`3`；`all` 和省略阶段均累计检查三个阶段，并运行文档测试。平台在临时工作区使用固定清单、锁与测试，只复制学员的 src。修改可见测试不能代替固定验收。初始 TODO 可以编译，业务测试失败是学习起点。

## 参数与规则

`connect_timeout` 限制连接建立；`total_timeout` 从请求开始覆盖响应头和整个正文，不因收到新片段而重置；`idle_timeout` 限制等待下一个正文片段。请求排队时间不包含在单请求 total_timeout 中。`max_bytes` 是每个正文的实际字节上限，即使服务器不发送 Content-Length 或采用 chunked，也必须逐块检查。并发最多8且每个正文最多8 MiB；网络任务只返回有界内存正文，控制器写入本批次暂存目录。

只接受 `http://127.0.0.1:端口/...` 或 `http://[::1]:端口/...` 等数字回环地址；不使用 DNS、系统代理或重定向。名称由 ASCII 字母、数字、点、下划线和横线构成，1–64字节；拒绝前导/末尾点、路径分隔符、Windows 保留设备名及大小写不敏感重复名称。CLI 自动生成安全名称；库测试另行覆盖非法输入。

取消信号使用 watch 的布尔值：一旦发出 true 不再重置。先置 true 时不应连接；关闭仍为 false 的发送端不代表取消，也不能引起忙循环。取消后停止投递，终止并等待本批次任务退出。远端服务可能已经开始处理请求，取消并不保证远端业务回滚。

## 三级提示

**提示1：画出边界。** 画出“任务计划→校验→等待连接→响应头→正文片段→暂存文件→发布目录”。标记哪一层拥有任务、字节缓冲区和暂存目录。写出每一步失败时哪些内容应保持不变。

**提示2：缩小实现。** 先在单线程 Tokio runtime 中完成单请求，再用 `select!` 同时等待取消和请求超时。将总超时包在完整传输外面，将空闲超时包在每次 chunk 等待外面。队列只保持 concurrency 个已启动任务；完成一个再补一个，而不是一次启动全部请求后再等待。

**提示3：处理收尾。** 用 JoinSet 管理拥有型任务，记录每项输入索引；结果按索引恢复顺序。网络任务不写文件，使控制器取消后可以安全清理。用拥有暂存目录的 Drop 守卫覆盖调用方直接丢弃 future 的路径，显式取消时还要 abort 并 join。发布前再次检查目标是否存在。

## 个人实验

用虚构数据比较 concurrency=1/2/8，但不要仅凭总耗时证明并发。观察服务器同时收到多少请求。分别让服务器不发送响应头、发一部分正文后停顿、持续慢速发送、返回302/404、声明错误长度；预测错误类型再运行。取消前后列出输出父目录并核对已有文件内容。增加一条自己的回归测试，并解释为什么错误处理与文件所有权应放在这些位置。

## 离线包

发布产物分为 `C1-reference-offline.zip` 和 `C1-learner-offline.zip`；学员包只含三个核心 TODO，参考包含完成实现。两者都带 Cargo.lock、vendor、`.cargo/config.toml` 和依赖许可证；需要本机事先装好 Rust1.85.0 与 Windows MSVC 或 Linux C 工具链，包不包含编译器。

学员包带有 `.applab-c1` 工程标记，可直接作为 LabQuest 的学员工程使用；不要在已解压的目录再次运行 start。参考包不带学员标记。

解压后在含 Cargo.toml 的目录运行 `cargo +1.85.0 test --frozen --all-targets`。参考包预期通过，学员包初始业务测试预期失败。不要删除以点开头的 Cargo 配置和校验文件。平台固定评测要使用该离线源时，在 PowerShell 设置 `$env:RUSTAPPLAB_C1_VENDOR = (Resolve-Path "C:/你的路径/vendor").Path`，再运行 `advanced-c1 check`；它只为自己的临时工程设置该源，不改全局配置。迁移和空缓存的实际验证以当前 CI 产物为准。

## 使用边界与阅读

输出父目录应为可信本地目录，每个输出路径只有一个写者。本课不提供敌对目录竞争安全、断电原子性、公网/TLS、重试、完整流式落盘或生产服务承诺。本地有界文件写入使用标准库同步 I/O，网络传输是真正的 Tokio/Reqwest 异步 I/O。评测会执行自己的可信 Rust 代码，不是恶意程序沙箱。C2 Axum 留在下一阶段。

阅读：[中文 Book 测试组织](https://rust.bootcss.com/ch11-03-test-organization.html)、[官方 async/await](https://doc.rust-lang.org/book/ch17-00-async-await.html)、[Tokio 教程](https://tokio.rs/tokio/tutorial)、[JoinSet](https://docs.rs/tokio/1.48.0/tokio/task/struct.JoinSet.html)、[Reqwest 0.12.24 源码](https://github.com/seanmonstar/reqwest/tree/v0.12.24)、[Cargo vendor](https://doc.rust-lang.org/cargo/commands/cargo-vendor.html)。中文旧 Book 用来理解基础概念，异步 API 按所锁定版本的官方资料核对。
