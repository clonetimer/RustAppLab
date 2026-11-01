# 异步网络依赖准备：面向维护者，不是新手前置作业

## 0.15.0 的实际完成范围：同步基础版 ≠ 异步正式版

本轮基础源码通过离线构建，但容器无法解析 crates.io，备用下载入口也没有取得发行包。没有真实 crate 就不能证明成熟异步框架已经编译运行。我们保留两款网络作品的规划，不用同步标准库实现冒充 async，也不把未运行的代码写成“已通过”。0.15.0 已交付 NetFetch/ServiceDesk 的**本机同步基础版**：真实 TCP/HTTP、文件下载与工单网页，由标准库实现和回环测试验证。Tokio/Reqwest/Axum 下载队列与 Web 框架的异步实现仍然未完成，也未拿到真实 crate 依赖。

这个准备流程解决具体的下一步：在可联网的开发机获取依赖源代码、生成真实 Cargo.lock、运行回环 HTTP 探针，并生成可携带的离线目录。它不是已经下载好的 vendor 包，也不代表异步应用已经编译通过。同步基础版不需要该准备器。

## 普通学员不用做什么

FirstCard、现有九款 Studio 练习以及 TaskPocket/PageMint 都不需要此网络依赖。不要为了进入 UnitLab 而执行本流程；根 Workspace 的 Cargo.lock 仍没有远程 crate。

## Windows 操作

在完整源码包根目录，双击 `PREPARE_NETWORK_WINDOWS.cmd`。这个入口**会联网**并运行第三方依赖的构建脚本，应只在你信任这些依赖且允许联网的开发环境执行。它使用已有 Rust/Cargo，不安装 Python，不关闭 TLS 验证，不要求管理员权限。

手动等价命令：

```powershell
cargo run --offline -p labquest -- ecosystem prepare network-offline-kit
```

外层 `--offline` 只保证编译现有 LabQuest 不联网；LabQuest 的 prepare 明确会启动联网 Cargo 子命令，这是有意设计，不是“全流程离线”。

想先查看会生成什么而不联网，可以用不同的新目录：

```powershell
cargo run --offline -p labquest -- ecosystem plan
cargo run --offline -p labquest -- ecosystem init network-review
```

init 不获取依赖、不编译网络探针，不会生成合格结论。

## 流程与结果

prepare 记录 rustc/cargo 版本；首次生成锁文件（重试保留原锁文件）；fetch 下载依赖；vendor 复制依赖源码和许可证；写入相对目录配置；在临时构建目录中执行 `cargo test --locked --offline --all-targets`。探针检查回环 HTTP/JSON、优雅关闭、超时和任务取消。无需外部 Web 站点，也不等于已经验证互联网 HTTPS 下载。

准备包是独立 Workspace，不添加到初学者根 Workspace。候选依赖为 Tokio 1.48.0、Axum 0.8.6、Reqwest 0.12.24、Serde 1.0.228、serde_json 1.0.145。版本用于固定这次兼容性实验，不宣称最新版、无漏洞或长期支持；正式网络作品仍需复查维护状态、安全公告和锁文件。探针声明的 Rust 1.85 也不是已经验证的完整依赖图最低版本，先使用已安装的较新工具链执行，实际版本写入日志。

生成内容：Cargo.toml、真实 Cargo.lock、src、vendor、.cargo/config.toml、logs、STATUS.md 与 PACK_WINDOWS.ps1。构建缓存放在临时目录，成功或正常失败返回后清理。不会复制用户的 Cargo 全局配置、账号凭据或其它项目。

**STATUS.md 才说明做到了哪一步。** 下载失败、编译失败、测试失败都必须停下，不能拿“目录存在”当作已经通过。失败时保留目录，处理网络后可以同一命令重试；检查器不会主动覆盖被修改的探针源码。如果你修改过它，请保留自己的修改后另选新目录。

## 打包与移交

在 PowerShell 运行：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\network-offline-kit\PACK_WINDOWS.ps1
```

ZIP 和 SHA-256 位于目录旁边；同名 ZIP 已存在时拒绝覆盖。压缩方式不排除 `.cargo`，因此配置与 vendor 会一起保存。只处理生成的准备目录，不扫描你的文档和学习成果。

一次准备失败但 vendor 已下载成功时，也可以打包用于排查；它仍是失败证据而不是通过证据。另一个操作系统收到依赖包后仍须自己编译测试，Windows 通过不代表 Linux/macOS 通过。

## 0.15.0 验证边界

本版已测试准备器本身的“不覆盖、失败即停、状态记录、重试保留锁文件、拒绝改动源码”等逻辑，其中成功路径用受控假执行器测试，不是 Cargo 联网实测。网络探针只有格式/语法解析检查，没有依赖可用时的类型检查和真实运行证据。请勿把这几项维护者单元测试计成 NetFetch / ServiceDesk 成果。

## 官方依据

Cargo fetch：<https://doc.rust-lang.org/cargo/commands/cargo-fetch.html>。
Cargo vendor 与本地 source 配置：<https://doc.rust-lang.org/cargo/commands/cargo-vendor.html>。
Tokio：<https://docs.rs/tokio/1.48.0/tokio/>。
Axum：<https://docs.rs/axum/0.8.6/axum/>。
Reqwest：<https://docs.rs/crate/reqwest/0.12.24>。
