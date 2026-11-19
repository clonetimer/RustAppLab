# LabQuest · 创建练习、查看下一步、检查自己的代码

LabQuest 是 Rust AppLab 的学习控制台：它创建 FirstCard、Studio 和独立作品工程，显示任务卡与提示，并在你主动检查时编译、运行学员代码。**运行 LabQuest 不等于你已经实现这些软件；成果在你自己的学员工程里。**

## 刚完成 FirstCard：现在做哪一步？

先读 [FirstCard → UnitLab 衔接页](../../course/AFTER_FIRSTCARD.md)，回顾自己改过的变量与输出，再开始函数返回值。所有下面带 `-p labquest` 的命令都在 **RustAppLab 根目录**运行。

如果还没有 `my-unitlab/`，执行创建；已有工程则从 `learn next` 续学：

```powershell
cargo run --offline -p labquest -- studio start unitlab my-unitlab
cargo run --offline -p labquest -- learn next my-unitlab
cargo run --offline -p labquest -- learn lesson my-unitlab 1
cargo run --offline -p labquest -- learn check my-unitlab 1
```

首次检查因为 TODO 未实现而 FAIL 是正常的。打开 `my-unitlab/START.html` 或 `TASK_CARDS.html`，只修改 `my-unitlab/src/lib.rs` 的 TODO(1)，保存后重跑检查。详细操作和预期输出见 [UnitLab README](../unitlab/README.md)。

三个函数依次是 `celsius_to_fahrenheit`、`convert`、`describe`：从计算一个值，到区分单位和错误，再连接文字输入与输出。不要把参考应用 `apps/unitlab/src/lib.rs` 当作自己的练习文件。

## 尚未开始 FirstCard 时

先打开根目录 `START_HERE.html`；Windows 可双击 `START_WINDOWS.cmd`。也可在 **平台根目录**创建：

```powershell
cargo run --offline -p labquest -- beginner start my-first-card
cargo run --offline -p labquest -- beginner hint 1
```

按 `my-first-card/跟着做.md`，编辑 `my-first-card/src/main.rs` 唯一需要修改的 `println!` 行并保存，然后运行：

```powershell
cargo run --offline -p labquest -- beginner check my-first-card
```

四组输入通过后生成 `FIRST_WIN.md`。还要进入 **学员目录**用两组自己的名字和任务实际运行：

```powershell
cd my-first-card
cargo run --offline -- "小林" "完成一次单位换算"
cargo run --offline -- "阿青" "解释一条编译错误"
```

两张任务卡都应使用对应的名字与任务。保存文本，解释“输入变了，输出哪两处跟着变”，再返回平台根目录开始 UnitLab；测试通过不是理解程度的认证。

## 学习顺序：一次只做当前作品

| 阶段 | 推荐作品 | 从这一段带走什么 |
|---|---|---|
| 从一行输出到函数 | FirstCard → UnitLab | 变量、参数、返回值、`match`、`Result` |
| 文本与状态 | QuizDeck → TextScope → LedgerLite → BadgeForge → TermGarden | 集合、所有权、借用、trait、事件与状态 |
| 并发与本机网络 | BatchMeter → MiniKV → NetFetch → ServiceDesk | 线程、迭代器、持久化、HTTP 客户端和服务端 |
| 失败恢复与等待 | StoreDesk → AsyncBoard | 参数绑定、事务、Future 与 `.await` |
| 独立开发 | TaskPocket → PageMint | 按合同组织程序、自写测试、个人数据与发布说明 |

FileQuest、LogLens、LocalHub、SafeSync 的第 00–23 章是参考工具拓展；CASE 01 是综合调查任务。它们不是完成 FirstCard 后必须先过的门槛。路线来自现有导航，可用 `cargo run --offline -p labquest -- learn route` 随时查看。

## 在一个 Studio 工程里怎样持续学习？

