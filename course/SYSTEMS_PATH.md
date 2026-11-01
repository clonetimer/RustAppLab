# 第三段 · 从单个文件到个人工单档案

本段把 BatchMeter、MiniKV、NetFetch、ServiceDesk 连成一条路线。你最后交付的不是四张“测试通过”截图，而是自己提出的工单、关闭前后的 JSON 快照、文件统计结果和可重新打开的索引数据库。

**开始条件**：能用 `Result` 区分成功和失败，能解释 `String` 与 `&str` 的区别，完成 TextScope、LedgerLite 和 TermGarden 的基本操作。仍不清楚借用或函数返回值时，先回到原作品，不要靠背网络命令跳过难点。这里不需要 Python、Playwright、云服务器或联网下载 crate。

## 1. 一眼看懂任务与交付

| 顺序 | 修改的函数 | 当次问题 | 自己留下什么 |
|---|---|---|---|
| BatchMeter | `analyze` → `aggregate` → `analyze_many` | 如何把逐文件失败、线程结果与稳定顺序分开？ | 同一组文件的单线程和多线程统计 |
| MiniKV | `validate_key` → `execute` → `next` | 文件写入失败时内存怎么办？为什么迭代器不能借用临时数据？ | 记录快照路径的 `.kv` 和 TSV 索引 |
| NetFetch | `parse_url` → `parse_response` → `download` | 连接成功为什么不等于拿到了完整文件？ | 从本机 HTTP 保存的两个 JSON 文件 |
| ServiceDesk | `validate_title` → `apply` → `render_json` | 输入怎样变成状态，再持久化并返回给网页？ | 自己的两条工单、状态转换与重启观察 |

顺序是学习依赖，不是强制天数。每一款都先预测、再修改一个函数、检查当步，然后尝试自己的输入。每款三步全部完成，才进入完整 CLI 与跨软件组合。

## 2. 先看作品，之后编辑自己的工程

平台根目录含 `apps`、`course`、`LEARN_NEXT_WINDOWS.cmd`。平台中的应用是已实现的参考软件。学员目录有自己的 `src/lib.rs` 和 `Cargo.toml`，这才是要编辑的代码。

在**平台根目录**依次创建（没有完成当前款时，不必一次创建全部）：

```powershell
cargo run --locked --offline -p labquest -- studio start batchmeter my-batchmeter
cargo run --locked --offline -p labquest -- studio start minikv my-minikv
cargo run --locked --offline -p labquest -- studio start netfetch my-netfetch
cargo run --locked --offline -p labquest -- studio start servicedesk my-servicedesk
```

已有工程不要重建，直接双击新版 `LEARN_NEXT_WINDOWS.cmd` 并输入原目录。L 查看任务，E 打开源码，1/2/3 依次查看提示，C 检查当前步；查看提示不运行代码。

例如检查 BatchMeter 的第一步（**平台根目录**）：

```powershell
cargo run --locked --offline -p labquest -- learn lesson my-batchmeter 1
cargo run --locked --offline -p labquest -- learn check my-batchmeter 1
```

也可进入**学员工程目录**执行 `cargo test --offline --test milestones step1_`。这条本地命令只测第一步；第二步改动后，平台检查会累计回归前面步骤。生成的新工程都带 `TASK_CARDS.md`，旧工程仍可用新平台读取任务，无需覆盖源码。

## 3. BatchMeter：先串行理解，再实现并发

第一步先拿一个文本解释“打开文件失败”和“空文件读成功”。正文为 `Rust 学习` 加一个 LF 换行时是 12 字节、1 行，不是 8 字节。CRLF 的字节数与 LF 不同，不要把不同换行的文件当成同一输入。

第二步只处理 `FileStats` 切片。先手算两个记录的字节和行数，再通过 `checked_add` 检查溢出。没有必要创建一个巨大文件才能测试累计溢出。

第三步先画分组：假设四个输入、两个工作线程，每个结果附带原输入索引。通道到达顺序不是最终显示顺序。发送端的副本、主发送端和接收端分别何时释放，要写清楚。

三步完成后，在**my-batchmeter**目录运行：

```powershell
cargo run --offline -- scan 1 samples/one.txt samples/two.txt
cargo run --offline -- scan 4 samples/one.txt samples/two.txt
```

默认样例应合计 2 文件、36 字节、5 行；两次逐文件结果和汇总一致。再把一个路径改成不存在的文件，预期为明确错误，不得悄悄少算一份。

**自查**：行为测试只验证输出、错误和顺序，串行实现也可能通过。要完成并发学习，还必须检查自己的代码确实创建有界工作线程、等待它们结束，并解释为何不会一直等待通道。速度不是通关指标；四个线程不保证比一个线程快。

## 4. MiniKV：把“程序里的值”变成“退出后还在的值”

第一步不做自动清洗：`snapshot/before` 是合法键，首尾有空格的键直接拒绝；限制是 128 个 UTF-8 字节，不是 128 个字符。第二步把操作分为只读与写入：读取不追加日志；变更先验证，再追加日志，成功后再改内存。第三步实现借用型前缀扫描，不返回对临时 `String` 的引用。

