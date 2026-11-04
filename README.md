# Rust AppLab · 0.19.0-rc.1 候选发布版

通过真实软件学习 Rust：先运行参考程序理解用途，再完成自己的函数，最后交付独立作品。此轮功能冻结在 **B4、C1、C2**；后续优先修复实际使用问题，不再增加课程数量。

**首次使用：[开始指南](README_START_HERE.md)。零基础读者也可直接打开 [START_HERE.html](START_HERE.html)。**

## 选择下载内容

在本仓库 Releases 的 `v0.19.0-rc.1` 候选版本中选择文件。发布状态以实际 Release 与同提交 CI 为准，文件名不是验收结论。

| 目的 | 下载包 | 需要什么 |
|---|---|---|
| 先体验成品 | `RustAppLab-0.19.0-rc.1-<平台>-runtime.zip` | 对应系统；参考程序无需 Cargo，运行学习检查仍需 Rust |
| 跟随全部课程学习 | `RustAppLab-0.19.0-rc.1-source.zip` | Rust/Cargo 与系统构建工具；这是推荐的完整学习入口 |
| 只取学员骨架 | `RustAppLab-0.19.0-rc.1-learner-kit.zip` | 17 个基础/进阶 A/B 工程；核心 TODO 待完成，不含 C1/C2 vendor |
| 离线完成异步课程 | `C1-learner-offline.zip`、`C2-learner-offline.zip` | Rust 1.85.0 已安装；包内提供锁定依赖源码 |
| 对照异步参考实现 | `C1-reference-offline.zip`、`C2-reference-offline.zip` | Rust 1.85.0；不是预编译 EXE |

## 已交付的学习路线

FirstCard → UnitLab → QuizDeck → TextScope → LedgerLite / BadgeForge / TermGarden → 并发、存储与同步网络 → 事务与等待 → TaskPocket / PageMint 独立作品 → 进阶 A → B1–B4 → C1 → C2。

| 阶段 | 主要内容 | 入口 |
|---|---|---|
| 基础与 Studio | 17 个根工作区包；12 款 Studio、36 张任务卡、108 条分级提示 | [课程目录](course/CURRICULUM.md)、[作品页](WORKSHOP.html) |
| 独立作品 | TaskPocket 与 PageMint；自行设计、实现和验证 | [独立开发](INDEPENDENT_PATH.html) |
| 进阶 A | 共享句柄、弱引用、闭包、快照与接口设计 | [进阶 A](ADVANCED_A.html) |
| B1–B4 | 业务库、多 crate、feature；独立 Serde JSON 导出 | [进阶 B](course/ADVANCED_B.md) |
| C1 | Tokio/Reqwest 有界异步下载、超时、取消、暂存发布 | [C1 讲义](course/ADVANCED_C1.md) |
| C2 | Axum JSON 路由、单写者日志、并发客户端与优雅关闭 | [C2 讲义](course/ADVANCED_C2.md) |

中文 Rust Book 用于基础概念与项目组织对照；异步 API 按锁定版本的官方文档学习。使用 `learn reading <工程>` 查阅相关章节，见 [Book 对照指南](course/BOOK_GUIDE.md)。

## 已有工程继续用

在完整源码包的根目录运行，不需要重建旧工程：

```powershell
cargo run --locked --offline -p labquest -- learn lesson "D:/练习/my-unitlab"
cargo run --locked --offline -p labquest -- learn check "D:/练习/my-unitlab"
```

Windows 可用 `LEARN_NEXT_WINDOWS.cmd`。参考程序输出不等于学员已通过；核心 TODO 初始检查失败是正常学习起点。源码修改后重新检查，读取历史进度不会执行学员代码。

## 候选版范围与试用

支持范围为 Windows x64 MSVC、Linux x86_64；基础/A/B 的最低 Rust 为 1.74，C1/C2 单独要求 1.85.0。StoreDesk 需系统 SQLite，见 [支持说明](SUPPORTED_PLATFORMS.md)。普通学员不需要 Python 或 Playwright，不运行维护者的发布验收脚本。

此版本为 **RC 候选版，不是 Stable**。当前源码必须重新通过 Windows/Linux、最低版本、浏览器、C1/C2 和交付检查；旧版本证据不继承。真人学习效果与 Win10 本机体验按 [试用记录](course/USER_TRIAL.md) 记录，不能以 CI 代填。参见 [发布规则](RELEASE_POLICY.md)、[已知边界](KNOWN_ISSUES.md)、[后续范围](ROADMAP.md)。

网络服务仅用于可信本机回环 HTTP；不提供公网/TLS、账号认证或恶意代码沙箱。历史发布记录保留在 [变更记录](RELEASE_NOTES.md) 与 `verification/`，不把历史状态当作当前范围。
