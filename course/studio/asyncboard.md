# AsyncBoard · 离线异步提醒工作台（第 12 款 Studio 作品）

你要交付一款自己的任务提醒程序：给出三项任务的等待时间，**同时开始等待**，最终按完成顺序生成 TSV 文件。不是观看参考软件，而是亲手完成三处 Rust 函数。

## 先看看要做成什么

在完整 Rust AppLab 根目录运行参考程序：

```powershell
cargo run --offline -p asyncboard -- demo my-async-demo
```

打开 `my-async-demo/jobs.tsv` 和 `my-async-demo/RESULTS.tsv`。三条任务依次等待 110、40、75 毫秒，通常会先看到 `write`、再 `review`、再 `read`；强烈的系统调度延迟可能改变观察次序。任务同时计时，先完成的排前面。这不是网络下载，也不使用 Tokio。

## 创建你自己的学员工程

```powershell
cargo run --offline -p labquest -- studio start asyncboard my-asyncboard
cargo run --offline -p labquest -- studio check my-asyncboard 1
```

第一次得到 FAIL 是正常的：任务还没有实现。现在只打开 `my-asyncboard/src/lib.rs`，不修改参考软件和 `tests/`。每做完一步按 Ctrl+S，重新运行 check。Windows 可用 `CONTINUE_WINDOWS.cmd` 选菜单 **12**。

## 第一步：解析任务清单 `parse_jobs`（输入与所有权）

**你要改的地方**：`src/lib.rs` 中 `// LESSON 1 BEGIN` 到 `// LESSON 1 END`。

输入每行包含三段，用 TAB 分隔：`ID`、非负整数毫秒、任务说明。需要跳过空行和 `#` 开头的注释；错误必须返回 `Err(...)`，不能静默忽略；最后返回拥有字符串的 `Vec<Job>`。你已经在 FileQuest、TextScope 学过 `String`、集合和 `Result`，现在用在真实清单校验。

边界合同：最多 128 项，毫秒不超过 30000；ID 不超过 24 字节，只能是 ASCII 字母数字、`-`、`_`，不能重复；说明不能为空、不超过 160 个字符，不能包含控制字符。

**先预测**：`todo\t0\t写代码` 合法；`todo\t-1\t写代码` 不合法；两个 `todo` 不能同时出现。

完成后：

```powershell
cargo run --offline -p labquest -- studio check my-asyncboard 1
```

通过再做第二步。若报 `expected u64`，检查 `delay.parse::<u64>()` 和 `?`；不要用 `unwrap()` 掩盖输入错误。

## 第二步：生成结果 `render_report`（借用与格式化）

**你要改的地方**：`// LESSON 2 BEGIN` 到 `// LESSON 2 END`。

按照 `completions` 的原有顺序生成 TSV，第一行固定是 `id<TAB>wait_ms<TAB>note`，每条结果占一行，结尾有换行。你无需修改 `completions`，函数参数是借用的切片 `&[Completion]`。

思考：为什么不先排序？因为执行器已经按实际完成顺序交回结果，擅自排序反而可能破坏语义。中文文本不应被替换成问号。

```powershell
cargo run --offline -p labquest -- studio check my-asyncboard 2
```

这会重新检查前两步。通过后比较 `&str`、`String` 与借用：你不需要为了写一行 TSV 克隆整个任务集合。

## 第三步：在真正的 `async fn` 中等待 `.await`

**你要改的地方**：`async fn complete` 的 `// LESSON 3 BEGIN` 到 `// LESSON 3 END`。

`Delay` 已由 `src/runtime.rs` 提供。你的任务只有两件事：**先 `delay.await`，再用输入的 Job 生成 Completion**。不要使用 `std::thread::sleep` 假装 `.await`，那会把执行线程阻塞住；也不要一收到 `Job` 就立刻返回结果。

```powershell
cargo run --offline -p labquest -- studio check my-asyncboard 3
```

检查器会使用多个真实计时任务并检查完成次序与等待时间，还会运行 CLI 演示。若写死输出或者略过 `.await`，测试应失败。由于系统调度存在误差，不要用测得的毫秒数推断精确性能。

## 为什么 Future 需要运行时？

`async fn` 返回惰性的 `Future`，它不会自动执行；执行器要不断在可继续的时候调用 `poll`。返回 `Poll::Pending` 时要保存 `Waker`，等事件发生后再唤醒。项目提供的 `src/runtime.rs` 是**教学型执行器**：单线程调度 Future，一条独立定时器线程负责到期唤醒，并不包含网络 I/O 驱动。你可以通过测试 `pending_is_not_busy_polled` 查看轮询次数，没有把忙轮询当成异步。

后续使用 Tokio 时，保持 `Future` 和 `.await` 这些概念，但要换成成熟的异步网络运行时，而不是复制这份教学执行器去做生产服务器。

## 交付作品

```powershell
cd my-asyncboard
cargo run --offline -- demo my-first-async-output
cargo run --offline -- preview my-first-async-output/jobs.tsv
cargo run --offline -- run my-first-async-output/jobs.tsv my-first-async-output/SECOND.tsv
```

交付 `src/`、`jobs.tsv`、`RESULTS.tsv`、`LEARNING_LOG.md`，再使用自己的新任务完成一次实验。请写下“为什么三个任务的总时间不等于三个等待时间之和”，以及“这里的异步为何不代表 HTTP 网络已经异步”。

## 分层提示与常见错误

先看 `HINTS.md` 的第一层提示，再看第二层。常见错误：漏掉 TAB、没有对重复 ID 处理、把 `await` 写在普通 `fn`、没有 `use` 需要的模块、第二次运行覆盖旧报告失败。最后一个是数据保护设计，不是 Rust 编译错误。

## 复盘：不要把配置时间当成计时结果

报告的 `wait_ms` 来自输入配置，不是实测耗时；相同延时不保证固定次序。Future 函数体是惰性的，但提供的 TimerDriver::delay 在调用时已经登记计时。drive 是协作式超时，不能打断单次 poll 内的阻塞或死循环。

完成三步后看平台根目录 RELIABILITY_PATH.html，并在平台运行 `cargo run --offline -p asyncboard --example wake_lab`，观察函数体、轮询与唤醒。这里不提供 Tokio 网络能力，也不把超时当作撤销外部副作用的事务。