在**my-minikv**目录完成三步后：

```powershell
cargo run --offline -- put my-index.kv snapshot/before before.json
cargo run --offline -- put my-index.kv snapshot/after after.json
cargo run --offline -- put my-index.kv course/next NetFetch
cargo run --offline -- scan my-index.kv snapshot/
cargo run --offline -- get my-index.kv course/next
```

前缀扫描只返回两条快照记录，按键顺序排列。每次 `cargo run` 都启动新进程；能重读自己的值比仅在一个 `HashMap` 中查询更有意义。

**错误实验**：在测试里令 `Journal::append` 返回错误，再检查旧值没有变化。这个实验不代表已保证断电一致性。已有讲义和文档测试还演示了“借用迭代期间修改数据库”的编译错误；修复方向是先收集拥有型键，结束借用后再修改，而不是泄漏内存。

## 5. NetFetch：分层排错，不把所有失败都叫网络问题

第一步只解析 URL，不要求有服务器。第二步只给函数一段 HTTP 字节，不要求网络。第三步才连接真实服务并写文件。这是把容易混淆的三个问题分开。

| 层次 | 可以故意制造的输入 | 应观察到什么 |
|---|---|---|
| URL | `https://localhost/`、端口 0、路径含空格 | 解析拒绝；不是让你安装 HTTPS 库 |
| HTTP | 404、重复 Content-Length、长度不足 | 响应拒绝；不保存错误页或半截正文 |
| 文件 | 输出已经存在、父目录不存在 | 明确失败；不能截断已有产物 |

`Content-Length` 是正文的字节数。正文“中”需要 3 个 UTF-8 字节。解析头时可以需要文本，但正文要保留为字节，不能假定所有文件都是 UTF-8。

当前软件只支持回环地址的同步 HTTP 子集，不支持公网下载、HTTPS、重定向与 chunked。不要把拒绝这些输入当成需要学员绕过的缺陷。实际下载留到下节联合实验。

## 6. ServiceDesk：让三个层次各司其职

`validate_title` 负责输入合同；`apply` 负责状态转换；`render_json` 负责表示结果。网络接入和日志基础设施已提供，不要求先重写 HTTP 服务器。

标题先拒绝控制字符，再修剪普通首尾空白，按 Unicode scalar 限制长度。记录由提供的操作与日志重放保持 ID 递增，创建时延续最后一条 ID 并检查溢出，不把数组下标当 ID。关闭不存在或已关闭的工单应失败，不应修改其它工单。

JSON 必须用提供的 `json_string` 做正确转义，布尔值不能写成字符串。`<>&` 的转义遵循本课程输出合同，不能拿 HTML 实体转义替代 JSON 编码。

**自己解释**：同一个 `apply` 为什么既能在测试中被调用，也能在网页点击后被调用？浏览器显示了“完成”，为什么还要停服再启动验证日志？

## 7. 联合交付：个人工单档案（先体验参考程序）

以下操作在 **Windows PowerShell、平台根目录**进行。你会开两个窗口：A 保持服务器运行；B 执行下载和整理。不要在 A 正在服务时继续输入 Cargo 命令，也不要把 A、B 当作共享变量的同一窗口。

### 窗口 A：创建一个新的档案目录并启动服务

```powershell
cargo build --locked --offline --release -p servicedesk -p netfetch -p batchmeter -p minikv
if ($LASTEXITCODE -ne 0) { throw "构建失败，先看上面的第一条错误" }
$Out = Join-Path (Get-Location) "my-ticket-archive"
New-Item -ItemType Directory -Path $Out -ErrorAction Stop
.\target\release\servicedesk.exe init "$Out\tickets.log"
.\target\release\servicedesk.exe serve "$Out\tickets.log" 8766
```

目录必须新建；已经存在时改用另一个名字，**不删除旧作、不加 Force 覆盖**。若端口占用，选一个未占用端口，例如 8767，并在窗口 B 和浏览器一并更改。不要关闭其它不认识的服务进程。

用浏览器打开 `http://127.0.0.1:8766/`，创建两条自己的学习工单，暂时都不关闭。窗口 A 此时保持运行是正常状态。

### 窗口 B：保存第一次快照

在**同一个平台根目录**另开 PowerShell：

```powershell
$Out = Join-Path (Get-Location) "my-ticket-archive"
.\target\release\netfetch.exe fetch http://127.0.0.1:8766/api/tickets "$Out\before.json"
Get-Content -LiteralPath "$Out\before.json" -Raw -Encoding UTF8
```

预期 JSON 有两条 `done:false` 记录。回浏览器关闭其中一条，然后在窗口 B 保存第二份：

```powershell
.\target\release\netfetch.exe fetch http://127.0.0.1:8766/api/tickets "$Out\after.json"
Get-Content -LiteralPath "$Out\after.json" -Raw -Encoding UTF8
```

