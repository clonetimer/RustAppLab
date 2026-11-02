## 0.19.0-dev.2 进阶 B 的实际范围

已完成 B1–B3：多 crate Workspace、业务库/CLI/报告分层、特性开关与黑盒检查。B4 Serde/serde_json、Tokio/Axum 异步网络未能在离线环境中编译验证，仍为待办。见 `course/ADVANCED_B.md`。

# Rust AppLab 路线 · 0.19.0-dev.2

## 已完成的基础阶段

保留12款Studio、36张任务卡、108条提示、FirstCard和TaskPocket/PageMint独立开发。用户已报告学员全部通过，不要求重复基础关卡。

## 共同进阶主线

| 单元 | 作品载体 | 状态 |
|---|---|---|
| A 所有权和接口设计 | TermGarden工作区：共享视图、闭包批次、撤销、快照报告 | 本轮交付参考软件、独立学员工程、三阶段检查和设计评审 |
| B 成熟生态和库工程 | TaskPocket：业务库、两个程序复用、功能开关；后续第三方依赖与序列化 | B1–B3 已交付并补齐累计/feature 检查；B4 待完成 |
| C 真实异步应用 | NetFetch/ServiceDesk：成熟运行时、有限并发、取消、正常退出 | 未交付；同步版和教学执行器不能替代 |

A完成后进入B和C。已有作品通过 [Book 对照阅读](course/BOOK_GUIDE.md) 衔接语言概念，后续优先推进 B4 与 C。GUI/WASM/嵌入式为方向选修。

## 本轮范围

A1：Rc/RefCell/Weak、借用守卫、快照引用；A2：Fn/FnMut/FnOnce、重入版本冲突、草稿和撤销；A3：Send/Sync、不可变快照、trait对象与泛型。额外保留编译失败对照和实际强引用环释放实验。

不声称提供全屏GUI、数据库事务、跨进程共享、原子报告目录、永久撤销日志或线程取消。已有旧RC保持冻结。当前版本面向Windows x64 MSVC、Linux x86_64；不声明macOS。程序通过与学员能解释设计分别记录。
