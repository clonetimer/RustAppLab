# Rust AppLab 0.19.0-rc.1 · 当前课程地图

本轮功能范围冻结在C2；先完成当前作品，再进入下一阶段。自动检查验证程序行为，不代替真人理解。参考程序、学员骨架和独立开发合同分别标注，不按目录数量计算学习成效。

## 推荐顺序与入口

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