现在应当一条完成、一条未完成。不要重复用 `before.json` 做输出，已有文件的拒绝是保护行为；实现时使用 create_new，不能先检查存在再用 create 截断。初学者使用 PowerShell 5.1 时不要用 `>` 创建要交给 BatchMeter 的 UTF-8 数据；这里的 JSON 是 Rust 下载器直接按字节写入的。

### 比较并发统计，建立可重开的索引

在窗口 B：

```powershell
.\target\release\batchmeter.exe scan 1 "$Out\before.json" "$Out\after.json"
.\target\release\batchmeter.exe scan 4 "$Out\before.json" "$Out\after.json"
.\target\release\minikv.exe put "$Out\index.kv" snapshot/before before.json
.\target\release\minikv.exe put "$Out\index.kv" snapshot/after after.json
.\target\release\minikv.exe scan "$Out\index.kv" snapshot/
.\target\release\minikv.exe export "$Out\index.kv" "$Out\index.tsv"
```

两种线程数对同一输入的结果应一致；关闭前后 JSON 的字节数不必相同，布尔值 `false` 改为 `true` 就会少一个 ASCII 字节。MiniKV 保存的是索引字符串，不会自动跟踪或验证文件存在；移动档案目录后，文件名仍相对于档案目录理解。

在 A 按 Ctrl+C，再用相同 `tickets.log` 重新执行 `serve`。刷新网页，确认记录和完成状态还在。最后再次停止服务。`my-ticket-archive` 就是可保存的作品，不要只保留屏幕截图。

### 把参考体验换成自己的实现

上面运行的是平台参考应用，**不能算作独立完成**。自己四款工程通过三个里程碑后，分别进入工程执行 `cargo build --offline --release`，再把上述命令的程序路径替换成对应学员工程的 `target\release\xxx.exe`。数据路径仍用同一个 `$Out` 的新目录，操作顺序不变。

Linux 中路径分隔符使用 `/`，程序无 `.exe`，同样从平台根目录构建，使用 `mkdir` 创建新目录；用 `./target/release/servicedesk serve my-ticket-archive/tickets.log 8766` 启动。其它参数一致。普通学员不需要运行维护者的 Python 检查。

## 8. 出错后从哪一层回退

| 现象 | 先检查 | 不应该做什么 |
|---|---|---|
| 第一步测试成功但完整 CLI 失败 | 第二、三步是否真的实现 | 把第一步成绩当成整款软件已完成 |
| 接收通道一直等 | 所有发送端副本是否释放，工作线程是否退出 | 无限增加线程或把错误吞掉 |
| `address already in use` | 8766 是否被自己的上一次服务占用 | 随意终止系统进程 |
| 下载连接失败 | A 是否运行、端口是否一致 | 放宽回环限制或重装 Rust |
| 已有目标被拒绝 | 是否应使用新的输出名 | 删除自己要保留的文件来凑通过 |
| MiniKV 提示占用 | 是否仍有使用该库的进程 | 程序还活着就删除锁文件 |
| 源码变更、报告过期 | 保存后重新运行累计检查 | 直接编辑历史报告为 PASS |

## 9. 最终复盘与边界

在档案目录新建 `LEARNING_LOG.md`，按顺序写出：自己的标题、预测的状态变化、两个快照差异、单/多线程结果、重启结果、使用过的提示、仍不懂的概念，以及一条自己新增的边界测试。`examples/systems/LEARNING_LOG_TEMPLATE.md` 可作起点；没有实际做的项写“未做”。

自动检测通过不证明真正并行实现，也不证明独立掌握生命周期。教师应让学员解释一处发送端释放、一处日志失败顺序和一处借用返回，再使用不同输入重复运行。

本段不宣称完成成熟异步框架、生产级数据库、桌面 GUI 或 WebAssembly。下一步可以进入 StoreDesk 的事务专题、AsyncBoard 的 Future/await 专题，之后再做 TaskPocket/PageMint 独立需求。原 0.17 RC 冻结版本和本教学开发分支分别管理；不声明 macOS 支持。

## 概念来源

- Rust 官方 `thread::scope` 与通道文档：https://doc.rust-lang.org/std/thread/fn.scope.html 、https://doc.rust-lang.org/std/sync/mpsc/index.html 。用于核对作用域线程借用与通道关闭语义。
- Rust Book 迭代器章节与标准库：https://doc.rust-lang.org/book/ch13-02-iterators.html 、https://doc.rust-lang.org/std/iter/trait.Iterator.html 。用于核对 next、惰性求值与组合。
- Rust 标准库 `TcpStream`：https://doc.rust-lang.org/std/net/struct.TcpStream.html 。用于核对连接和读写超时的 API 边界。
- 本项目各 `apps/*/tests/milestones.rs` 和 `course/studio/*.md` 是作品行为的公开教学合同；协议限制是本课程选择，不应扩写成通用 HTTP 实现承诺。

完成后打开 [第四段：事务与等待](../RELIABILITY_PATH.html)，依次学习 StoreDesk 与 AsyncBoard；保留本段工单档案，不需重建。
