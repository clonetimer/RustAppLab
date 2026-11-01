# StoreDesk · 为社团物料做一个 SQLite 库存台

## 目标：自己的程序、真正的数据库文件

教学场景：你有 10 支学习用笔和 3 本练习本，需要登记物品、调整数量、检查变化流水。一次领用了笔却发现书不够时，这一批操作应该全部取消，而不是只留下半次领取。

完成后，你会拥有可重复使用的 `storedesk` 程序，以及一个真实的 `inventory.db` SQLite 数据库。它不是把 TSV 换一个 `.db` 后缀，也不需要运行远程数据库服务器。

**先修要求：**完成 LedgerLite 的记录建模和持久化，理解 `Result` 与 `?`。初学者不用学习或修改底层 FFI：`src/sqlite.rs` 是已提供的有限接口，三个练习仍只在 `src/lib.rs`。本课的交付范围是 SQL、参数绑定、事务与版本检查；成熟第三方 crate 的选型和依赖管理仍需后续课程，不能把使用这个适配器算作掌握整个 Rust 数据库生态。

## 先检查环境，不要把链接错误误当作练习失败

在完整 RustAppLab 根目录运行：

```powershell
cargo run --offline -p storedesk -- doctor
```

成功会显示实际 SQLite 版本、backend 和 `schema=1`，版本号由你的系统决定。Windows MSVC 使用 Windows SDK 的 `winsqlite3`，Linux 使用系统 SQLite；不声明 macOS 支持。程序不附带 SQLite 引擎，也不在线下载 Rust crate。

出现 `cannot find -lsqlite3` 或 `LNK1104 ... winsqlite3.lib` 时，先阅读完整包的 `course/SQLITE_SETUP.md`。这是原生链接依赖问题，不是 `TODO` 答案错误。已确认的 0.11.1 Windows 通过情况不能代替这款新应用的原生验证。

## 先看完成后的软件

以下在完整平台根目录执行，演示目录必须是新的：

```powershell
cargo run --offline -p storedesk -- demo my-store-demo
cargo run --offline -p storedesk -- list my-store-demo/inventory.db
cargo run --offline -p storedesk -- history my-store-demo/inventory.db
```

你应看到 BOOK 为 4、PEN 为 8，并有两条变化流水。演示目录中的 `INVENTORY_REPORT.txt` 可以直接阅读；`inventory.db` 是二进制数据库，不要用记事本修改。运行 `adjust ... PEN -100` 会报错，再次 list 应仍为 8。

## 创建自己的学员工程

在 **RustAppLab 根目录**：

```powershell
cargo run --offline -p labquest -- studio start storedesk my-storedesk
cargo run --offline -p labquest -- studio check my-storedesk 1
```

首次失败正常。打开 `my-storedesk/START.html` 和 `my-storedesk/src/lib.rs`。`main.rs` 已经连接命令行；`sqlite.rs` 负责释放连接与语句、绑定参数、事务守卫；本次核心任务是校验、保存物品、提交一批库存变化。

## 第 1 步：把需求写成可执行的校验

修改 **TODO(1)，`validate_item`**。

`Item` 是拥有数据的结构体：`sku` 是物品编号、`name` 是可显示名称、`stock` 是整数库存。借用 `&Item` 就能检查，不需要拿走整个物品，也不需要访问数据库。

| 字段 | 合同 |
|---|---|
| SKU | 1–32 字节，只允许 ASCII 字母、数字、`-_.` |
| 名称 | 去掉首尾空白后不为空，总长不超过 80 个字符，不含控制字符 |
| 库存 | 0 到 1,000,000,000，包括边界 |

名称可以含中文和单引号，例如 `Alice's 钢笔`。禁止单引号并不是正确的 SQL 防注入方法；下一步要把它当参数处理。SKU 的 byte 长度与中文名称的 char 数量是两种不同约束，先回忆 TextScope 学过的区别。

`Result<()>` 的成功值是 `()`，意思是“检查通过，不额外返回数据”；不是没有返回值。如果发现错误，尽早返回 `Err`；所有检查通过后再 `Ok(())`。

进入学员目录：

```powershell
cargo test --offline --test milestones step1_
```

预期为两项通过。自己补一个恰好 80 字的名称和一个负库存案例。仅通过第 1 步并不要求后两步已经实现。

## 第 2 步：用参数保存，不把用户字符串拼入 SQL

修改 **TODO(2)，`save_item`**。先 `validate_item(item)?`，再向已提供的 `items` 表插入一条记录。SKU 是主键，同一 SKU 再次添加必须报错，不能 `REPLACE` 覆盖旧库存。

表结构：`items(sku TEXT PRIMARY KEY, name TEXT NOT NULL, stock INTEGER NOT NULL)`，实际建表还有库存范围约束。底层提供 `db.execute(sql, params)`，返回受影响行数或错误；`Param::Text` 与 `Param::Int` 区分参数类型。需要时在文件顶部加入 `use sqlite::Param;`。

你需要的 SQL 形状是：

```sql
INSERT INTO items(sku,name,stock) VALUES (?1,?2,?3)
```

