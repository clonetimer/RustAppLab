# MiniKV · 本地配置与书签仓库

## 先判断：现在适合学这款吗？

这不是零基础第一课。你应已完成 TextScope，能区分 `String` 与 `&str`；完成 BadgeForge，知道 trait 是一组约定；完成 BatchMeter，见过线程与 `move`。尚不熟悉这些内容时，先回到对应作品，不要因为本章排在新版菜单里就跳级。

你不需要先懂存储引擎。本章从一个具体需求开始：保存“下一课学什么”“常用网址”“自己的作品路径”，关闭程序后再打开，资料还在；输入 `book/` 时只看书签。这比单纯运行一个 HashMap 示例多了真实用途，但不会要求你先实现底层文件协议。

本章交付：一个独立 Rust 程序、一份真实 `.kv` 数据文件、一份 TSV 导出，以及你对一个借用错误的解释。文件不是加密的，请不要保存密码和重要唯一副本。

## 先体验参考软件，看清自己的目标

在完整 RustAppLab 根目录（能看到 Cargo.toml）打开 PowerShell：

```powershell
cargo run --offline -p minikv -- demo my-kv-demo
cargo run --offline -p minikv -- put my-kv-demo/my-bookmarks.kv "course/next" "完成自己的迭代器"
cargo run --offline -p minikv -- get my-kv-demo/my-bookmarks.kv "course/next"
cargo run --offline -p minikv -- scan my-kv-demo/my-bookmarks.kv "bookmark/"
```

第二次运行 `demo` 的目录必须换一个名字，它不会覆盖原有文件。`get` 应输出你刚写的文字；`scan` 只列出 `bookmark/` 开头的键，按键的字节序排列，不是按你添加的顺序，也不是中文拼音排序。

打开 `my-kv-demo`：`my-bookmarks.kv` 是日志文件，`bookmarks.tsv` 是首次演示时的可读导出，`MY_PRODUCT.md` 是作品说明。**后来更新数据库不会自动更新此前的导出**，需要重新导出到另一个新文件。十六进制是编码，不是加密。

做一个预测：连续两次保存同一个键，`scan` 最终显示一条还是两条？为什么日志文件可能有多条记录？记录答案后用自己的键验证。

## 创建自己的工程：只改你的 lib.rs

Windows 学员可双击 `CONTINUE_WINDOWS.cmd`，选 9。也可在 RustAppLab 根目录执行：

```powershell
cargo run --offline -p labquest -- studio start minikv my-minikv
cargo run --offline -p labquest -- studio check my-minikv 1
```

首次检查会失败，因为练习还没写。浏览器打开 `my-minikv/START.html`，编辑 `my-minikv/src/lib.rs`；不要编辑平台的 `apps/minikv/src/lib.rs`，那是参考实现。

文件关系：

| 文件 | 作用 | 这次是否需要修改 |
|---|---|---|
| Cargo.toml | 独立包名、版本和 Rust 版本声明 | 不需要；不引用平台目录，也没有第三方 crate |
| src/main.rs | 读取命令行、选择文件、显示结果、演示线程共享 | 先不改；三步完成后阅读 |
| src/lib.rs | 业务类型、Database 和迭代器 | 依次完成 TODO(1)、TODO(2)、TODO(3) |
| src/journal.rs | 已提供的内存/文件日志、编码与锁释放 | 先当作工具；进阶阅读，不要求从零实现 |
| tests/milestones.rs | 三步的公开行为合同 | 可以读预期，不要删除来制造通过 |
| LEARNING_LOG.md | 预测、错误、解释与自己的扩展 | 每步记录一项实测结果 |

主线数据流是 `命令行 → Operation → Database::execute → Journal → 文件`。扫描不修改文件，它从 Database 中借用当前的数据。CLI 的职责不等于数据库逻辑，底层文件编码也不等于业务规则。

## 第 1 步：让错误不再只是一句字符串

**修改位置：`validate_key` 函数的 TODO(1)。** 其它 TODO 暂时不动。

键必须同时满足：UTF-8 字节长度 1–128；不包含控制字符；没有首尾空白。中间的普通空格、斜杠和中文允许使用。不要静默去掉空白再存储，否则 `" book"` 和 `"book"` 会被错误地当成同一个输入。

你已经学过 `Result<T, E>`。本章成功时不需要返回一个新值，因此成功类型是 `()`；错误类型不再是随意的 `String`，而是 `KvError`。它把键非法、键不存在、文件错误等分开表示。调用者可以匹配错误类别，而不必猜测中文报错字符串是否改变了。

先预测下表，再写测试：

