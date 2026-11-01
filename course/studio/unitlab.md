# UnitLab · 随身单位换算器

**本作品的中心思想**：用一款可运行的小软件，把三个连续能力连接起来，而不是完成三道无关语法题。

## 开始前只确认这件事

你当前应位于这个学员工程的目录，能够看到 `Cargo.toml`。终端中的 `cargo` 命令默认在这里运行。`src/main.rs` 是已经接好的命令行外壳；本轮只修改 `src/lib.rs` 的 TODO(1)、TODO(2)、TODO(3)。`tests/milestones.rs` 是公开的验收合同，不能通过删除测试“通关”。

在完整 RustAppLab 根目录创建本工程的命令是 `cargo run --offline -p labquest -- studio start unitlab my-unitlab`，然后进入 `my-unitlab`。本讲义也随学员包提供，包内已有工程就不必再次生成。Windows 可以用 `CONTINUE_WINDOWS.cmd` 选择作品。

三个里程碑依次是：**写出温度转换公式 → 用 enum 和 match 区分单位 → 连接 parse、Result 与格式化输出**。未实现的后两步不会阻止你运行第一步测试；语法错误仍会阻止整个库编译。

## 先看你要做出的软件

你在看天气预报时想把摄氏温度转换成华氏温度，或把跑步距离从米换成千米。目标不是背公式，而是做一款能接收不同输入、拒绝错误输入的小工具。完成后在本工程目录运行：

```powershell
cargo run --offline -- convert 25 c f
cargo run --offline -- convert 1500 m km
```

预期分别为 `25.00 c = 77.00 f`、`1500.00 m = 1.50 km`。先预测 `-40 c f` 的结果，不要急着运行。这里 c/f 表示温度，m/km 表示长度，不支持其它单位，也不做物理温度下限判断。

## 第 1 步：只写一个会计算的函数

打开 `src/lib.rs`，找到 `pub fn celsius_to_fahrenheit(celsius: f64) -> f64`。今天先不要研究 `main.rs`。`fn` 声明函数；括号内是输入；`f64` 是浮点数类型；箭头后是返回值类型。`pub` 表示其它模块能够调用它。把 TODO(1) 中的占位代码替换成公式 `摄氏度 × 9 ÷ 5 + 32`。用 `9.0`、`5.0`、`32.0` 表示浮点字面量。

函数最后一个表达式不加分号时，它的结果就是返回值。不要把 `println!` 当作返回值：它只是把文字输出到屏幕，不能把数交还给调用者。这里不需要 `mut`，因为不必修改原输入。

```powershell
cargo test --offline --test milestones step1_
```

看到 `2 passed; 0 failed` 再进入下一步。第一次可能失败，这是占位函数的正常状态。`step1_freezing` 对应零摄氏度；另一个测试会用负数和小数，不接受只返回 `32.0`。

**自己解释**：函数调用 `celsius_to_fahrenheit(100.0)` 中，100.0 到了哪个变量里？末尾加上分号为什么改变含义？在 `LEARNING_LOG.md` 用自己的话写两句。

## 第 2 步：一个函数支持多种转换

仍然只改 `src/lib.rs` 的 TODO(2)。现有 `Unit` 枚举已经为你定义四种合法单位，不需要自己新增类型。`match (from, to)` 匹配一对单位，分别处理摄氏→华氏、华氏→摄氏、米→千米、千米→米和相同单位。华氏转摄氏的顺序是先减 32，再乘 5/9。

函数返回 `Result<f64, String>`：成功用 `Ok(数值)`，失败用 `Err(说明文字)`。温度转长度不是零，也不是成功，是用户请求本身不成立。浮点输入可能是 NaN 或无穷大，进入计算前先检查 `value.is_finite()`，运算后也检查结果；不要用 `unwrap()` 把可预期的用户错误变成崩溃。

```powershell
cargo test --offline --test milestones step2_
```

测试还会检查 `f64::MAX` 千米转米的溢出。只有 `is_finite()` 检查了输入但没有检查结果，也会失败。先理解这两个检查的位置，不用学习浮点数二进制编码。

**小实验**：故意交换乘 1000 和除 1000，运行测试，再改回来。写下测试名称怎样帮助你找到错误分支。

## 第 3 步：把程序接到命令行

TODO(3) 中把字符串参数 `value` 解析成 `f64`，调用 `Unit::parse` 和你刚完成的 `convert`，最后返回一段格式化字符串。`value.parse::<f64>()` 可能失败；`map_err` 把解析错误变成人能读懂的提示；`?` 在成功时取出值，在失败时把错误提前交回上层。

使用 `format!` 而非 `println!`，因为这里需要返回 `String`。格式 `:.2` 保留两位小数；浮点表示和舍入规则不应被误当成十进制金额精度，因此账本项目会换一种数据表示。

```powershell
cargo test --offline --test milestones
cargo run --offline -- convert -40 c f
cargo run --offline -- convert 1 m f
cargo run --offline -- convert abc c f
```

最后两条应该明确报错，退出码非零。在 PowerShell 紧接着输入 `$LASTEXITCODE` 查看退出状态。报错是正确功能的一部分，不等于程序没有完成。

## 交付与你自己的变体

用 `cargo build --offline --release` 编译，Windows 程序在 `target/release/unitlab.exe`。运行它分别转换你的常见气温和步行距离。把五组输入、预期、实际写入 `LEARNING_LOG.md`；再尝试新增厘米，不改坏已有测试。新增单位是扩展任务，本轮基础验收不要求它。

**常见卡点**：E0308 往往是返回类型不一致；需要 f64 却得到 `()` 时检查最后一行分号。非穷尽 match 则表示遗漏分支，用错误返回补全不能处理的组合，而不是给所有未知请求一个零。

参考：Rust 官方书第 3 章的函数与控制流、第 6 章枚举、第 9 章错误处理。https://doc.rust-lang.org/book/

## 一次只检查当前阶段

本工程内可以运行 `cargo test --offline --test milestones step1_`。完成一步，再运行 step2_、step3_，最后执行全量 `cargo test --offline`。完整平台还提供权威检查：回到 RustAppLab 根目录执行 `cargo run --offline -p labquest -- studio check "my-unitlab" 1`，把最后的 1 改成 2、3 或 all。它会用自带的测试检查当前源码，不信任你编辑后的本地 tests。第 2 步会重新检查第 1 步，第 3 步会重查前两步。

检查会真实编译并运行你写的 Rust，只用于你信任的自编代码，不是安全沙箱。报告按检查次数生成新文件，包含当前源文件摘要；旧报告不能证明修改后的代码仍正确。遇到错误先看第一个 FAIL 和对应函数，再看 HINTS.md。

## 完成标准

功能测试通过只是其中一项。还需用自己的数据运行一次，给一个错误输入，保存真实产物或控制台结果，并在 LEARNING_LOG.md 解释一个关键 Rust 概念。然后为未提供的变体自行增加一个测试。不要把参考程序已通过等同于自己已学会。
