# Rust AppLab · 0.19.0-rc.6 候选发布版

通过亲手制作小软件学习 Rust：先读懂输入和输出，每次实现一个函数，用自己的数据验证，最后完成独立作品。

**推荐从网页工作台开始。** Windows 双击 `START_WORKBENCH_WINDOWS.cmd`，Linux 执行 `./START_WORKBENCH.sh`。在一个浏览器窗口内读中文任务卡、编辑自己的源码、保存、运行和查看真实检查结果；默认展示当前课程及后续两课，点击「开始本课」即可进入；已有 33 个网页课程入口（13 个作品、7 个 Book 专题、13 个现代 Rust host 课程）。部分进阶课需要另行准备对应工具链和离线学员包。安装、续学和操作说明见 [网页学习工作台](course/WORKBENCH.md)。

## 我现在应该做什么？

| 你现在的位置 | 现在只做这一件事 | 做完应得到什么 |
|---|---|---|
| 还没有运行过 Rust | 按 [开始指南](README_START_HERE.md) 准备环境，打开网页工作台并创建 FirstCard | 能随姓名和任务变化的个人任务卡 |
| **FirstCard 已经通过，不知道怎么继续** | **打开 [FirstCard 完成后的下一步](course/AFTER_FIRSTCARD.md)**；复盘后创建 UnitLab | 明确自己练到了什么，并开始第一个计算函数 |
| 已有 `my-unitlab` 等工程，今天想接着学 | 工作台选对应课程，输入实际路径并点击“打开项目” | 找到当前任务、自己的源码和真实检查结果 |
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

## 下载源码后，哪些文件是给我看的？

| 你要做的事 | 从这里进入 |
|---|---|
| 开始、续学、查看当前任务 | [开始指南](README_START_HERE.md)、[FirstCard 之后](course/AFTER_FIRSTCARD.md)、[课程目录](course/README.md) |
| 看参考源码和实验材料 | 各 `apps/`、`projects/`、`examples/` 的 README；自己的代码在 `my-...` 工程 |
| 维护项目或打包 | [维护文档与源码地图](docs/README.md) |

按当前作品的 README 继续即可。维护文档不是学员必读材料。

## 选择下载内容

网页工作台随 **0.19.0-rc.6** 源码和对应平台运行包交付。请从 [Releases](https://github.com/clonetimer/RustAppLab/releases) 选择该版本及其校验文件；发布资格以同一提交的 CI 与实际 Release 为准。也可下载 [当前 main 源码 ZIP](https://github.com/clonetimer/RustAppLab/archive/refs/heads/main.zip)。已发布的 rc.5 及更早的包保持原样，不会自动获得新界面；rc.6 的下载以实际 Release 为准。

更新时解压到新的平台文件夹，保留原来的 `my-...` 工程，再从工作台打开其实际路径。

| 目的 | 下载包 | 需要什么 |
|---|---|---|
| 直接打开工作台、体验成品 | `RustAppLab-0.19.0-rc.6-<平台>-runtime.zip` | 对应系统；工作台与参考程序已编译，运行或检查自己的 Rust 仍需 Rust/Cargo |
| 使用完整源码与课程 | `RustAppLab-0.19.0-rc.6-source.zip` | Rust/Cargo 与系统构建工具；首次打开会编译本地工作台 |
| 只取学员骨架 | `RustAppLab-0.19.0-rc.6-learner-kit.zip` | 17 个基础/进阶 A/B 工程；核心 TODO 待完成，不含 C1/C2 vendor |
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

新版工作台可以搜索中文概念、Rust 名称或错误编号，打开本地讲义与官方章节，再进入对应专题的个人副本。D1–D6 支持同页编辑、保存与真实 Cargo 测试；11 类编译错误提供隔离的失败／修复对照实验。中文 Rust Book 用于基础概念与项目组织对照；异步 API 按锁定版本的官方文档学习。使用 `learn reading <工程>` 查阅相关章节，见 [Book 对照指南](course/BOOK_GUIDE.md)。

遇到相应概念时，可补读 [所有权与切片](course/BOOK_OWNERSHIP_PRACTICE.md)、[集合与错误传播](course/BOOK_COLLECTIONS_ERRORS.md)、[泛型与生命周期](course/BOOK_GENERICS_LIFETIMES.md)，或选读 [31 项概念补课任务](course/BOOK_31_GAPS_REPAIR.md)。它们不要求在 UnitLab 前完成。

## 已有工程继续用

网页工作台内选择课程，再输入旧工程的完整路径。课程卡来自平台内置材料，即使旧工程缺少新生成的讲义也能继续学习；已有源码不会被重新初始化。终端方式仍可使用：

在完整源码包的根目录运行，不需要重建旧工程：

```powershell
cargo run --locked --offline -p labquest -- learn next "D:/练习/my-unitlab"
cargo run --locked --offline -p labquest -- learn lesson "D:/练习/my-unitlab"
cargo run --locked --offline -p labquest -- learn check "D:/练习/my-unitlab"
```

把示例路径替换成自己已有工程的实际路径。Windows 可用 `LEARN_NEXT_WINDOWS.cmd`。`start` 只创建新工程，不用它来继续已有作品。源码修改后重新检查，读取历史进度不会执行学员代码。

## 候选版范围与试用

本次收口以教学工作台 R1–R6 为主，不新增课程。原有 B4、C1、C2、D1–D6 专题与现代 Rust host 课程保留；独立作品和进阶 A/B/C 保留原入口。五门现代依赖课程需从对应专项验收产物准备离线学员包；GUI、ARM/QEMU、C/WASM、工具链迁移和 IDE 等专项能力按各自证据判定，网页 host 测试不替代它们。

支持范围为 Windows x64 MSVC、Linux x86_64；基础/A/B 的最低 Rust 为 1.74，C1/C2 单独要求 1.85.0。StoreDesk 需系统 SQLite，见 [支持说明](SUPPORTED_PLATFORMS.md)。普通学员不需要 Python 或 Playwright，不运行维护者的发布验收脚本。

此版本为 **RC 候选版，不是 Stable**。当前源码必须重新通过 Windows/Linux、最低版本、浏览器、C1/C2 和交付检查；旧版本证据不继承。真人学习效果与 Win10 本机体验按 [试用记录](course/USER_TRIAL.md) 记录，不能以 CI 代填。参见 [发布规则](RELEASE_POLICY.md)、[已知边界](KNOWN_ISSUES.md)、[后续范围](ROADMAP.md)。

网络服务仅用于可信本机回环 HTTP；不提供公网/TLS、账号认证或恶意代码沙箱。当前修订见 [变更记录](RELEASE_NOTES.md)。
