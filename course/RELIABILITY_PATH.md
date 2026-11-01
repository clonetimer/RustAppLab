# 第四段：一批操作要么成功，要么不留下半份结果

你已经用小软件保存文件、处理工单。本段做两件不同的事：用 **StoreDesk** 管理自己的物品，让失败批次能够回滚；用 **AsyncBoard** 安排本地复习提醒，让等待不阻塞其它任务。它们没有自动互相调用，不共享事务，也不执行远程提醒。

最终交付是你自己写的软件、一个可重新打开的 SQLite 数据库、两份提醒报告和一页实验记录。不用把测试数量、截图数量当作理解程度。

## 先选自己的入口

| 当前情况 | 从哪里开始 |
|---|---|
| 没做完 LedgerLite，尚不理解 `Result` | 先回基础第二段，暂不写 SQL 或 Future |
| 想理解数据库为何不能只写一半 | StoreDesk 第一步，先只写校验函数 |
| 理解 BatchMeter 的线程和 MiniKV 的借用 | 再做 AsyncBoard，先解析清单，不先手写执行器 |
| 这两款以前已建工程 | 保留旧目录，用新版导航输入旧目录完整路径；不要覆盖源码 |

平台根目录有 `apps` 和 Workspace 的 `Cargo.toml`；自己的学员工程内有 `src/lib.rs`、`START.html` 和独立的 `Cargo.toml`。下文每组命令都说明在哪个目录运行。

## 1. StoreDesk：先预测，再写一行数据

在**平台根目录**检查原生 SQLite 环境并创建练习：

```powershell
cargo run --locked --offline -p storedesk -- doctor
cargo run --locked --offline -p labquest -- studio start storedesk my-storedesk
cargo run --locked --offline -p labquest -- learn lesson my-storedesk 1
cargo run --locked --offline -p labquest -- learn check my-storedesk 1
```

第一次不通过是因为函数还没有实现。若出现 `winsqlite3.lib` 或 `-lsqlite3` 链接错误，先读 `course/SQLITE_SETUP.md`；这是环境问题，不能用乱改 `TODO` 解决。项目只声明 Windows x64 MSVC、Linux x86_64，不声明 macOS。

只修改自己的 `src/lib.rs`。打开 `TASK_CARDS.html`，一层层展开提示。第一步 `validate_item` 不需要数据库；第二步 `save_item` 才处理 SQL；第三步 `apply_changes` 才处理事务。第一步通过，不代表整个 CLI 已经可用。

用自己的话先回答三个问题：库存为 0 合法吗？中文名称为什么不用字节数限制到 80？名称 `Alice's 钢笔` 为什么不应该因为单引号就被禁止？参数绑定把用户数据与 SQL 结构分开，并非通过删除所有特殊字符达到目的。[1]

第二步导入 `Param` 时放在 `pub mod sqlite;` 下面，不要把 `use` 插到文件开头的 `//!` 模块文档注释之前。

## 2. 准备一套只属于自己的实验文件

接下来这组是**平台参考软件演练**，用于理解目标，不能算作独立完成。先在**平台根目录**复制样例到新目录：

```powershell
if (Test-Path -LiteralPath "my-reliability") { throw "目录已存在，请换一个新名字，保留旧作品。" }
Copy-Item -Recurse -LiteralPath "examples/reliability" -Destination "my-reliability"
cargo run --locked --offline -p storedesk -- init my-reliability/inventory.db
cargo run --locked --offline -p storedesk -- add my-reliability/inventory.db PEN "Alice's 钢笔" 10
cargo run --locked --offline -p storedesk -- add my-reliability/inventory.db BOOK "我的 Rust 练习本" 3
cargo run --locked --offline -p storedesk -- list my-reliability/inventory.db
cargo run --locked --offline -p storedesk -- history my-reliability/inventory.db
```

应有 PEN=10、BOOK=3，流水为空。`.db` 是 SQLite 二进制文件，不用记事本修改。关闭终端后重新执行 list，确认不是仅存在本次进程内。

