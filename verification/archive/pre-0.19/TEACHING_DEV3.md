# Rust AppLab 0.18.0-dev.3 · 教学开发版实际验证记录

日期：2026-10-30。执行环境：Linux x86_64、rustc/cargo 1.99.1，离线 Cargo。**本版本是 0.18 独立教学开发分支，不继承冻结的 0.17.0-rc.2 / RC_READY。** 支持声明为 Windows x64 MSVC、Linux x86_64，不声明 macOS。

## 本轮源码改造

1. LedgerLite、BadgeForge、TermGarden 增加 9 张具体函数微任务卡、27 条层级提示，含预测、错误示例、用自己的数据运行、复盘；三款新生成工程均附带离线 TASK_CARDS.md。
2. `labquest learn overview <练习父目录>` 扫描直接子目录与 `Studio/` 两层布局，区分未检查、失败、旧报告过期、历史三步通过，不执行学员代码也不创建报告。对可编辑历史报告不得宣称认证通过。
3. Windows `LEARN_NEXT_WINDOWS.cmd` 的 PowerShell 菜单增加 P，显示所选工程同级的进度总览。未在 Windows PowerShell 原生运行，只有静态脚本测试。
4. 新增 `course/FOUNDATION_NEXT_STEPS.md` 与 `course/PROGRESS_OVERVIEW.md`，以及连续学习文档入口。
5. 原有 17 个 Cargo Package、12 款 Studio 工程与分步评测合同未修改，全部原有学员作品可继续使用。

## 实际检查结果

| 检查 | 本轮结果 |
|---|---|
| Rustfmt --check | PASS |
| Debug workspace Clippy --all-targets -D warnings | PASS |
| Release workspace Clippy --all-targets -D warnings | PASS |
| Debug workspace 全部测试目标编译 --no-run | PASS |
| 16 个非 LabQuest 包 Debug all-targets 测试 | 203 PASS，0 FAIL；15 项学员挑战按设计 ignored |
| LabQuest 库 Debug 测试 | 45 PASS，0 FAIL（包含 3 项新增教学回归） |
| LabQuest 库 Release 测试 | 45 PASS，0 FAIL |
| Workspace 文档测试 | 2 PASS，0 FAIL |
| 脚本 Python unittest | 21 PASS，0 FAIL（Windows 入口仅静态验证） |
| 全 Workspace Release 构建 | PASS，17 程序 |
| 三款学员工程各阶段红绿试验 | 3/3 套 PASS：未实现 step1 失败，正确改动后 step1 两项通过，step2 未实现仍失败 |
| LedgerLite 真实 LabQuest 平台检查 | 第 1 步 PASS；创建历史报告，随后 `learn overview` 显示已记录 1/1、下一步 2 |
| 中文与空格路径多作品总览 | PASS，三款工程均识别，阅读不修改源码 |

**未完成，不可误报：**单次全 Workspace `cargo test --all-targets` 因执行时间上限中断；LabQuest `learning_e2e` 6 项中有 3 项已在日志中完成，其余未取得结论。故本轮不宣称全仓 Debug/Release 一次性通过，也不宣称整个 Studio E2E 回归完成。冻结的 RC 证据仍归属于旧版源码。

## 原始证据和用户边界

见随包 evidence ZIP 的 `*-tests.log`、`*-build.log`、`red-green.json`、`platform-check.txt`、`overview-demo.txt`。学生修改答案后参加测试的演练使用维护者参考代码，不是自然学员的教学通过率证据。学员工程测试只是验证教学合同可以分步执行。无答案学生 ZIP 不包含参考函数实现。

下一轮应继续补齐 BatchMeter、MiniKV、网络阶段的分层引导并取得 Windows 0.18 新分支原生复验及真实学员过程记录，而不以测试用例总数替代教学效果。
