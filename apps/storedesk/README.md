# StoreDesk · 让库存与流水一起成功或回滚

**这次要做成什么：**用真实 SQLite 文件登记物品、调整库存、查询流水；一批操作中任何一条失败，库存与流水都不留下半次结果。

**开始条件：**完成 [ServiceDesk](../servicedesk/README.md)，并理解 LedgerLite 的记录建模、`Result` 与 `?`。本课提供有限 SQLite 接口，不要求先修改底层 FFI。

本目录是已实现的参考软件；你的练习代码放在下面新建的 `my-storedesk/` 中。

## 1. 看目标，创建自己的工程

在 **RustAppLab 根目录**（能看到根 `Cargo.toml`）运行：

```powershell
cargo run --offline -p storedesk -- doctor
cargo run --offline -p storedesk -- demo reference-stock
cargo run --offline -p labquest -- studio start storedesk my-storedesk
cargo run --offline -p labquest -- learn lesson my-storedesk 1
cargo run --offline -p labquest -- learn check my-storedesk 1
```

`doctor` 显示系统实际 SQLite 版本、backend 和 `schema=1`；演示中 BOOK 库存 4、PEN 库存 8，`audit_rows=2`。若缺 `sqlite3` 或 `winsqlite3.lib`，先看 [SQLite 环境说明](../../course/SQLITE_SETUP.md)，这是链接依赖问题。

学员首次检查因 TODO 未实现而 FAIL 是正常的。先看第一条失败；若是编译器或链接器找不到，则应先修好环境。

打开 `my-storedesk/START.html` 或 `TASK_CARDS.html`；只编辑 **`my-storedesk/src/lib.rs`**，每次一个 TODO。命令行外壳 `src/main.rs` 已提供，先不改。目录已存在就继续使用；新建另一份练习时换名称。

## 2. 三个检查点

| 步骤与函数 | 要完成的行为 | 本次概念 |
|---|---|---|
| TODO(1) · `validate_item` | 检查 SKU、名称、库存，校验期间不写数据库 | 业务约束、纯函数、`Result` |
| TODO(2) · `save_item` | 使用绑定参数 INSERT，重复 SKU 必须失败 | 参数化 SQL、复用校验、错误传播 |
| TODO(3) · `apply_changes` | 整批调整与流水在同一事务内提交或回滚 | 闭包、`&mut Db`、事务、失败一致性 |

每步先写一组输入和预测，再修改、保存。第一步通过后，在 **平台根目录**继续：

```powershell
cargo run --offline -p labquest -- learn lesson my-storedesk 2
cargo run --offline -p labquest -- learn check my-storedesk 2
cargo run --offline -p labquest -- learn lesson my-storedesk 3
cargo run --offline -p labquest -- learn check my-storedesk 3
```

后一步会累计回归前面的步骤。提示用 `cargo run --offline -p labquest -- learn hint my-storedesk 1`；进度用 `cargo run --offline -p labquest -- learn next my-storedesk`。详细解释见 [storedesk 讲义](../../course/studio/storedesk.md)。

## 3. 用自己的输入完成一次作品

三步通过后，在全新数据库里只登记 PEN。先预测：提供的 `samples/changes.tsv` 依次调整 PEN 和尚未登记的 BOOK，第二条失败后，第一条变化应该保留吗？

从 **平台根目录进入学员目录**执行：

```powershell
cd my-storedesk
cargo run --offline -- init my-items.db
cargo run --offline -- add my-items.db PEN "我的学习用笔" 10
cargo run --offline -- adjust my-items.db PEN -2
cargo run --offline -- list my-items.db
cargo run --offline -- history my-items.db
cargo run --offline -- apply my-items.db samples/changes.tsv
cargo run --offline -- list my-items.db
cargo run --offline -- history my-items.db
```

第一次调整后库存为 8，流水只有 `PEN -2` 一条。批次先尝试 PEN 再减 2，随后因 BOOK 不存在而失败；最后两次查询仍应是库存 8、一条流水，说明第一条变化也被回滚。再复制清单、改成自己的库存变化，先预测再运行；两列之间保留实际 Tab。

## 4. 交付与解释

在 **学员目录**运行 `cargo test --offline` 回归全部测试。保存个人数据库、成功与失败的库存/流水结果。把预测、错误和解释写入 `LEARNING_LOG.md`，再补一个自己的边界测试。

用自己的话回答：为什么用户名称必须绑定为 SQL 参数？为什么库存和流水要在同一事务？为什么看到代码中有 transaction，还要实际制造第二条失败？

测试通过只说明已检查的行为符合合同；个人输入、真实产物和解释能帮助你判断学到了什么。

SKU 1–32 位 ASCII 字母、数字、`-_.`；名称最多 80 个字符；库存范围 0–1000000000；一批 1–1000 条变化，查询最多 10000 行。数据库有 application_id 与 schema v1 标记，拒绝陌生或更高版本数据库，没有实现版本迁移。

程序使用系统 SQLite：Windows MSVC 需要 SDK 的 winsqlite3，Linux 需要 SQLite 开发库；Windows 原生验证仍待完成，不声明 macOS 支持。提供的 FFI 适配器不代表已掌握 rusqlite/SQLx，也不是通用生产驱动。

## 下一款：AsyncBoard

接下来在理解失败与提交之后学习等待，区分同时等待、线程阻塞和真正的 `.await`。在 **RustAppLab 根目录**运行（若仍在学员目录，先 `cd ..`）：

```powershell
cargo run --offline -p labquest -- studio start asyncboard my-asyncboard
cargo run --offline -p labquest -- learn lesson my-asyncboard 1
```

继续阅读 [AsyncBoard README](../asyncboard/README.md)。
