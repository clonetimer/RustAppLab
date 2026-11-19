# MiniKV · 保存下一课目标和自己的书签

**这次要做成什么：**用键保存文字，关闭程序再读取仍在；按前缀查找并导出 TSV。你将实现结构化错误、业务读写和借用迭代器，底层文件日志已提供。

**开始条件：**完成 [BatchMeter](../batchmeter/README.md)，并能说明 TextScope 的 `String/&str` 和 BadgeForge 的 trait。

本目录是已实现的参考软件；你的练习代码放在下面新建的 `my-minikv/` 中。

## 1. 看目标，创建自己的工程

在 **RustAppLab 根目录**（能看到根 `Cargo.toml`）运行：

```powershell
cargo run --offline -p minikv -- demo reference-kv
cargo run --offline -p minikv -- get reference-kv/my-bookmarks.kv course/next
cargo run --offline -p labquest -- studio start minikv my-minikv
cargo run --offline -p labquest -- learn lesson my-minikv 1
cargo run --offline -p labquest -- learn check my-minikv 1
```

演示会创建数据库、`bookmarks.tsv` 和 `MY_PRODUCT.md`；第二条应显示 `理解借用迭代器`。目录必须是新的，后续改库不会自动更新旧导出。

学员首次检查因 TODO 未实现而 FAIL 是正常的。先看第一条失败；若是编译器或链接器找不到，则应先修好环境。

打开 `my-minikv/START.html` 或 `TASK_CARDS.html`；只编辑 **`my-minikv/src/lib.rs`**，每次一个 TODO。命令行外壳 `src/main.rs` 已提供，先不改。目录已存在就继续使用；新建另一份练习时换名称。

## 2. 三个检查点

| 步骤与函数 | 要完成的行为 | 本次概念 |
|---|---|---|
| TODO(1) · `validate_key` | 检查 1–128 字节、无首尾空白和控制字符，返回 `KvError` | `Result<(), E>`、结构化错误、字节与字符 |
| TODO(2) · `Database<J>::execute` | 处理 Put/Get/Delete；日志追加成功后才更新内存 | 泛型 trait、借用与移动、失败时的一致性 |
| TODO(3) · `PrefixScan::next` | 每次返回一组借用的键值，前缀不匹配后结束 | 实现 `Iterator`、`Option`、生命周期 |

每步先写一组输入和预测，再修改、保存。第一步通过后，在 **平台根目录**继续：

```powershell
cargo run --offline -p labquest -- learn lesson my-minikv 2
cargo run --offline -p labquest -- learn check my-minikv 2
cargo run --offline -p labquest -- learn lesson my-minikv 3
cargo run --offline -p labquest -- learn check my-minikv 3
```

后一步会累计回归前面的步骤。提示用 `cargo run --offline -p labquest -- learn hint my-minikv 1`；进度用 `cargo run --offline -p labquest -- learn next my-minikv`。详细解释见 [minikv 讲义](../../course/studio/minikv.md)。

## 3. 用自己的输入完成一次作品

三步通过后，在自己的数据库中记一条书签和下一课目标，导出文件使用新名称。

从 **平台根目录进入学员目录**执行：

```powershell
cd my-minikv
cargo run --offline -- put my-life.kv "book/rust" "我在学迭代器"
cargo run --offline -- put my-life.kv "course/next" "本机 HTTP"
cargo run --offline -- get my-life.kv "course/next"
cargo run --offline -- scan my-life.kv "book/"
cargo run --offline -- export my-life.kv my-product.tsv
```

`get` 应显示 `本机 HTTP`，`scan` 只返回 `book/rust` 那条，导出应有两条数据和表头。退出终端后重新运行 `get`，确认跨进程持久化。再次导出到相同文件应拒绝覆盖；另存新文件才能得到更新快照。

## 4. 交付与解释

在 **学员目录**运行 `cargo test --offline` 回归全部测试。保存数据库副本、TSV 和跨进程运行记录。把预测、错误和解释写入 `LEARNING_LOG.md`，再补一个自己的边界测试。

用自己的话回答：为什么日志写失败时内存也必须不变？为什么 `scan` 返回引用而 Get 返回独立 String？仍在使用扫描结果时修改数据库，为什么会触发借用冲突？

测试通过只说明已检查的行为符合合同；个人输入、真实产物和解释能帮助你判断学到了什么。

架构为 CLI → `Database<J>` → `Journal`；文件头 `RUSTAPPLAB_MINIKV_V1`，记录用 Tab 分隔的 P/D 与十六进制键值。十六进制不是加密。键限 128 字节、值限 65536 字节、日志限 16 MiB；无日志压缩、多键事务或断电一致性保证。

正常 Drop 释放 `.lock`，异常退出会保守保留；先确认所有实例已退出再处理锁。损坏日志拒绝打开，不自动修复。Windows/macOS 原生复验和声明的 Rust 1.74 MSRV 验证仍待补；不要保存密码或唯一重要副本。

三步完成后，可在学员目录运行 `cargo test --offline --doc` 观察预期编译失败的借用示例，或 `cargo run --offline -- share-demo new-shared.kv` 观察 `workers=4 records=40 expected=40`；后者使用 Arc/Mutex 共享同一数据库。

## 下一款：NetFetch

接下来把已有的错误处理、字节和文件知识用于本机 HTTP 下载，先学同步客户端。在 **RustAppLab 根目录**运行（若仍在学员目录，先 `cd ..`）：

```powershell
cargo run --offline -p labquest -- studio start netfetch my-netfetch
cargo run --offline -p labquest -- learn lesson my-netfetch 1
```

继续阅读 [NetFetch README](../netfetch/README.md)。
