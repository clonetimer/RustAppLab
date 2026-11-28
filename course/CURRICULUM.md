# Rust AppLab 0.19.0-rc.6 · 当前课程地图

**这张表用来了解方向，不是今天的任务清单。** 完成 FirstCard 后请直接看 [下一步操作与能力复盘](AFTER_FIRSTCARD.md)。每一课结束都应留下自己的代码、输入、结果和概念解释，再跟随该应用 README 进入下一课。

本轮功能范围冻结在C2；先完成当前作品，再进入下一阶段。自动检查验证程序行为，不代替真人理解。参考程序、学员骨架和独立开发合同分别标注，不按目录数量计算学习成效。

## 推荐顺序与入口

下表 0–4 阶段均可从 [网页学习工作台](WORKBENCH.md) 进入；其余路线保留各自工程和执行入口。

| 阶段 | 作品/课程 | 学习内容 | 入口 |
|---|---|---|---|
| 0 | FirstCard | 工具安装、参数、变量、格式化；第一份个人任务卡 | [新手指南](BEGINNER_WINDOWS_GUIDE.md) |
| 1 | UnitLab → QuizDeck → TextScope | 函数、enum、Result、集合、所有权、借用、迭代器 | [基础路线](FOUNDATION_PATH.md) |
| 2 | LedgerLite → BadgeForge → TermGarden | 错误传播、状态、trait/泛型、按行交互 | [基础后续](FOUNDATION_NEXT_STEPS.md) |
| 3 | BatchMeter → MiniKV → NetFetch → ServiceDesk | 线程、消息传递、持久化、同步TCP/HTTP | [系统应用](SYSTEMS_PATH.md) |
| 4 | StoreDesk → AsyncBoard | 系统SQLite事务/回滚、Future/Waker与协作式等待 | [事务与等待](RELIABILITY_PATH.md) |
| 5 | TaskPocket / PageMint | 自主拆分需求、数据与接口、自己的测试和交付 | [独立作品](INDEPENDENT_PATH.md) |
| A | TermGarden Workbench | 共享/弱引用、闭包、快照、Send/Sync与接口设计 | [进阶A](ADVANCED_A.md) |
| B1–B3 | TaskPocketPro | taskcore业务库、多crate Workspace、CLI/报告复用、feature合同 | [进阶B](ADVANCED_B.md) |
| B4 | JSON Export | 固定锁、Serde derive、DTO、只读导出、四阶段累计检查 | [B4 Windows步骤](ADVANCED_B_WINDOWS.md) |
| C1 | 异步下载队列 | Tokio/Reqwest、并发上限、超时、取消与暂存发布 | [C1](ADVANCED_C1.md) |
| C2 | 异步工单服务 | Axum路由/JSON/共享状态、单写者日志、并发客户端与优雅关闭 | [C2](ADVANCED_C2.md) |

基础阶段同时保留FileQuest、LogLens、LocalHub、SafeSync与32章原有应用教程，章节入口见 [课程首页](README.md)。对照阅读见 [Rust Book指南](BOOK_GUIDE.md)。

## Rust Book 内容完整性补课（独立于 0.19.0-rc.3 课程范围）

[逐节审计矩阵](BOOK_COVERAGE_AUDIT.md) 以中文版 Rust Book 第 1–20 章及附录 A–G 为基准，区分已有教学、关联实作、待补缺口与新课待 CI；不以关键词出现作为教学完成证明。

**D1 Cargo 进阶**：[完整讲义](BOOK_D1_CARGO.md) 与 [独立实验](../projects/book-d1-cargo/README.md) 覆盖构建 profile、打包检查、Workspace、专用安装根与自定义 Cargo 子命令。

**D2 智能指针**：[六节讲义](BOOK_D2_SMART_POINTERS.md) 与 [独立实验](../projects/book-d2-smart-pointers/README.md) 覆盖 Box、Deref、Drop、Rc、RefCell 和 Weak/引用循环。

**D3 面向对象与模式匹配**：[讲义](BOOK_D3_OOP_PATTERNS.md) 与 [独立实验](../projects/book-d3-oop-patterns/README.md) 覆盖封装、trait object、状态模式，以及 18.1–18.3 的模式语法与可反驳性。D1/D2/D3 已随 PR #6 合并并通过跨平台验收，作为 RC 之外的独立补充 Workspace。

**D4 高级特性**：[讲义](BOOK_D4_ADVANCED_FEATURES.md) 与 [独立实验](../projects/book-d4-advanced-features/README.md) 覆盖 unsafe Rust、高级 Trait/类型/函数与三类过程宏。实验使用 Rust 1.74、零第三方依赖，并把不安全操作限制在有安全契约的局部代码中；D4-A2 已通过 17/17 CI Jobs，并随 PR #6 合并。