| 输入 | 应有结果 | 关键原因 |
|---|---|---|
| `book/rust` | 成功 | 普通业务键 |
| `书签/Rust` | 成功 | 中文不等于非法字符 |
| ` book` | 失败 | 首部空白不能静默去掉 |
| `a\tb`（真正的 Tab） | 失败 | 控制字符 |
| 128 个 ASCII x | 成功 | 恰好达到字节上限 |
| 43 个“好” | 失败 | UTF-8 总字节数超过 128，不是按字符个数计算 |

建议先写“空/过长”的分支，再写“空白/控制字符”的分支，最后返回成功。判断合法不需要新建字符串、不需要打开文件。保存后：

```powershell
cargo run --offline -p labquest -- studio check my-minikv 1
```

常见卡点：返回 `true` 不符合 `Result<(), KvError>`；只用 `trim().is_empty()` 会漏掉首尾空白；`chars().count()` 不是本作品约定的字节上限。发生编译错误先看第一条 error，不要一次改三个函数。

解释任务：`KvError::InvalidKey` 与 `KvError::Missing` 分别表示什么？输入非法与输入合法但不存在，为什么不应该合并成一种情况？

## 第 2 步：一次读写，从借用检查走到拥有数据

**修改位置：`Database<J>::execute` 的 TODO(2)。** `open` 中的恢复流程已经提供，先不修改。

软件接收三类操作：Put 保存值；Get 读取值；Delete 删除键。用 `match` 完成，不要靠固定字符串猜测命令。业务合同如下：

| 操作 | 成功结果 | 是否写日志 | 失败时的要求 |
|---|---|---|---|
| Put 新键 | Stored，replaced=false | 是 | 参数非法或日志失败时不改内存 |
| Put 已有键 | Stored，replaced=true | 是 | 不先删旧值 |
| Get 已有键 | Value | 否 | 不存在返回 Missing |
| Delete 已有键 | Removed | 是 | 不存在先报错，不能留下删除记录 |

`Database<J>` 中的 J 是实现了 Journal trait 的某个类型。`MemoryJournal` 不写磁盘，便于实验；`FileJournal` 写真实文件。它们都实现同一组方法，但不需要把“如果是测试就怎样”散布到业务代码中。你已经用过泛型，此处泛型服务于可替换的存储，而不是为了炫技。

最容易写错的顺序是先执行 `data.insert(...)`，然后才调用日志。假如磁盘写失败，内存已经显示新值，重启后却恢复旧值。这里的合同要求：**先验证 → 成功追加日志 → 更新内存**。调用 `append` 后的 `?` 表示失败时立即返回，不继续改 map。提供的日志适配器在部分写入错误后停止后续写入；这不是数据库级多操作事务，也不承诺断电时完整持久化。

所有权提示：先 `match &operation`，此时只借用键和值来验证；随后 `append(&operation)` 也只借用。验证和追加成功后，再 `match operation`，把拥有的 String 放入 map。这样通常无需先克隆整条操作。

Get 可以直接在第一轮匹配中返回。它返回独立拥有的 String，调用者稍后修改数据库不会让这个值失效；第 3 步会对比“批量扫描为什么使用借用”。

```powershell
cargo run --offline -p labquest -- studio check my-minikv 2
```

本阶段会重新运行第 1 步。测试还会使用一个故意返回写入错误的 Journal，验证旧值没被更改；它不是在你的电脑上真的把磁盘写满。

自己的实验：保存 `project/name`，保存两次不同内容，预测两次 replaced 的值；读取它；删除它；再次读取。把每次实际结果写进 LEARNING_LOG.md。此时第 3 步未完成，完整 CLI 的 scan 还不能作为通过依据。

## 第 3 步：你来实现 Iterator，而不只是调用它

**修改位置：`impl Iterator for PrefixScan` 中 `next` 的 TODO(3)。**

调用 `db.scan("book/")` 得到一个迭代器，但还没把全部结果复制出来。`inner` 已经定位到第一个不小于前缀的键，BTreeMap 保证有序。本步每次只需要取下一个条目：若匹配前缀，返回键和值的引用；若不匹配或已到末尾，记录结束并返回 None。结束后再次 next 仍应返回 None。

例如顺序为 `book/a`、`book/b`、`bookmark`、`z` 时，`book/` 只匹配前两项。遇到 `bookmark` 后停止；因为键已经排序，不需要继续遍历整个数据库。空前缀匹配所有键，缺失前缀没有结果。

`type Item = (&'a str, &'a str)` 说的是“每个条目由两个引用组成”。这里的 `'a` 来自数据库的借用，**不是凭空延长字符串寿命**。不要在 next 里创建一个临时 String，再返回它的 `&str`；临时值离开函数就不存在了。直接从底层 map 的键和值借用。

