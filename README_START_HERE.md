> 已完成基础与进阶 A 的学员：见 `ADVANCED_B.html`，可双击 `START_ADVANCED_B_WINDOWS.cmd` 开始多程序复用课程。

## 按当前工程查阅 Rust Book

已有学员工程时，在平台根目录执行 `cargo run --locked --offline -p labquest -- learn reading "my-unitlab"`。按当前作品查看中文概念、官方资料与思考实验，阅读后回到原有检查命令。详见 [对照学习指南](course/BOOK_GUIDE.md)。

# 已完成基础课程的学员

0.19.0-dev.1开始进阶A。直接打开 [ADVANCED_A.html](ADVANCED_A.html)，或Windows双击 `START_ADVANCED_WINDOWS.cmd`，生成独立的my-workbench。已有基础工程不用覆盖。旧续学菜单同样识别新工程。

没有完成基础课程的学员继续下面的原入口。

---

# 从这里开始 · 0.18.0-dev.6

正式支持范围是 Windows x64 MSVC 和 Linux x86_64；不声明 macOS。普通学员不运行 VERIFY_RELEASE、VERIFY_BROWSER 或 MSRV 验收入口。

## 你现在只需要选一条路

| 现在的情况 | 做什么 |
|---|---|
| 从没写过 Rust | 打开 `START_HERE.html`，按说明安装 Rust，再双击 `START_WINDOWS.cmd` 完成第一张任务卡 |
| 任务卡做完了，还没有 UnitLab | 双击 `CONTINUE_WINDOWS.cmd`，按 Enter，创建 UnitLab |
| 已经有自己的练习目录 | 双击 `LEARN_NEXT_WINDOWS.cmd`，输入该目录；旧工程不用重建，也不用删除 |
| 只下载了学员练习包 | 先读其中 `START_HERE.md`；每个工程可以独立使用 Cargo，不要求平台在旁边 |

当前学习导航的任务卡与真实检查支持 FirstCard 和十二款 Studio 工程。TaskPocket、PageMint 仍使用原来的独立作品合同与 `app check`；`learn next` 会告诉你对应命令。

## 已有工程：一次只做一个函数

在 `LEARN_NEXT_WINDOWS.cmd` 中输入 `my-unitlab`，或者输入旧版练习的完整路径，例如 `D:\学习资料\my-unitlab`。

按 **L** 看当前任务卡，按 **E** 打开自己的源码，只修改任务卡指定函数。按 **Ctrl+S** 保存，返回窗口按 **C** 真正编译和检查。需要帮助时，按 **1 / 2 / 3** 查看逐级提示；按 **S** 可指定检查第 1、2、3 步或 all。Q 只关闭向导，源码和报告都保留。

新源码和旧检查记录不一致时，导航不会宣称通过；实际检查会同时回归前面步骤。例如正在做第 3 步，前两步也会重新运行，改坏第 1 步就会明确停在第 1 步。

## 不要弄混两个文件夹

**平台根目录**包含 `apps`、`course` 和 `LEARN_NEXT_WINDOWS.cmd`。下面这些命令在平台根目录执行：

```powershell
cargo run --locked --offline -p labquest -- learn lesson "my-unitlab"
cargo run --locked --offline -p labquest -- learn hint "my-unitlab" 1
cargo run --locked --offline -p labquest -- learn check "my-unitlab"
```

**你的学员工程**包含自己的 `Cargo.toml`、`src/lib.rs`、`LESSON.md`。完成 UnitLab 三步后，在该目录运行你自己的软件：

```powershell
cargo run --offline -- convert -10 c f
```

预期：`-10.00 c = 14.00 f`。在只完成第一步时，完整 CLI 还不可用，不要因为它失败就认为第一步没学会。

## 学完一款以后

FirstCard → UnitLab → QuizDeck → TextScope。完成每一款后，用自己的输入运行一次、保存实际输出、写下用过的提示，再进入下一款。不要跳到线程或数据库来避开当前不理解的概念。

完整的衔接与真实作品练习见 [基础路线](course/FOUNDATION_PATH.md)。命令与常见问题见 [学习导航](course/LEARNING_NAVIGATOR.md)。

Windows 新版菜单尚待原生实测；之前冻结 RC 的 Windows 通过结果不自动证明这次新增脚本已经通过。


### 进度与续学（0.18.0-dev.6）

完成 UnitLab、QuizDeck、TextScope 后，建议进入 `course/FOUNDATION_NEXT_STEPS.md` 的 LedgerLite、BadgeForge 和 TermGarden。三个都有每一步具体函数、预测问题和三级提示。

多个练习工程散落在一个文件夹时，可运行 `cargo run --offline -p labquest -- learn overview "<练习父目录>"`；Windows 续学入口 `LEARN_NEXT_WINDOWS.cmd` 按 P 查看同级进度。这里只读取历史报告和源码摘要，不会启动测试，历史通过不能代替真实功能检查。


### 0.18.0-dev.6 · 第四段衔接

BatchMeter → MiniKV → NetFetch → ServiceDesk 的系统应用路线继续保留；StoreDesk、AsyncBoard 本轮补齐 6 张微任务卡、18 条分层提示，总覆盖为 12 款/36 张/108 条。完整路径见 [course/SYSTEMS_PATH.md](course/SYSTEMS_PATH.md)。新建工程有 TASK_CARDS.md，旧工程可直接使用新版 learn lesson/hint，无需覆盖。StoreDesk/AsyncBoard 的引导已补齐，事务与等待实验见 RELIABILITY_PATH.html；成熟异步网络、桌面和 WASM 仍未完成。


新版十二款任务卡同时生成可双击的 `TASK_CARDS.html`，提示默认折叠，可逐级查看；Markdown 版仍保留。联合讲义可直接打开根目录 `SYSTEMS_PATH.html`，不需要 Markdown 阅读器或网页服务。已有练习不必重建，继续使用新版命令行导航即可。

本轮具体入口：[第四段：事务与等待](RELIABILITY_PATH.html)。新手不跳级；StoreDesk 先修 LedgerLite，AsyncBoard 先理解 BatchMeter。后续由 TaskPocket/PageMint 检验独立开发，不把 36 个 TODO 完成自动认定为掌握整个 Rust 生态。


## 完成分步作品之后

打开 `INDEPENDENT_PATH.html`。第一次独立开发建议先 TaskPocket 再 PageMint；新工程先读 START.html，按只读→业务→完整合同推进。仍使用 LEARN_NEXT_WINDOWS.cmd 的 L/E/C/S/G 操作。不要把 LabQuest 平台根目录当成学员目录。旧工程无新文件时读平台页面即可；不会自动覆盖源码。
