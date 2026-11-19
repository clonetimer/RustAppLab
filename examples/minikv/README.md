# MiniKV 数据与历史运行产物

本目录提供**可选参考产物**，用于观察持久化、前缀迭代、导出和损坏检测；不是学员答案或个人成绩。`EVIDENCE.json` 标记的是 **0.14.0 的历史运行记录**，其中的旧机器路径和退出码不代表当前提交已经重新验收。

## 先理解什么

建议完成 TextScope、BadgeForge、BatchMeter，能区分 `String` 与 `&str`，见过 trait、线程和 `move`。具体三步练习见 [MiniKV 讲义](../../course/studio/minikv.md)。FirstCard 初学者请先看 [基础后续指南](../../course/AFTER_FIRSTCARD.md)。

| 文件 | 用途与边界 |
|---|---|
| `我的 书签.kv` | 可读取的参考日志；最终含 `book/rust` 与 `project/next` 两个键 |
| `my-product.tsv` | 当时导出的两条最终记录；后续修改数据库不会自动更新此文件 |
| `four-workers.kv` | 四线程共享写入的历史产物，记录中报告最终 40 项 |
| `truncated.kv` | 故意截断的错误输入；应被识别为损坏，不能当正常库使用 |
| `EVIDENCE.json` | 历史命令、退出码和独立解析结论，供核对来源 |

## 先读取现有样例

在 **RustAppLab 源码根目录**运行；含中文和空格的路径用引号：

```powershell
cargo run --locked --offline -p minikv -- get "examples/minikv/我的 书签.kv" book/rust
cargo run --locked --offline -p minikv -- scan "examples/minikv/我的 书签.kv" book/
cargo run --locked --offline -p minikv -- scan examples/minikv/truncated.kv
```

第一条应显示“已经理解借用”；第二条只列出 `book/` 前缀的键。第三条**应失败**，指出日志损坏，且不自动修复原文件。这些是参考程序行为，不是要通过修改坏样例消除的报错。

要观察导出，仍在 **平台根目录**选择一个不存在的输出文件名：

```powershell
cargo run --locked --offline -p minikv -- export "examples/minikv/我的 书签.kv" my-minikv-export.tsv
```

导出含表头和两条记录。重复导出到同一文件应拒绝覆盖，保留原文件。

## 转为自己的练习

平台创建个人工程的命令是：

```powershell
cargo run --locked --offline -p labquest -- studio start minikv my-minikv
cargo run --locked --offline -p labquest -- learn lesson my-minikv 1
cargo run --locked --offline -p labquest -- learn check my-minikv 1
```

只编辑自己工程 `src/lib.rs`，按 `validate_key` → `Database::execute` → `PrefixScan::next` 完成三个检查点。初次 TODO 失败正常，固定合同检查与个人使用实验都要做。

三步通过后，在 **自己的 MiniKV 工程目录**运行，使用新的数据库和导出文件名：

```powershell
cargo run --offline -- put my-bookmarks.kv book/rust "我对借用的解释"
cargo run --offline -- put my-bookmarks.kv course/next "下一步学习 NetFetch"
cargo run --offline -- scan my-bookmarks.kv book/
cargo run --offline -- export my-bookmarks.kv my-bookmarks.tsv
```

新进程读取时，数据仍应存在。再更新同一个键：预测日志是否增加记录、扫描是否只显示最新值；重新导出时另选文件名。不要直接改写本目录中的历史产物。

## 学习自查与下一步

提交自己的 `.kv`、TSV、一次错误记录与一个新增测试，并解释：日志与当前状态有什么区别？前缀迭代为什么借用数据？写入失败时为什么不能提前改变内存？

日志编码不是加密，教学数据库也不保证断电一致性或多进程安全。故意损坏实验只在副本上做。

完成后按 [系统应用路线](../../course/SYSTEMS_PATH.md) 进入 [NetFetch](../../course/studio/netfetch.md)，把本地数据处理扩展到获取真实回环 HTTP 数据；保留自己的索引与学习记录。