**D5 多线程 Web Server**：[讲义](BOOK_D5_WEB_SERVER.md) 与 [独立实验](../projects/book-d5-webserver/README.md) 覆盖 TCP/HTTP 请求响应、固定大小 Worker 线程池、MPSC 消息队列和 Drop 优雅清理。独立 Workspace 使用 Rust 1.74、零第三方依赖；PR #8 已合并，Run 197 的 Linux/Windows、MSRV、交付矩阵通过。

**D6-A 附录参考与实践**：[讲义](BOOK_D6_APPENDICES.md) 与 [独立参考 Workspace](../projects/book-d6-appendices/README.md) 覆盖关键字、运算符与符号、可派生 Trait、现代开发工具、Edition、翻译版本和 Stable/Beta/Nightly。实验使用 Edition 2021、Rust 1.74、零第三方依赖；关键字清单与运算符优先级已加入精确回归测试。

**D6-B 基础知识缺口**：[讲义](BOOK_D6_FOUNDATIONS.md) 与 [独立参考 Workspace](../projects/book-d6-foundations/README.md) 已覆盖 12 项基础知识缺口；14 项单元测试、6 项 CLI 测试、2 项 Doctest 均通过 Linux、Windows、Rust 1.74 的 Debug/Release 验证。Run 215 候选包包含 D6-A/B 全部材料，原有 17 个学员工程保持不变。

**D6-B 后续概念实操**：[所有权与切片](BOOK_OWNERSHIP_PRACTICE.md)、[集合与错误传播](BOOK_COLLECTIONS_ERRORS.md)、[泛型与生命周期](BOOK_GENERICS_LIFETIMES.md) 扩展同一个参考工作区。当前本地 Rust 1.74 / 1.85 各通过 48 项 Debug、48 项 Release 和 76 项 Doctest；跨平台验收以当前提交 CI 为准。[概念审计](BOOK_CONCEPT_AUDIT.md) 仅量化本次第 4–6、8–10 章的 118 个目标，不代表全书或个人学习达标。

## 当前材料数量与性质

根Cargo工作区17个包：FileQuest、LogLens、LocalHub、SafeSync、LabQuest、UnitLab、QuizDeck、TextScope、LedgerLite、BadgeForge、BatchMeter、TermGarden、StoreDesk、MiniKV、NetFetch、ServiceDesk、AsyncBoard。LabQuest是学习平台，不冒充学生作品。

12款Studio合计36张任务卡、108条逐级提示，提供参考实现、学员核心TODO、可见测试和固定平台验收。TaskPocket/PageMint提供独立工程骨架、需求合同和评测，不把教师夹具算作学员已完成作品。进阶A/B和C1/C2使用独立学员工程，不覆盖基础作品。

## 概念与实际边界

| 主题 | 当前材料 | 边界 |
|---|---|---|
| 所有权/借用/生命周期 | TextScope、TermGarden、进阶A | 通过具体输入与编译诊断解释，不只看通过数 |
| 错误处理/trait/泛型 | LedgerLite、BadgeForge、MiniKV、进阶B | 自己补测试，按公开接口验证 |
| 并行与异步 | BatchMeter线程；AsyncBoard教学执行器；C1/C2 Tokio | 不把同步网络、Future教学或线程并行混称同一机制 |
| 存储 | TaskPocket/MiniKV教学日志；StoreDesk系统SQLite；C2沿用ServiceDesk日志 | 无生产灾备、断电原子性或多进程共享写入保证 |
| 网络 | 原同步NetFetch/ServiceDesk与独立C1/C2真实异步实现 | 只做可信本机回环HTTP；无公网/TLS/账号体系 |
| 界面 | 按行终端、HTML/SVG和工单网页 | raw-mode TUI、GUI、WASM不在本轮范围 |

## 学习方法与依赖

先预测→改一个函数→真实检查→个人输入→解释原因。新建Studio工程附TASK_CARDS.html和Markdown；已有工程用learn lesson/hint/check，不必重建。使用学习总览时，旧成绩仅用于导航，修改源码后必须重跑。

基础/A/B最低Rust1.74；B4首次显式准备固定Serde依赖。C1/C2要求Rust1.85.0并有独立vendor包；StoreDesk另需系统SQLite，见 [SQLite说明](SQLITE_SETUP.md)。普通学员不需要Python/Playwright。当前目标平台Windows x64 MSVC、Linux x86_64，不声明macOS。

## 本轮之外

ChatDock、DeskNotes与WasmBoard保留为方向选修，不列为当前候选发布欠账。C2已覆盖持久化、并发验证和优雅关闭，不再为相同内容另建C3。后续从实际试用问题出发，先修复阻断和教学衔接，再讨论新的学习目标。

**按需基础复习**：若在主线课程中分不清 Cargo 操作，可补做 D6-B 的 Cargo 三步复习；数组、元组和 [变量与模块专项](BOOK_D6_LANGUAGE_BASICS.md) 也按实际问题选读。它们不是进入 UnitLab 前的必修清单。工具链、Edition 迁移与 Nightly 的可执行流程见 [D/E/G 实操](BOOK_D6_PRACTICAL.md)。这些参考实验不增加学员作品数量，也不自动填写个人掌握状态。
