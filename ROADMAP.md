## C2 Axum · 新增异步工单课程

独立 Axum/Tokio 工作区、固定 ServiceDesk 业务日志、三阶段学员接口与回归测试。见 [C2 指南](course/ADVANCED_C2.md)；当前提交的实际支持范围以 CI 结果为准。原同步版本、C1 和基础 Rust1.74 保留。

## 0.19.0-dev.2 进阶 B 的实际范围

B1–B4 覆盖多 crate Workspace、业务库/CLI/报告分层、feature 与真实 Serde JSON 导出。B4 显式准备固定依赖，之后离线累计检查；当前源码跨平台结果以 CI 为准。C1 下载和 C2 Axum 工单服务已加入独立课程，具体测试结果见当前 CI。见 [进阶 B](course/ADVANCED_B.md)。

# Rust AppLab 路线 · 0.19.0-dev.2

## 已完成的基础阶段

保留12款Studio、36张任务卡、108条提示、FirstCard和TaskPocket/PageMint独立开发。用户已报告学员全部通过，不要求重复基础关卡。

## 共同进阶主线

| 单元 | 作品载体 | 状态 |
|---|---|---|
| A 所有权和接口设计 | TermGarden工作区：共享视图、闭包批次、撤销、快照报告 | 本轮交付参考软件、独立学员工程、三阶段检查和设计评审 |
| B 成熟生态和库工程 | TaskPocket：业务库、两个程序复用、feature、Serde JSON 适配工作区 | B1–B4：固定依赖准备、只读 JSON 导出与累计检查；当前源码平台验证见 CI |
| C 真实异步应用 | NetFetch/ServiceDesk：Tokio/Reqwest 下载与 Axum 工单服务 | C1 已合并；C2 已实现并进入 Windows/Linux 完整验收，限本机回环 HTTP；公网/TLS 不在本阶段范围 |

A完成后进入B和C。已有作品通过 [Book 对照阅读](course/BOOK_GUIDE.md) 衔接语言概念，B4 通过显式准备衔接第三方依赖；后续优先推进 C。GUI/WASM/嵌入式为方向选修。

## 本轮范围

A1：Rc/RefCell/Weak、借用守卫、快照引用；A2：Fn/FnMut/FnOnce、重入版本冲突、草稿和撤销；A3：Send/Sync、不可变快照、trait对象与泛型。额外保留编译失败对照和实际强引用环释放实验。

不声称提供全屏GUI、数据库事务、跨进程共享、原子报告目录、永久撤销日志或线程取消。已有旧RC保持冻结。当前版本面向Windows x64 MSVC、Linux x86_64；不声明macOS。程序通过与学员能解释设计分别记录。
