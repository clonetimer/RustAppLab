## 0.18.0-dev.1：新增自助导航

已有练习工程时，不需要再次 `studio start`。可在根目录运行 `cargo run --offline -p labquest -- learn next my-unitlab`，或 Windows 双击 `LEARN_NEXT_WINDOWS.cmd`。导航读取最近检查报告与本次源码摘要，提醒下一步或报告过期，但**绝不签发学员通过证明**。需要真实测试时仍运行 `studio check`。

编译错误可先保存为 UTF-8 文件，再用 `learn diagnose` 解释第一条常见错误；完整步骤见 `course/LEARNING_NAVIGATOR.md` 和 `course/RUST_ERROR_LABS.md`。

---

# Studio 使用手册：不要再把“可运行示例”当成“学会开发”

## 学员路径

第一次任务卡完成后，在 Windows 双击 `CONTINUE_WINDOWS.cmd`，按 Enter 选择 UnitLab。向导打开 `START.html` 与 `src/lib.rs`。只实现 TODO(1)，保存后按 Enter 检查；通过后进入第二步。途中退出时文件不删除，下次继续原文件。

手动命令均在 RustAppLab 根目录：

```powershell
cargo run --offline -p labquest -- studio catalog
cargo run --offline -p labquest -- studio start unitlab my-unitlab
cargo run --offline -p labquest -- studio check my-unitlab 1
```

第一阶段尚未实现时，check 返回失败是预期状态。编辑 `my-unitlab/src/lib.rs` 而不是 `apps/unitlab/src/lib.rs`；后者是参考版。每次修复第一个失败，不要同时修改三个 TODO。通过第一步后，把最后的 1 改成 2，再到 3。第 2 步会重查第 1 步，防止为了完成新功能破坏旧功能。

## 三种材料不能混淆

| 材料 | 用途 | 是否包含完整实现 |
|---|---|---|
| 完整源码 apps/unitlab 等 | 维护者回归、教师讲解、学员完成后对照 | 是 |
| studio start 生成的 my-unitlab 等 | 学员逐步实现核心能力 | 核心三个函数未完成，CLI 外壳已提供 |
| learner-kit 十二款 Studio 工程 | 离线分发、不带教师答案 | 无完整核心实现；可独立 cargo test |

学员包本身可用本地测试完成学习，不需要先编译所有平台软件。权威 `studio check` 在完整项目或对应平台 LabQuest 二进制中运行，另需本机 Rust/Cargo 编译学员源码。当前不提供 Windows 预编译 exe 包。

## 自动检查具体做什么

检查器读取 `.applab-studio` 识别作品，把本次 `src` 复制到全新的临时 Cargo 工程。它使用内置 manifest 和合同测试，不信任你可修改的 tests 或 Cargo.toml。前两步各运行两项测试；前九款最后一步再运行两项真实 CLI（共 8 项），NetFetch/ServiceDesk 最后一步分别额外覆盖五项/六项网络与 CLI 测试（总计 9 项/10 项）。

因此删掉本地 tests、把本地 Cargo.toml 改为其它内容，不会变成通过；破坏 main.rs 导致软件无法工作，也会在最后一步被发现。当前合同没有外部Rust crate。StoreDesk另需系统SQLite，参考SQLITE_SETUP；检查器会复制内置样例供CLI编译/运行，src 内最多 64 个 .rs 文件、总计 1 MiB，不复制自定义资源和 build.rs。这是本轮学习边界，不是禁止你以后使用第三方库。

每次检查创建新的 `.studio-reports/ASSESSMENT-*.md`，包含本次拷贝源码 SHA-256、命令结果与里程碑状态。旧报告不会被覆盖，也不会自动证明新代码仍正确。报告是文件快照证据，不是防作弊认证；熟练用户仍可以针对公开合同投机，理解需要人工解释与迁移任务。

编译等待上限 120 秒；每个测试阶段 8 秒；限制输出大小。超时尝试终止直接子进程，不承诺杀死全部派生进程；只评测自己信任的代码，不是恶意代码沙箱。测试中若学员程序再派生子进程，进程树清理由平台环境决定。

## 术语在这里分别指什么

Cargo.toml：软件工程的说明书。main.rs：程序入口与 CLI 连接。lib.rs：可测试、可复用的业务功能。tests：软件行为合同。target：编译产生的缓存与程序，不需要手动编辑。Result：把成功值或错误交给调用者。PASS：本次功能检查通过，不等于真人已能独立完成任何 Rust 项目。

## 教师使用方式

先运行参考软件演示产物，但不先展示全部核心源码。让学员预测一组输入，尝试当前 TODO，再运行单阶段测试。失败时先读“期待/实际”，只给第一层方向；依然卡住再打开更具体提示。最后换一份自己的数据或添加一个变体需求，观察是否仍需要照抄。

具体记录“在哪个步骤卡住、出现哪条 Rust 错误、看过哪层提示、能否口述原因、能否改成新输入”。不能凭自动测试通过数量推导教学有效性。

## Windows 验证说明

用户已反馈 0.11.1 测试通过，这记录为既有 Windows 新手启动修复的用户复验结果；没有原始全量日志，不能扩大为整个项目或 0.14.0 的 Windows 全量认证。本轮提供 `scripts/windows_studio_smoke.ps1`，在 Windows 对九款作品逐一执行初始失败与正确实现通过；CI 已接入，运行结果须另行收集。


## 两款新作品的辅助源文件

TermGarden生成storage.rs；StoreDesk生成sqlite.rs。它们先作为已提供的基础设施使用，学员三项核心任务仍位于lib.rs。检查器拷贝本次全部src，因此辅助代码也纳入源码摘要；不是在检查时悄悄用参考业务函数替换你的实现。

Stage1只改当前TODO就能通过，后两步故意未实现不会使Stage1失败。SQL链接失败则属于环境故障而非业务测试，向导和检查器提供不同指引。TermGarden的CLI样例由检查器内置复制，不依赖学生机器上的外部路径。

MiniKV基础版先修TextScope/BadgeForge/BatchMeter；提供journal.rs底层适配器，学员完成三项业务函数。依赖准备器不计作品，也不是普通学员前置。


## 0.15.0 网络基础进阶

NetFetch/ServiceDesk 在课程地图中位于基础文件与状态作品之后，Windows 续学菜单编号 10、11；无经验者仍选 UnitLab。两款新学员工程使用与前九款相同的三步合同、标记和评测流程。ServiceDesk 提供的 `http.rs` 与 `samples/index.html` 是基础设施，当前阶段主要修改 `lib.rs` 的验证、状态、JSON 函数。两款基础版使用标准库同步网络编程，不等于异步 Tokio/Axum 学习已完成。


## 0.16.0 新增第 12 款 Studio：AsyncBoard

建议先完成 BatchMeter 与同步网络基础课，再学 AsyncBoard 的 `async fn`、`Future` 与 `Waker`。学员只修改 `parse_jobs`、`render_report`、`complete` 三处，`runtime.rs` 为提供的底层设施；前两步各两项测试、第三步四项测试（含真实 CLI），总计八项。Stage3 验收会检查实际经过的时间，遗漏 `.await` 的立即返回不能通关。此机制不证明学员理解，也不是高精度性能基准。

Windows 续学菜单选 12；最初入口仍选 UnitLab。`asyncboard` 无外部 crate、无 SQLite 依赖，因此可以单独离线编译。Windows 新版还没有原生执行报告，不能继承 0.11.1 的通过结论。此阶段还不涉及 Tokio 非阻塞网络。
