# LedgerLite · 把文字账目变成可核对的金额

**这次要做成什么：**追加一条开销，退出后仍能读取，按分类精确汇总。保存一份练习账本，并证明非法记录不会被当成正常金额。

**开始条件：**完成 [TextScope](../textscope/README.md)，能读懂逐行解析、`Result` 和集合。请用虚构开销练习。

本目录是已实现的参考软件；你的练习代码放在下面新建的 `my-ledgerlite/` 中。

## 1. 看目标，创建自己的工程

在 **RustAppLab 根目录**（能看到根 `Cargo.toml`）运行：

```powershell
cargo run --offline -p ledgerlite -- demo
cargo run --offline -p labquest -- studio start ledgerlite my-ledgerlite
cargo run --offline -p labquest -- learn lesson my-ledgerlite 1
cargo run --offline -p labquest -- learn check my-ledgerlite 1
```

参考演示中餐饮合计 12.60、交通 3.00，总额 `total=15.60`。

学员首次检查因 TODO 未实现而 FAIL 是正常的。先看第一条失败；若是编译器或链接器找不到，则应先修好环境。

打开 `my-ledgerlite/START.html` 或 `TASK_CARDS.html`；只编辑 **`my-ledgerlite/src/lib.rs`**，每次一个 TODO。命令行外壳 `src/main.rs` 已提供，先不改。目录已存在就继续使用；新建另一份练习时换名称。

## 2. 三个检查点

| 步骤与函数 | 要完成的行为 | 本次概念 |
|---|---|---|
| TODO(1) · `parse_cents` | 把 `0.10` 转成整数 10 分；拒绝负数、多余小数和溢出 | 字符串拆分、`Result`、检查整数运算 |
| TODO(2) · `parse_entries` | 日期、分类、金额三列转为 `Vec<Entry>`，定位坏行 | 结构体、所有权、复用校验与 `?` |
| TODO(3) · `totals` | 按分类有序汇总，发现累计金额溢出 | `BTreeMap`、可变借用、`checked_add` |

每步先写一组输入和预测，再修改、保存。第一步通过后，在 **平台根目录**继续：

```powershell
cargo run --offline -p labquest -- learn lesson my-ledgerlite 2
cargo run --offline -p labquest -- learn check my-ledgerlite 2
cargo run --offline -p labquest -- learn lesson my-ledgerlite 3
cargo run --offline -p labquest -- learn check my-ledgerlite 3
```

后一步会累计回归前面的步骤。提示用 `cargo run --offline -p labquest -- learn hint my-ledgerlite 1`；进度用 `cargo run --offline -p labquest -- learn next my-ledgerlite`。详细解释见 [ledgerlite 讲义](../../course/studio/ledgerlite.md)。

## 3. 用自己的输入完成一次作品

三步通过后，用一个尚不存在的 `my-book.tsv`。注意重复执行 `add` 会真实追加一条记录。

从 **平台根目录进入学员目录**执行：

```powershell
cd my-ledgerlite
cargo run --offline -- add my-book.tsv 2026-11-19 餐饮 0.10
cargo run --offline -- add my-book.tsv 2026-11-19 餐饮 0.20
cargo run --offline -- summary my-book.tsv
cargo run --offline -- add my-book.tsv 2023-02-29 餐饮 1.00
cargo run --offline -- summary my-book.tsv
```

两次 `summary` 都应有餐饮 `0.30`、`total=0.30`。不存在的日期应报错，文件仍只有两行。退出终端重新运行 `summary`，确认数据来自磁盘；另选新账本记录自己设计的三条虚构开销。

## 4. 交付与解释

在 **学员目录**运行 `cargo test --offline` 回归全部测试。保存练习账本、成功与失败命令及两次汇总。把预测、错误和解释写入 `LEARNING_LOG.md`，再补一个自己的边界测试。

用自己的话回答：为什么金额用整数分？为什么要验证整份旧账本后才能追加？累计值溢出时为什么不能输出绕回的小数值？

测试通过只说明已检查的行为符合合同；个人输入、真实产物和解释能帮助你判断学到了什么。

TSV 每行是 `YYYY-MM-DD<TAB>分类<TAB>金额`；金额非负、最多两位小数。没有并发写入锁、事务、断电恢复保证或日期筛选。

## 下一款：BadgeForge

接下来把数据变成能打开的 SVG，学习输出转义、trait 与泛型。在 **RustAppLab 根目录**运行（若仍在学员目录，先 `cd ..`）：

```powershell
cargo run --offline -p labquest -- studio start badgeforge my-badgeforge
cargo run --offline -p labquest -- learn lesson my-badgeforge 1
```

继续阅读 [BadgeForge README](../badgeforge/README.md)。