以已经创建的 `my-unitlab` 为例，以下仍在 **平台根目录**运行；换作品时把目录名换成你实际的学员目录。

| 完整命令 | 什么时候用 |
|---|---|
| `cargo run --offline -p labquest -- learn next my-unitlab` | 忘记做到哪里，查看当前源码与最近记录的关系 |
| `cargo run --offline -p labquest -- learn lesson my-unitlab 2` | 读取第 2 步的目标、函数、预测与个人实验 |
| `cargo run --offline -p labquest -- learn hint my-unitlab 1 2` | 获取第 2 步的第一层提示，必要时再看第 2、3 层 |
| `cargo run --offline -p labquest -- learn check my-unitlab 2` | 真实编译并累计检查第 1–2 步 |
| `cargo run --offline -p labquest -- learn reading my-unitlab` | 按当前作品对照阅读已有 Rust 概念资料 |
| `cargo run --offline -p labquest -- learn overview .` | 列出当前父目录下可识别的练习项目 |

完整工程也生成 `LESSON.md`、`HINTS.md`、`TASK_CARDS.md/html` 和 `LEARNING_LOG.md`。在 **学员目录**可用 `cargo test --offline --test milestones step1_` 只检查当前一步；平台检查累计回归并使用提供的行为合同。

`studio check <目录>` 默认只检查第 1 步；`learn check <目录>` 根据最近记录建议检查范围，也可显式指定 1、2、3 或 all。修改源码后不能沿用旧报告当作新 PASS。三步通过后，换自己的输入、保留产物、加一个新测试，再解释关键概念。

## 完成基础作品后：独立实现

读根目录 [独立作品路线](../../INDEPENDENT_PATH.html)，在 **平台根目录**执行：

```powershell
cargo run --offline -p labquest -- app init taskpocket my-taskpocket
cargo run --offline -p labquest -- learn lesson my-taskpocket 1
cargo run --offline -p labquest -- learn check my-taskpocket 1
```

先读生成的 `CONTRACT.md`，在 `my-taskpocket/src/` 自己设计实现；三个累计行为检查点不规定内部函数。初始检查失败是正常的。TaskPocket 做跨进程任务持久化；完成后用 `cargo run --offline -p labquest -- app init pagemint my-pagemint` 练习多文件笔记转离线 HTML。

## 可选综合练习：CASE 01

已经熟悉四款参考工具后，在 **平台根目录**创建新案例：

```powershell
cargo run --offline -p labquest -- start my-investigation
cargo run --offline -p labquest -- next my-investigation
cargo run --offline -p labquest -- board my-investigation
```

打开 `my-investigation/deliverables/CASE_BOARD.html`。按 [第 24–27 章](../../course/24_labquest_case.md) 完成文件地图、事故报告、配置修复、独立复验、亲手写 Rust、快照封存六项；Rust 源码在案例的 `project/` 中。

`status` 只读检查证据；`next` 给下一步；`simulate` 重生成案例日志；`test-app` 编译运行案例程序；`finish` 在六项齐全后生成报告。新案例检查当前配置和 150/400/750ms 多种输入，旧版案例保留原单配置合同。案例结论只适用于教学样例。

## 执行与文件边界

创建命令要求全新目录；已有工程直接续学。`learn next/lesson/hint` 用于导航，不执行学员程序；只有显式 `check`、`test-app` 等检查才会编译执行。请只检查自己创建或审查过的代码：运行有超时限制，但它不是安全沙箱。

CASE 的 `test-app` 构建上限 120 秒、每组输入运行上限 5 秒，尝试终止直接子进程，不等于完整进程隔离。源码摘要用于发现普通改动，不提供签名认证。打包的 LabQuest 二进制也不能替代编译学员程序所需的本机 Rust/Cargo。

进阶 A、B、C 使用独立创建与准备命令，基础学习不要求现在进入。需要时查根目录进阶路线或 `cargo run --offline -p labquest -- help`；涉及依赖的 prepare 会显式联网。