样例里的列间是**实际 Tab**。修改文字时保留 Tab，不要把它换成空格。Unix shell 可用 `cp -R examples/reliability my-reliability` 代替 `Copy-Item`，同样先确认目标目录不存在。

## 3. 故意让第二项失败：观察的是数据，不只是退出码

`rollback.tsv` 内容为 PEN -2、BOOK -99。在运行之前，先在 `LEARNING_LOG_TEMPLATE.md` 写下两项库存和新增流水的预测。

```powershell
cargo run --locked --offline -p storedesk -- apply my-reliability/inventory.db my-reliability/rollback.tsv
cargo run --locked --offline -p storedesk -- list my-reliability/inventory.db
cargo run --locked --offline -p storedesk -- history my-reliability/inventory.db
```

第一条命令**应当失败**；不要因此停止实验。结果应该仍然是 PEN=10、BOOK=3，流水为空。错误实现可能返回失败却留下 PEN=8，这正是我们要发现的问题。

SQLite 事务通过明确的 COMMIT/ROLLBACK 确定提交或撤销；本项目包装器在业务闭包返回错误时回滚。[2] 关键是**整批操作和流水都在同一个事务内**。每条操作各开一个事务，或者捕获第二项错误后返回 `Ok(())`，都不符合本课合同。

不要拿数据库文件的 SHA-256 不变作为回滚标准。数据库内部页结构、日志处理与逻辑内容不是同一层；这里比较库存、名称、流水，而不是数据库二进制布局。本实验也不是断电持久性验证。

## 4. 成功批次与重复 SKU

```powershell
cargo run --locked --offline -p storedesk -- apply my-reliability/inventory.db my-reliability/success.tsv
cargo run --locked --offline -p storedesk -- list my-reliability/inventory.db
cargo run --locked --offline -p storedesk -- history my-reliability/inventory.db
cargo run --locked --offline -p storedesk -- apply my-reliability/inventory.db my-reliability/repeated-sku.tsv
cargo run --locked --offline -p storedesk -- list my-reliability/inventory.db
cargo run --locked --offline -p storedesk -- history my-reliability/inventory.db
```

| 时点 | PEN | BOOK | 流水总数 |
|---|---:|---:|---:|
| 初始登记后 | 10 | 3 | 0 |
| 失败批次回滚后 | 10 | 3 | 0 |
| success：PEN -2、BOOK +1 | 8 | 4 | 2 |
| repeated-sku：PEN -1、PEN -2 | 5 | 4 | 4 |

同一个 SKU 在一批出现两次，应先后作用于当前事务内的最新值，不是都用最初的库存。再尝试以同一 PEN 编号添加 99，应该拒绝，不能偷偷 REPLACE 成新库存。

三步完成后，将这些命令换成自己的 StoreDesk 运行：进入 `my-storedesk`，把 `cargo run --locked --offline -p storedesk --` 换成 `cargo run --offline --`，并使用数据库和 TSV 的**完整路径**。学员独立工程可能尚未生成 Cargo.lock，首次不要求 `--locked`。

## 5. AsyncBoard：把等待写进函数，而不是卡住线程

回到**平台根目录**：

```powershell
cargo run --locked --offline -p labquest -- studio start asyncboard my-asyncboard
cargo run --locked --offline -p labquest -- learn lesson my-asyncboard 1
cargo run --locked --offline -p labquest -- learn check my-asyncboard 1
```

按 `parse_jobs → render_report → complete` 完成三步。只通过前两步时，程序可能能生成一份格式正确、却根本没有等待的报告，所以必须检查第三步。

参考软件体验：

```powershell
cargo run --locked --offline -p asyncboard -- preview my-reliability/jobs.tsv
cargo run --locked --offline -p asyncboard -- run my-reliability/jobs.tsv my-reliability/RESULTS.tsv
cargo run --locked --offline -p asyncboard -- run my-reliability/equal-delay.tsv my-reliability/EQUAL.tsv
cargo run --locked --offline -p asyncboard -- run my-reliability/duplicate-jobs.tsv my-reliability/SHOULD_NOT_EXIST.tsv
```

