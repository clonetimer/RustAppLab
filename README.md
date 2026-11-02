# Rust AppLab · 0.19.0-dev.2 进阶单元 B（可复用软件）

## 2026-11-02：Serde JSON 导出与对照阅读

新增 [Rust Book 对照学习指南](course/BOOK_GUIDE.md)。在当前工程旁查看相关中文概念、官方资料和思考实验：

```powershell
cargo run --locked --offline -p labquest -- learn reading "my-textscope"
```

进阶 B 的累计检查覆盖 CLI `list`、独立报告程序与默认/严格标题配置；维护者检查显式运行独立参考 Workspace 和 doctest。Windows 编译错误实验改为显式 UTF-8 读取中文诊断。此前 B1–B3 的结果见 [对照阅读测试记录](verification/2026-11-02-book-guided-quality.md)。

已经完成进阶 A 的学员，现在可阅读 [进阶 B](course/ADVANCED_B.md)（[Windows 操作步骤](course/ADVANCED_B_WINDOWS.md)），双击 `START_ADVANCED_B_WINDOWS.cmd` 或运行 `cargo run --offline -p labquest -- advanced-b start my-taskpocket-pro`。

进阶 B1–B3 已实现独立的 Cargo Workspace：`taskcore` 业务库、TaskPocket CLI、Markdown 报告程序，包含真实行为测试和学员生成器。**B4 新增独立的 Serde / serde_json 只读 JSON 导出工作区、显式依赖准备和四阶段累计检查；原 B1–B3 仍保持无 registry 依赖。** 原有课程与 RC 冻结版不受影响。

## 原进阶 A 教学


基础阶段已完成的学员，直接打开 [进阶A](ADVANCED_A.html)，或双击 **START_ADVANCED_WINDOWS.cmd**。不需要重做基础课，不需要重新运行旧发布验收。

本轮扩展 TermGarden 为工作区：同线程共享句柄、非拥有视图、批量编辑与撤销、拥有型快照和后台报告。讲义对照单一控制器+ID方案，不把Rc/RefCell当万能设计。三个累计检查点覆盖所有权/借用、闭包/编辑、Send/Sync/接口分发，并要求个人设计说明和新测试。

```powershell
cargo run --locked --offline -p termgarden --example workbench -- demo my-advanced-demo
cargo run --locked --offline -p labquest -- advanced start my-workbench
cargo run --locked --offline -p labquest -- advanced check my-workbench 1
```

新学员工程独立保存，旧TermGarden和已有15个学员工程保留。主Workspace仍为17个包。进阶 B1–B3 已交付，B4 增补第三方依赖生态与 JSON 导出，C（真实异步网络）尚待完成；此版仍是教学开发版。详细教程见 [源讲义](course/ADVANCED_A.md)。

## 原有基础教学入口

通过真实软件学习 Rust：先运行成品理解用途，再分步实现自己的函数，最终交付独立可用的作品。正式支持范围是 **Windows x64 MSVC、Linux x86_64**；不声明 macOS。

上一轮已补齐 StoreDesk、AsyncBoard 的分层任务，提供“事务与等待”实验：故意让批次第二项失败，检查库存与流水回滚；用真实 Future 实验区分惰性、唤醒与协作式超时。保留原有 17 个 Cargo Package、12 款 Studio 分步作品、TaskPocket/PageMint 独立作品和综合案例，不用新应用数量代替教学深度。

## 现在从哪里开始

第一次使用请读 [开始指南](README_START_HERE.md)，双击 `START_HERE.html`。已经有练习目录的 Windows 学员双击 **LEARN_NEXT_WINDOWS.cmd**，输入旧工程目录也可以继续，不必重新创建。

在续学菜单中，L 看任务卡、E 打开自己源码、C 真实检查，1/2/3 查看分层提示。只有选择检查才执行学员代码；其他导航动作只读。

```powershell
cargo run --locked --offline -p labquest -- learn lesson "my-unitlab"
cargo run --locked --offline -p labquest -- learn hint "my-unitlab" 1
cargo run --locked --offline -p labquest -- learn check "my-unitlab"
```

