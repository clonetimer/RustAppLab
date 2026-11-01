# 0.18.0-dev.2 · 教学改造与验证记录

日期：2026-10-30。当前源码为教学开发版；冻结 0.17.0-rc.2 / RC_READY 保持独立，不被覆盖。

## 本轮修改

1. 新增 `learn lesson`、`learn hint`、`learn check`，复用既有有界检查器；任务卡/提示/导航只读，只有明确 check 执行学员代码。
2. UnitLab、QuizDeck、TextScope：九张微任务卡、27 条分层提示，包含编辑位置、先预测、当前概念、自选输入实验与复盘。新建工程提供 TASK_CARDS.md；旧工程直接使用新版导航。
3. 修复累计报告 2/3 被误指回第 2 步的问题；编译失败保留尝试范围，功能失败定位第一项失败。三步全部完成后引导交付真实产物。源码改变后重新累计检查，不凭旧记录放行。
4. Studio 编译失败保留原始 stderr 并解释首条诊断和位置；功能失败回到具体函数。外部诊断支持 UTF-8、带 BOM 的 UTF-16 LE/BE、ANSI 颜色序列。
5. Windows LEARN_NEXT_WINDOWS.cmd 接入持续菜单：任务、提示、编辑、检查、重试。调用原生命令时保留正常 stderr，以退出码判断，不再次引入 PowerShell 5.1 的 NativeCommandError 误判方式。
6. 重写开始指南，清除旧版本教程堆叠造成的相互矛盾说明；新增 FOUNDATION_PATH.md 与题库/答案/笔记数据。区别平台目录和学员目录、只完成函数与完整 CLI、程序自动测试与真人学习成效。

## 已执行的验证

环境：Linux x86_64，rustc/cargo 1.99.1，使用用户提供的离线工具链。

| 项目 | 本轮结果 |
|---|---|
| 修改前 dev.1 核心库与 bin 测试 | 通过 |
| Debug 单次全 Workspace/all-targets | 309 通过，0 失败，15 项挑战忽略 |
| Release 单次全 Workspace/all-targets | 299 通过，0 失败，15 项挑战忽略 |
| 文档测试 | 2 通过 |
| 新的真实 CLI 学习流程集成 | 6 项通过，已包含于上述全仓计数 |
| Rustfmt / 严格 Clippy Debug 与 Release | 通过 |
| Workspace Release 构建 | 通过，17 个应用程序 |
| Python 发布/入口回归 | 21 通过，0 失败；Windows 脚本部分为静态检查 |
| Rust 错误实验 | 4 份代码按预期编译失败，4 份修正代码编译运行成功 |
| 基础讲义参考程序实际运行 | 10 次调用符合预期，包括三次应当拒绝的错误输入 |

完整测试日志、最终归档全新解压复验、学员包检查和二进制哈希在同版 EVIDENCE 包与外部 FRESH_VALIDATION.json 中。不把它们与真人学习记录混同。

## 支持与限制

正式范围仅 Windows x64 MSVC、Linux x86_64，不声明 macOS。新增 Windows 学习菜单尚待原生实测；新版源代码尚无 Rust 1.74 实测结果。0.17 冻结版的 MSRV/Windows 证据不自动升级为 0.18 的结果。

微任务卡本轮覆盖三款基础作品，其余九款 Studio 保留原有讲义；TaskPocket/PageMint 仍走独立作品合同和 app check。尚未交付的 Tokio/Reqwest/Axum、桌面 GUI、WASM 课程仍是缺口。没有新增真人学员的学习效果观察数据。

## 学员使用

新版源码解压到新目录，保留旧学员工程。双击 LEARN_NEXT_WINDOWS.cmd，输入旧工程的完整路径；L 读任务、E 编辑、C 检查、1/2/3 看提示。初学者仍从 START_WINDOWS.cmd 做 FirstCard，不要求先运行发布验收。