实现 next 后，标准迭代器方法就可以组合使用：

```rust
// 只读前两条匹配数据，不提前复制整份数据库。
let first_two: Vec<_> = db.scan("book/").take(2).collect();
```

```powershell
cargo run --offline -p labquest -- studio check my-minikv 3
```

最后一步也会运行真实 CLI，跨不同进程保存和读取中文数据，并检查导出不能覆盖已有文件。只返回写死的两个样例、忽略前缀、删除本地测试，都不能完成权威检查。

## 实验 A：故意理解一次 E0502

完成三步之后，打开 `src/lib.rs` 中 scan 上方的文档示例，观察下面的冲突：

```rust
let mut rows = db.scan("");
db.execute(Operation::Put { key: "new".into(), value: "value".into() })?;
println!("{:?}", rows.next());
```

rows 仍将在最后一行被使用，它借用数据库。中间的 execute 却需要独占可变借用。编译器拒绝，是为了防止你一边引用结构内部的数据，一边修改结构。不要把每个值都 clone 一遍来碰运气。

要删除某个前缀下的全部键，可以先把**需要跨过借用边界的键**收集成 `Vec<String>`，结束 scan，再逐个执行删除。这个修正示例也在文档中。

在参考应用目录或已完成的学员目录执行：

```powershell
cargo test --offline --doc
```

其中 compile_fail 示例“如预期无法编译”才算测试通过；另一个修正示例必须编译并运行通过。不要把前者的编译失败当作产品构建失败。

## 实验 B：四个线程为什么没有复制四份数据库？

先看 `src/main.rs` 的 `share_demo`，再运行自己的程序（确保三步已完成）：

```powershell
cd my-minikv
cargo run --offline -- share-demo shared-demo.kv
```

预期输出 `workers=4 records=40 expected=40`。每次实验换一个新路径，避免误修改旧数据。

`Arc::clone` 共享拥有同一个对象，不是深拷贝所有记录；Mutex 让某一时刻只有持锁线程进入这段业务操作。锁守卫离开作用域时释放锁，不要求手动调用 unlock。与 BatchMeter 的消息传递不同，这里展示共享状态。四个工作线程并不意味着磁盘写入变成四倍速度，本实现把同一数据库的写操作串行化。

解释任务：如果把锁范围移到整个循环外，结果是否一定错？吞吐和公平性可能怎样变化？不要在没有测量的情况下把线程数当性能提升。

## 最终作品：自己的资料，而不是重复演示

在学员工程目录执行：

```powershell
cargo run --offline -- put my-life.kv "book/rust" "我正在学迭代器"
cargo run --offline -- put my-life.kv "project/next" "为自己的工具加搜索"
cargo run --offline -- scan my-life.kv "project/"
cargo run --offline -- export my-life.kv my-product.tsv
cargo build --offline --release
```

退出终端后重新打开，再运行 get，确认不是仅仅在内存中保留了数据。Windows 的可执行文件在 `target/release/minikv.exe`。

选择一个自主变体并补测试：按值里的关键词搜索；导出只包含某个前缀；增加“键已存在则拒绝”的 insert-only 命令。这里不给完整答案；先写输入、预期输出和失败条件，再实现。它们不是本版已有功能，不能只在说明书中写上就当作完成。

验收交付包括：自己的源码、数据库副本、TSV、运行命令、一个新增测试，以及 LEARNING_LOG.md 中对 E0502 或 Arc/Mutex 的解释。自动通过说明这份源码满足测试，不代表你已经理解全部实现。

## 范围与异常处理

这是有序内存索引加追加日志的教学键值库，不是 SQLite 或 Redis 的替代品；没有多键事务、日志压缩、索引持久化、网络服务、加密和恶意篡改防护。单个日志最多 16 MiB；每次打开完整重放，数据规模大时不适合本设计。

正常退出会删除 `.lock`；异常终止可能留下它。不要在另一个程序仍运行时删除锁。先确认所有 MiniKV 实例已退出，保留数据库副本，再处理锁文件。损坏或截断日志会被拒绝，软件不会擅自修复你的数据。文件权限、断电持久化和网络文件系统不是本章的完整保证。

## 核对用的官方资料

Iterator 及 next：<https://doc.rust-lang.org/book/ch13-02-iterators.html>。
共享状态及 Arc/Mutex：<https://doc.rust-lang.org/book/ch16-03-shared-state.html>。
这些是概念补充；本章的主线是你自己的可运行作品，不要求先读完整本书。
