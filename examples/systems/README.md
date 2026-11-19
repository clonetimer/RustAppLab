# 个人工单档案的学习记录入口

本目录只提供 **`LEARNING_LOG_TEMPLATE.md` 记录模板**，属于可选辅助材料；没有预生成的成功记录、工单数据库、快照或学员核心答案。实际产物由自己的程序和操作生成。

## 什么时候使用

按 [系统应用路线](../../course/SYSTEMS_PATH.md) 学习 BatchMeter → MiniKV → NetFetch → ServiceDesk 时使用。先能用 `Result` 区分成功与失败、解释 `String` / `&str`，完成 TextScope、LedgerLite、TermGarden 的基本操作。

刚完成 FirstCard 时，先按 [后续指南](../../course/AFTER_FIRSTCARD.md) 进入 UnitLab。本段不需要 Python、Playwright、云服务器或下载第三方 crate。

## 记录怎样对应软件产物

| 软件 | 你要做的事 | 应保存的真实证据 |
|---|---|---|
| BatchMeter | 串行理解后使用有界线程统计文件 | 两种 worker 数的逐文件结果和相同汇总 |
| MiniKV | 保存并重新读取快照路径索引 | 自己的 `.kv`、前缀扫描和一次失败记录 |
| NetFetch | 从本机 HTTP 服务保存响应字节 | 新路径下的 `before.json` / `after.json` |
| ServiceDesk | 创建、关闭工单，再重启服务 | 日志、工单状态和重启后仍存在的记录 |

完整联合演练需要两个终端，步骤见系统应用讲义。快照是不同时间的输入，不是直接复制模板里的“正确结果”。

## 先留一个独立记录目录

在 **RustAppLab 源码根目录**运行以下 PowerShell 命令：

```powershell
if (Test-Path -LiteralPath "my-systems-notes") { throw "目录已存在，请换一个新名字。" }
Copy-Item -Recurse -LiteralPath "examples/systems" -Destination "my-systems-notes"
```

打开 `my-systems-notes/LEARNING_LOG_TEMPLATE.md`，先填要验证的输入与预测；实际运行后再填结果。已有记录请保留，新实验用新目录。

只有学员包时，可在包根目录复制这份模板；从包内 `Studio/batchmeter` 的 README / TASK_CARDS 继续，跳过下面需要完整平台的创建命令。

## 从当前作品的一个检查点开始

还没有 BatchMeter 工程时，在 **平台根目录**创建；已有工程就直接用原路径继续 lesson / check：

```powershell
cargo run --locked --offline -p labquest -- studio start batchmeter my-batchmeter
cargo run --locked --offline -p labquest -- learn lesson my-batchmeter 1
cargo run --locked --offline -p labquest -- learn check my-batchmeter 1
```

打开自己工程的 `src/lib.rs`，一次实现一个 TODO。初次检查失败正常；逐步完成 1、2、3，不修改平台参考代码来代替个人作品。

三步完成后，在 **自己的 BatchMeter 工程目录**运行：

```powershell
cargo run --offline -- scan 1 samples/one.txt samples/two.txt
cargo run --offline -- scan 4 samples/one.txt samples/two.txt
```

未修改的默认样例应合计 2 文件、36 字节、5 行，两次逐文件结果与汇总一致。再用自己的文件预测结果，最后把一个路径改为不存在；应报错，不能悄悄少算一份。

## 形成个人工单档案

继续按讲义完成 MiniKV、NetFetch、ServiceDesk 的学员检查，再用自己的实现替换参考演练。先保存全部未完成的快照，关闭一项工单后另存第二份；通过 BatchMeter 统计文件，通过 MiniKV 保存路径索引，停服重启后核对工单状态。

输出路径已存在时换新名称；快照由 NetFetch 按字节写入，避免用 PowerShell 5.1 的 `>` 改变编码。不要手工预填“已成功”的 JSON 或索引来代替真实操作。

## 自查与下一步

你的记录应能回答：哪个程序负责哪一步？失败属于文件、HTTP、业务还是持久化？为什么两种 worker 数结果相同不能单独证明代码真的并发？为什么网页显示完成后仍要重启验证？

保留自己的源码、快照、索引、日志、一次异常处理和一个新增测试。系统应用讲义提供各作品的源码位置与三级提示，模板只保存你实际做过的过程。

完成本段后进入 [事务与等待路线](../../course/RELIABILITY_PATH.md)，用 StoreDesk 学整批回滚、用 AsyncBoard 学 Future / Waker；已有档案保留，不必重建。这些同步网络与教学存储不声明公网 / TLS、生产数据库或断电一致性。