最后一条应失败，且不能出现 `SHOULD_NOT_EXIST.tsv`。报告已存在时会拒绝覆盖，重试请换名字，不要删掉旧结果来掩盖行为。

`jobs.tsv` 三项提醒的等待配置不同，通常短等待任务先被观察完成；报告保留执行器实际观察顺序。相同延时不要要求固定顺序，系统调度也可能影响不同延时的观察次序。必须核对每个 ID 出现一次、说明和配置保留，不能只核对某一条演示顺序。

**`wait_ms` 是输入配置，不是实际测量耗时。** 不能将这列相加当总耗时，也不能仅凭报告次序就证明性能或并发正确。整体耗时需要另测；本课不给硬件性能承诺。

## 6. 四个小问题，把 .await 讲明白

Rust 的 Future 需要被 poll 才能推进；返回 Pending 后应通过 Waker 通知再次调度，不是在忙循环中反复 poll。[3] 运行提供的最小实验，观察它们之间的区别：

```powershell
# 在完整平台根目录运行，不在自己的学员工程中
cargo run --locked --offline -p asyncboard --example wake_lab
```

| 实验输出 | 要解释的含义 |
|---|---|
| `created_future body_runs=0` | 仅构造 async Future，函数体尚未执行 |
| `driven_future body_runs=1` | 执行器驱动后才产生函数体内的效果 |
| `missing_wake polls=1 result=timeout (expected)` | 只返回 Pending 却不登记唤醒，不能靠忙轮询“碰运气” |
| `timer_wake result=awake` | 提供的定时器到期唤醒后，等待才能完成 |

有一个容易混淆的细节：这个项目的 `TimerDriver::delay` 在**调用时已经登记计时**，它不是 async 函数；`complete(job, delay)` 的 async 函数体则是惰性的。不能因为 Future 惰性，就推断它创建前的所有代码都不会有副作用。

`drive` 的超时是**协作式超时**，只在轮询间和等待通知时检查。单次 `poll` 里面的 `thread::sleep`、死循环或阻塞系统调用，它不能强行打断；不能把它当作恶意代码沙箱。不要实际运行无限循环来试这个边界。

等待超时也不等于撤销所有外部副作用，更不等于数据库事务回滚。这个提醒器没有自动恢复、网络通知或 Tokio I/O 驱动。已有的 `asyncboard` Linux 二进制包不包含 `wake_lab` 实验；实验通过完整源码的 Cargo 命令运行。

## 7. 换成自己的数据，最后走向独立开发

用自己的两个物品、两条成功变化、一条必失败变化替换示例；提醒清单改成自己的三项复习任务。在学习记录中分别写下预测、实测、用过的提示和原因。不要直接把本讲义的结果表作为自己的观察记录。

最小作品目录应包含：`inventory.db`、三份变化 TSV、正常提醒报告、同延时报告、失败终端输出或文字记录、你的源码与 `LEARNING_LOG_TEMPLATE.md`。不需要把整个 `target` 打包。

最后写两条自己的测试：第一条验证“第二条失败也回滚第一条与流水”；第二条验证“报告保留传入顺序且调用前后 Completion 切片不变”。允许先用提示，但应记录，不把参考实现的运行当作独立开发能力证明。

这十二款 Studio 全部仍提供外壳。接下来回平台运行 `cargo run --offline -p labquest -- app init taskpocket my-independent-taskpocket`，阅读生成的 `CONTRACT.md`，按需求独立实现输入、业务与持久化；此时不再用三个 TODO 代替整体软件设计。

## 参考与验证边界

[1] SQLite 参数绑定接口：<https://www.sqlite.org/c3ref/bind_blob.html>。  
[2] SQLite Transaction：<https://www.sqlite.org/lang_transaction.html>。  
[3] Rust Future 文档：<https://doc.rust-lang.org/std/future/trait.Future.html>。

本段的运行时细节以随包源码为准。Linux 自动演练不等于 Windows 本轮原生验证；自动测试不等于真人教学有效性认证。正式支持范围仍为 Windows x64 MSVC 与 Linux x86_64。
