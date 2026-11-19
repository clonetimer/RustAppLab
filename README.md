# Rust AppLab · 0.19.0-rc.3 候选发布版

通过亲手制作小软件学习 Rust：先读懂输入和输出，每次实现一个函数，用自己的数据验证，最后完成独立作品。

## 我现在应该做什么？

| 你现在的位置 | 现在只做这一件事 | 做完应得到什么 |
|---|---|---|
| 还没有运行过 Rust | 打开 [开始指南](README_START_HERE.md)；Windows 也可双击 [START_HERE.html](START_HERE.html)，跟着做 FirstCard | 能随姓名和任务变化的个人任务卡 |
| **FirstCard 已经通过，不知道怎么继续** | **打开 [FirstCard 完成后的下一步](course/AFTER_FIRSTCARD.md)**；复盘后创建 UnitLab | 明确自己练到了什么，并开始第一个计算函数 |
| 已有 `my-unitlab` 等工程，今天想接着学 | 双击 `LEARN_NEXT_WINDOWS.cmd`，或使用下方 `learn next` 命令 | 找到当前任务、要改的函数和检查命令 |
| 当前作品三步都通过了 | 用自己的输入运行一次、试一个错误输入、写下解释，再按该作品 README 的“下一步”继续 | 有真实用途、运行结果和理解记录的作品 |
| 想先了解整个课程 | 看 [课程地图](course/CURRICULUM.md)，再回到当前这一步 | 知道每个阶段为什么学，不必现在学完所有专题 |

**今天只需决定当前一步。** 新手的前三款后续作品固定为 **UnitLab → QuizDeck → TextScope**。FileQuest 等原有 00–31 章节，以及 Rust Book 专题，保留为拓展和概念补课入口。

## 每一课怎样学习，怎样判断学会了？

1. **先知道用途。** README 说明要做什么软件、需要哪些基础，以及完成后能做什么。
2. **每次实现一个函数。** 学员工程在 `my-...`；平台 `apps/...` 中的是参考源码。先读任务卡，再改自己的 TODO。
3. **保存并真实检查。** 初始 TODO 失败是起点；通过第一步后才做第二步，最后再运行完整软件。
4. **换自己的输入。** 先预测结果，试一组新数据和一组错误输入，记录实际行为。
5. **解释并继续。** 在工程的 `LEARNING_LOG.md` 写出一个函数的输入、返回值和失败原因，再跟随 README 的下一课入口。

| 检查层次 | 它说明什么 | 你需要做的事 |
|---|---|---|
| 自动检查通过 | 当前程序满足已经检查的行为 | 修改代码后重新检查，不沿用旧成绩 |
| 自己能复现 | 能离开演示输入，独立运行自己的作品 | 换数据、预测、运行，找到自己的产物 |
| 能解释和迁移 | 对本课概念形成了可观察的理解 | 解释为什么这样写，再尝试一个新输入或新增测试 |

例如，**FirstCard** 练习变量、参数与格式化输出；**UnitLab** 练习函数返回值、分支和错误；**QuizDeck** 进一步处理结构化文本与集合；**TextScope** 练习字符串统计、借用和迭代。每一步都有具体用途，自动通过不等于已经掌握整门语言。

## 选择下载内容

在本仓库 Releases 的 `v0.19.0-rc.3` 候选版本中选择文件。发布状态以实际 Release 与同提交 CI 为准，文件名不是验收结论。

| 目的 | 下载包 | 需要什么 |
|---|---|---|
| 先体验成品 | `RustAppLab-0.19.0-rc.3-<平台>-runtime.zip` | 对应系统；参考程序无需 Cargo，运行学习检查仍需 Rust |
| 跟随全部课程学习 | `RustAppLab-0.19.0-rc.3-source.zip` | Rust/Cargo 与系统构建工具；这是推荐的完整学习入口 |
| 只取学员骨架 | `RustAppLab-0.19.0-rc.3-learner-kit.zip` | 17 个基础/进阶 A/B 工程；核心 TODO 待完成，不含 C1/C2 vendor |
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

遇到相应概念时，可补读 [所有权与切片](course/BOOK_OWNERSHIP_PRACTICE.md)、[集合与错误传播](course/BOOK_COLLECTIONS_ERRORS.md)、[泛型与生命周期](course/BOOK_GENERICS_LIFETIMES.md)。它们不要求在 UnitLab 前完成。[概念审计](course/BOOK_CONCEPT_AUDIT.md) 记录第 4–6、8–10 章本次 118 个目标的证据和剩余 31 项；这个范围不能作为整本书的完成率。

## 已有工程继续用

在完整源码包的根目录运行，不需要重建旧工程：

```powershell
cargo run --locked --offline -p labquest -- learn next "D:/练习/my-unitlab"
cargo run --locked --offline -p labquest -- learn lesson "D:/练习/my-unitlab"
cargo run --locked --offline -p labquest -- learn check "D:/练习/my-unitlab"
```

把示例路径替换成自己已有工程的实际路径。Windows 可用 `LEARN_NEXT_WINDOWS.cmd`。`start` 只创建新工程，不用它来继续已有作品。源码修改后重新检查，读取历史进度不会执行学员代码。

## 候选版范围与试用

此轮功能冻结在 **B4、C1、C2**；优先修复实际使用和教学衔接问题，不再增加课程数量。

支持范围为 Windows x64 MSVC、Linux x86_64；基础/A/B 的最低 Rust 为 1.74，C1/C2 单独要求 1.85.0。StoreDesk 需系统 SQLite，见 [支持说明](SUPPORTED_PLATFORMS.md)。普通学员不需要 Python 或 Playwright，不运行维护者的发布验收脚本。

此版本为 **RC 候选版，不是 Stable**。当前源码必须重新通过 Windows/Linux、最低版本、浏览器、C1/C2 和交付检查；旧版本证据不继承。真人学习效果与 Win10 本机体验按 [试用记录](course/USER_TRIAL.md) 记录，不能以 CI 代填。参见 [发布规则](RELEASE_POLICY.md)、[已知边界](KNOWN_ISSUES.md)、[后续范围](ROADMAP.md)。

网络服务仅用于可信本机回环 HTTP；不提供公网/TLS、账号认证或恶意代码沙箱。历史发布记录保留在 [变更记录](RELEASE_NOTES.md) 与 `verification/`，不把历史状态当作当前范围。
