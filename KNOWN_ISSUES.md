# 0.19.0-dev.2 当前范围与后续事项

更新：2026-11-02。工程测试结果见 [本轮测试记录](verification/2026-11-02-book-guided-quality.md)，历史变更见 [RELEASE_NOTES.md](RELEASE_NOTES.md)。

## 已交付的当前范围

- 基础教学：十二款 Studio、36 张任务卡、108 条逐级提示；FirstCard 与 TaskPocket/PageMint 独立作品路线。
- 进阶 A：TermGarden 的共享句柄、弱引用、闭包、快照与接口设计。
- 进阶 B1–B3：业务库、多 crate Workspace、CLI 与报告程序、默认/strict-titles 两种配置的累计检查。
- Rust Book 对照阅读：按现有工程显示相关章节和思考实验；新版生成任务卡附同源参考段。

本轮修复了 Windows 编译实验 UTF-8 解码、进阶 B list/feature 漏验和 B1..B0 导航文案，并将独立工作区及教材输入纳入工程验证。

## 后续工作与使用范围

| 项目 | 当前情况与下一步 |
|---|---|
| B4 第三方依赖 | Serde/serde_json 课程待实现，需依赖锁定、序列化练习和 Windows/Linux 实测 |
| C 真实异步网络 | Tokio/Reqwest/Axum 课程待完成；现有 NetFetch/ServiceDesk 为同步本机网络，AsyncBoard 是教学型 Future 执行器 |
| GUI / WASM | 仍为方向选修规划 |
| 学习效果 | 自动检查提供程序行为证据；零基础学员的连续独立完成情况需实际观察 |
| 学习记录 | 本地记录用于导航；源码改变后需重新运行当前检查 |
| 存储 | StoreDesk 依赖系统 SQLite；TaskPocket/MiniKV 等教学存储的持久化能力按各自讲义说明使用 |
| 平台 | 正式支持 Windows x64 MSVC、Linux x86_64；macOS 未纳入当前支持范围 |

本版本保持开发版。冻结 RC 0.17.0-rc.2 的资格仅对应当时源码；当前工程结果、真人教学观察和交付物检查分别记录。
