# LedgerLite · 本地生活账本

**本作品的中心思想**：用一款可运行的小软件，把三个连续能力连接起来，而不是完成三道无关语法题。

## 开始前只确认这件事

你当前应位于这个学员工程的目录，能够看到 `Cargo.toml`。终端中的 `cargo` 命令默认在这里运行。`src/main.rs` 是已经接好的命令行外壳；本轮只修改 `src/lib.rs` 的 TODO(1)、TODO(2)、TODO(3)。`tests/milestones.rs` 是公开的验收合同，不能通过删除测试“通关”。

在完整 RustAppLab 根目录创建本工程的命令是 `cargo run --offline -p labquest -- studio start ledgerlite my-ledgerlite`，然后进入 `my-ledgerlite`。本讲义也随学员包提供，包内已有工程就不必再次生成。Windows 可以用 `CONTINUE_WINDOWS.cmd` 选择作品。

三个里程碑依次是：**用整数分精确表示金额 → 解析并验证可持久化记录 → 借用记录并按分类累计**。未实现的后两步不会阻止你运行第一步测试；语法错误仍会阻止整个库编译。

## 先看你要做出的软件

做一本放在本机 TSV 文件里的生活账本：新增支出后关掉终端，再打开仍能汇总；中文分类可以直接阅读。完成后在本工程目录运行：

```powershell
cargo run --offline -- summary samples/expenses.tsv
cargo run --offline -- add my-book.tsv 2026-10-28 餐饮 12.50
cargo run --offline -- add my-book.tsv 2026-10-28 餐饮 0.10
cargo run --offline -- summary my-book.tsv
```

自己的账本应显示餐饮 `12.60`、`total=12.60`。这些是演示记录，不是建议支出或会计凭证。基本版只记录非负金额，不支持退款、转账或多币种。

## 第 1 步：金额不能只图方便用 f64

打开 `src/lib.rs` 的 `parse_cents`。本作品用整数“分”记录金额：12.50 是 1250，0.1 是 10。输入按小数点拆分，最多两部分；整数部分必须至少一位数字，小数部分若出现，只允许一位或两位。

一位小数补到十分位，例如 1.2 的小数部分是 20 分。`-1`、`1e2`、`NaN`、`1.234`、`1.` 都应返回错误。不要先 parse 为 f64 再乘 100 四舍五入，那会引入不必要的二进制浮点问题。先解析整数，再用 `checked_mul(100)` 和 `checked_add` 检查范围。

```powershell
cargo test --offline --test milestones step1_
```

`0.1 + 0.2` 在我们的表示里就是 `10 + 20 = 30`。解释“屏幕显示 0.30”和“内部准确保存 30”为什么是两件事。

## 第 2 步：数据文件不是可以随便相信的字符串

每行严格为 `YYYY-MM-DD<TAB>分类<TAB>金额`。TODO(2) 把文本转为 `Vec<Entry>`；Entry 的日期和分类是 String，金额为整数分。日期校验辅助函数 `valid_day` 已提供，你现在不必同时学习日历算法。

处理 Windows CRLF 和首行 UTF-8 BOM。空文件是合法空账本；格式错误的非空行不是零元记录，必须返回含行号的错误。分类去掉前后空白，不能为空、不能含控制字符。日期要真实存在，2024-02-29 合法，2023-02-29 不合法。

```powershell
cargo test --offline --test milestones step2_
```

注意暂存借用与返回拥有值的区别：split 得到的字段借用原输入，存入 Entry 时取得自己的 String；不要让返回的数据依赖函数内部临时拼接的字符串。

## 第 3 步：分类聚合与错误传播

TODO(3) 借用所有 Entry，通过 `BTreeMap<String,u64>` 累加同分类金额，输出按分类有序。`checked_add` 返回 Option：成功时取出总额，失败时把 None 转成有意义的 Err。不能在溢出时回到零，也不能悄悄截断数据。

```powershell
cargo test --offline --test milestones step3_
cargo run --offline -- demo
```

demo 应有分类结果，最后为 `total=15.60`。全部通过后，读 `src/main.rs` 的 add 分支：先验证新输入，再完整验证旧文件，最后才打开文件追加。业务验证失败不能先把旧文件截断。

## 三个真实验收实验

第一次新增你的分类并关闭终端，再启动 summary，验证跨进程持久化。第二次用不存在的日期新增，确认失败后原文件字节不变。第三次复制一份账本，在副本制造坏行，再尝试 add，确认错误不会被略过。

程序用追加写入，不保证多个进程同时写的隔离，也不保证中途 I/O 故障或断电的事务回滚。不要把这款教学小账本用于关键账务。后续数据库作品会解决事务与迁移；这里先弄懂文件与 Result。

## 交付与扩展

交付可执行程序、一份虚构样例账本和 README（命令、文件格式、限制）。用自己的日期做过滤是扩展任务；先定义闭区间还是开区间，再写测试，不要让边界含义靠猜。

参考：Rust 官方错误处理 https://doc.rust-lang.org/book/ch09-00-error-handling.html 和集合 https://doc.rust-lang.org/book/ch08-00-common-collections.html 。

## 一次只检查当前阶段

本工程内可以运行 `cargo test --offline --test milestones step1_`。完成一步，再运行 step2_、step3_，最后执行全量 `cargo test --offline`。完整平台还提供权威检查：回到 RustAppLab 根目录执行 `cargo run --offline -p labquest -- studio check "my-ledgerlite" 1`，把最后的 1 改成 2、3 或 all。它会用自带的测试检查当前源码，不信任你编辑后的本地 tests。第 2 步会重新检查第 1 步，第 3 步会重查前两步。

检查会真实编译并运行你写的 Rust，只用于你信任的自编代码，不是安全沙箱。报告按检查次数生成新文件，包含当前源文件摘要；旧报告不能证明修改后的代码仍正确。遇到错误先看第一个 FAIL 和对应函数，再看 HINTS.md。

## 完成标准

功能测试通过只是其中一项。还需用自己的数据运行一次，给一个错误输入，保存真实产物或控制台结果，并在 LEARNING_LOG.md 解释一个关键 Rust 概念。然后为未提供的变体自行增加一个测试。不要把参考程序已通过等同于自己已学会。