把 `&item.sku`、`&item.name`、`item.stock` 按顺序交给参数列表。不要写 `format!("INSERT ... '{}'", item.name)`。SQL 中的 `?1` 是占位符；Rust 调用末尾的 `?` 是错误传播运算符，两个问号不是同一机制。

已提供的 `list_items` 使用 SELECT 读取数据并按 SKU 排序。阅读它，观察为什么数据库返回的临时内容需要转换成拥有数据的 String，然后才能组成返回的 `Vec<Item>`。

```powershell
cargo test --offline --test milestones step2_
```

这两项测试会真正写入磁盘、关闭连接再打开，并检查 SQLite 文件头。还会保存一个看起来像 SQL 的名称，确认它只是普通数据而没有删除表。

从平台运行 `studio check my-storedesk 2`，同时复查第 1 步。错误常见于：没有导入 `Param`；参数顺序错；没有调用验证；错误地用 REPLACE 把重复添加变成了覆盖。

## 第 3 步：一批变化必须一起成功或一起失败

修改 **TODO(3)，`apply_changes`**。输入是借用的 `&[Change]`，每项包含 SKU 和有正负号的整数变化量。一次必须有 1–1000 项；每项变化非零，范围不超过正负 1,000,000,000；SKU 要合法。

先预测：PEN 初始 10，一批先 `-2` 再 `-99`，最后应该是 8、10 还是负数？本合同要求 **10**：第二项失败，第一项与对应流水也必须撤销。

提供的事务接口形状是：

```rust
// 示意，内部业务需要你完成。
db.transaction(|tx| {
    // 逐条调整库存，并为每次变化插入一行 movements。
    // 任意失败使用 Err 返回，全部成功后 Ok(())。
    Ok(())
})
```

这是“接收一个闭包”的函数。事务开始后，你在闭包里使用 `tx: &Db`；外部 `db` 被可变借用，在事务结束之前不应再从别处操作它。这个限制帮助表达一段有明确开始与结束的工作。

库存更新必须把范围条件放进同一条 UPDATE：只有目标 SKU 存在且 `stock+变化量` 仍在合法范围内才改变。不要先查询旧数值、离开事务，再写回结果。受影响行数不是 1 时返回错误。随后往 `movements(sku,delta)` 插入流水，使用同一个 `tx`。**流水不能在事务外单独写。**

正常返回会 COMMIT；`Err` 会 ROLLBACK；底层的 Drop 守卫也会在 Rust 栈展开时尝试回滚。不要把这个机制称作任意断电场景都不会丢数据：本课没有物理断电验证。

```powershell
cargo test --offline
```

完整分步合同为 8 项。最后两项启动真实程序，在不同进程间读取同一数据库，并确认失败批次的退出码、库存和流水都正确。

## 把自己的物品录进去

在学员工程目录运行；`my-items.db` 必须还不存在：

```powershell
cargo run --offline -- init my-items.db
cargo run --offline -- add my-items.db PEN "我的学习用笔" 10
cargo run --offline -- add my-items.db BOOK "Rust 练习本" 3
cargo run --offline -- adjust my-items.db PEN -2
cargo run --offline -- list my-items.db
cargo run --offline -- history my-items.db
```

重开 PowerShell，再次进入同一目录执行 list，确认不是仅保存在内存。运行 `adjust my-items.db PEN -99`，观察明确失败后数据不变。数据库路径拼错时，list/adjust 不会自动创建新库；只有 init 或明确的演示创建流程能新建。

把下面两列写入 `samples/my-changes.tsv`，列间是 Tab，不是若干空格：

```text
PEN	-1
BOOK	2
```

执行 `cargo run --offline -- apply my-items.db samples/my-changes.tsv`。为自己的数据生成一份文本报告并填写 `LEARNING_LOG.md`。程序的 list 输出可以重定向保存，但不同 PowerShell 版本的重定向文本编码可能不同；数据库本身仍是 SQLite 文件。

## 版本与安全边界

数据库有 `application_id` 身份标记和 `user_version=1`。陌生库、标记异常和更高版本会拒绝打开，不自动降级。这只是 v1 初始化与版本保护，**没有实现 v1→v2 数据迁移**。

当前开发版 Windows 原生验证仍待完成；不声明 macOS；本课使用系统 SQLite，实际引擎版本由系统决定。自己写的有限 FFI 适配器不是生产级通用驱动，不支持 BLOB、浮点结果、扩展加载和任意大查询；成熟 rusqlite/SQLx 的使用与依赖治理是后续课程，不算已完成。

## 独立扩展与验收

新增 `low-stock <数据库> <阈值>`：只读查询库存低于阈值的物品，按 SKU 排序。先写两个测试，至少包含“没有低库存”和“阈值恰好等于库存”的情况，然后再实现。不要为查询改变库存。

完成后交付 `target/release/storedesk.exe`（Windows）或对应平台程序、你的数据库、运行说明和学习记录。应能解释：为什么必须绑定参数？为什么库存与流水必须放在同一事务？为什么“代码里写了事务”还需要故意制造第二条失败的测试？

## 完成三步后的故障实验

继续平台根目录 RELIABILITY_PATH.html：先预测第二条失败会保留什么，再观察库存和流水。该讲义区分参考演练与自己的实现，提供真实 Tab 样例；无需重建已有学员工程。