目前三十六张微任务卡覆盖 UnitLab、QuizDeck、TextScope、LedgerLite、BadgeForge、TermGarden、BatchMeter、MiniKV、NetFetch、ServiceDesk、StoreDesk、AsyncBoard；每步包含编辑位置、预测、概念、逐级提示、个人输入实验和复盘。这十二款新建工程还附带离线 `TASK_CARDS.md`；已有工程直接使用新版导航，不覆盖旧源码。

## 先产出，再理解，再迁移

[基础作品连续路线](course/FOUNDATION_PATH.md) 串联自己的任务卡、换算结果、复习题库和笔记分析。练习数据在 [examples/foundation](examples/foundation/README.md)。完整作品入口是 [WORKSHOP.html](WORKSHOP.html)，课程覆盖见 [CURRICULUM](course/CURRICULUM.md)。

检查发生编译错误时，原始诊断与第一条错误解释一起显示；功能断言失败时，反馈回到当前函数。前两步通过、第三步失败会留在第三步；旧报告不证明修改后代码仍然通过，真实检查包含前面步骤的回归。更多操作见 [学习导航](course/LEARNING_NAVIGATOR.md)。

## 验证与支持边界

这是 **0.19.0-dev.2 开发版**。此前 B1–B3 的 Windows/Linux、Rust 1.74 与浏览器结果见 [对照阅读测试记录](verification/2026-11-02-book-guided-quality.md)；B4 当前源码的结果以本分支/主干 CI 为准，正式支持范围见 [SUPPORTED_PLATFORMS](SUPPORTED_PLATFORMS.md)。真实学员学习效果单独评估；冻结的 0.17.0-rc.2 / RC_READY 记录保留原版本范围。

普通学员不需要 Python、Playwright，也不运行发布验收脚本。StoreDesk 仍依赖系统 SQLite；NetFetch/ServiceDesk 为同步网络教学版，AsyncBoard 是教学型异步执行器，不等于 Tokio/Reqwest/Axum。尚未完成的成熟异步框架、桌面 GUI、WASM 课程不计入已交付能力。

教学检查只用于自己的可信源码，不是恶意程序沙箱。参考软件和自动测试结果也不能代替真实学员的理解与独立完成能力。


## 并发、存储与网络的连续学习

完成 TermGarden 后进入 [第三段：个人工单档案](course/SYSTEMS_PATH.md)。十二款作品均具备完整微任务卡（36 张、108 条提示）。联合实验把关闭前后 JSON、文件统计、持久索引和重启记录组织为真实作品，不以参考程序输出冒充独立开发成绩。


新版十二款任务卡同时生成可双击的 `TASK_CARDS.html`，提示默认折叠，可逐级查看；Markdown 版仍保留。联合讲义可直接打开根目录 `SYSTEMS_PATH.html`，不需要 Markdown 阅读器或网页服务。已有练习不必重建，继续使用新版命令行导航即可。

## 第四段：事务与等待

完成前面课程后打开 [RELIABILITY_PATH.html](RELIABILITY_PATH.html)：StoreDesk → AsyncBoard → TaskPocket 独立开发。已有 6 张任务卡、18 条提示、真实 SQLite 回滚实验、提醒报告与 `wake_lab`。样例位于 [examples/reliability](examples/reliability/README.md)；教师可用 `scripts/smoke_reliability_path.py` 复核。学生不需要 Python，旧工程不覆盖。


## B4 使用与维护者验证

新工程带 `json-export`；旧工程显式 `advanced-b add-json <目录>`，再执行 `advanced-b prepare-json <目录>` 准备固定依赖，最后 `advanced-b check <目录> 4`。`all` 和 `learn check` 累计到 B4，显式 `3` 保留原离线合同。详见 [B4 教材](course/ADVANCED_B.md)。Tokio/Axum 真实异步网络课程 C 仍待交付。

维护者先明确联网 `cargo fetch --locked --manifest-path projects/advanced-b/reference/json-export/Cargo.toml`，再运行完全离线的 `python scripts/release_check.py --output ../qualification-current`、`sh scripts/verify.sh` 或 `scripts/verify.ps1`。完整门禁包括 B4 默认/严格 Debug、Release、Clippy、doctest 和 feature 隔离的平台 E2E。主干 Windows/Linux、MSRV 与浏览器 CI 全绿后，仅创建 `checkpoint/v<VERSION>-<SHA前12位>` 轻量阶段标签；它不表示 Stable 发布或学员已通过课程。
