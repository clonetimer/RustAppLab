# AsyncBoard · 同时等待，按完成顺序记录

**这次要做成什么：**读取自己的提醒清单，让任务同时开始等待，生成按实际完成顺序排列的 TSV。你将亲手使用 `async fn` 和 `.await`，运行时由项目提供。

**开始条件：**完成 [StoreDesk](../storedesk/README.md)，熟悉 `Result`、借用、数据校验与文件输出；还应能说明 BatchMeter 的线程模型。

本目录是已实现的参考软件；你的练习代码放在下面新建的 `my-asyncboard/` 中。

## 1. 看目标，创建自己的工程

在 **RustAppLab 根目录**（能看到根 `Cargo.toml`）运行：

```powershell
cargo run --offline -p asyncboard -- demo reference-async
cargo run --offline -p labquest -- studio start asyncboard my-asyncboard
cargo run --offline -p labquest -- learn lesson my-asyncboard 1
cargo run --offline -p labquest -- learn check my-asyncboard 1
```

打开 `reference-async/jobs.tsv` 与 `RESULTS.tsv`。三项配置等待 110、40、75 毫秒，通常 write、review、read 依次完成；系统调度可能改变观察次序，演示目录必须是新的。

学员首次检查因 TODO 未实现而 FAIL 是正常的。先看第一条失败；若是编译器或链接器找不到，则应先修好环境。

打开 `my-asyncboard/START.html` 或 `TASK_CARDS.html`；只编辑 **`my-asyncboard/src/lib.rs`**，每次一个 TODO。命令行外壳 `src/main.rs` 已提供，先不改。目录已存在就继续使用；新建另一份练习时换名称。

## 2. 三个检查点

| 步骤与函数 | 要完成的行为 | 本次概念 |
|---|---|---|
| TODO(1) · `parse_jobs` | 解析 ID、毫秒、说明，拒绝空清单、重复 ID 和非法边界 | `Vec<Job>`、字符串所有权、`Result` |
| TODO(2) · `render_report` | 借用完成结果，按原有顺序生成带表头的 TSV | 切片、格式化、保持结果语义 |
| TODO(3) · `complete`（`async fn`） | 真正等待 `delay.await` 后生成 Completion | Future 的惰性、`.await`、等待与阻塞 |

注意源码中 `complete` 在 `render_report` 前面，但学习顺序仍按 TODO(1)、TODO(2)、TODO(3)。先不改 `src/runtime.rs` 的执行器。

每步先写一组输入和预测，再修改、保存。第一步通过后，在 **平台根目录**继续：

```powershell
cargo run --offline -p labquest -- learn lesson my-asyncboard 2
cargo run --offline -p labquest -- learn check my-asyncboard 2
cargo run --offline -p labquest -- learn lesson my-asyncboard 3
cargo run --offline -p labquest -- learn check my-asyncboard 3
```

后一步会累计回归前面的步骤。提示用 `cargo run --offline -p labquest -- learn hint my-asyncboard 1`；进度用 `cargo run --offline -p labquest -- learn next my-asyncboard`。详细解释见 [asyncboard 讲义](../../course/studio/asyncboard.md)。

## 3. 用自己的输入完成一次作品

三步通过后，在学员工程保存 `samples/my-jobs.tsv`，使用 UTF-8 无 BOM，列之间保留实际 Tab。把说明换成自己的三个任务。

samples/my-jobs.tsv：

```text
read	120	阅读自己的代码
write	20	写一条预测
review	60	解释运行结果
```

从 **平台根目录进入学员目录**执行：

```powershell
cd my-asyncboard
cargo run --offline -- preview samples/my-jobs.tsv
cargo run --offline -- run samples/my-jobs.tsv my-results.tsv
cargo run --offline -- run samples/my-jobs.tsv my-results.tsv
```

预览应列出三项；报告应有 `id<TAB>wait_ms<TAB>note` 表头和三条完成记录，通常 write 最先。第二次写同名报告应拒绝覆盖。把一项等待改为 0，再写入另一份新报告比较；不要把固定次序当成调度保证。

## 4. 交付与解释

在 **学员目录**运行 `cargo test --offline` 回归全部测试。保存个人 jobs.tsv、两份报告和调度观察。把预测、错误和解释写入 `LEARNING_LOG.md`，再补一个自己的边界测试。

用自己的话回答：为什么不能用 `thread::sleep` 替代这里的 `.await`？为什么报告保留完成顺序而不再按 ID 排序？`wait_ms` 为什么不是实测耗时？

测试通过只说明已检查的行为符合合同；个人输入、真实产物和解释能帮助你判断学到了什么。

最多 128 项，每项最多 30000 ms；ID 1–24 字节，仅 ASCII 字母数字、`-`、`_`；说明 1–160 个字符且不能有控制字符。报告不覆盖已有文件。

这里使用真正的 Future/Poll/Waker、单线程任务调度和独立定时器驱动线程，没有 Tokio、网络 I/O 驱动、多线程任务池或 TLS。报告 `wait_ms` 来自配置；执行器超时是协作式的，不能中断一次 poll 内的阻塞，也不能撤销外部副作用。进一步观察见 [可靠性路线](../../course/RELIABILITY_PATH.md)。

## 下一款：TaskPocket / PageMint

接下来按合同独立组织自己的程序，从提供外壳的三处填空过渡到自主设计。在 **RustAppLab 根目录**运行（若仍在学员目录，先 `cd ..`）：

```powershell
cargo run --offline -p labquest -- app init taskpocket my-taskpocket
cargo run --offline -p labquest -- learn lesson my-taskpocket 1
cargo run --offline -p labquest -- learn check my-taskpocket 1
```

先读 `my-taskpocket/CONTRACT.md` 和 [独立作品路线](../../INDEPENDENT_PATH.html)。TaskPocket 完成后，用 `cargo run --offline -p labquest -- app init pagemint my-pagemint` 创建离线笔记网页项目；两者都要保留自己的测试、真实产物与发布说明。
