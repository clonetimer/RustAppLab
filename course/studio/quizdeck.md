# QuizDeck · 终端知识闯关

**本作品的中心思想**：用一款可运行的小软件，把三个连续能力连接起来，而不是完成三道无关语法题。

## 开始前只确认这件事

你当前应位于这个学员工程的目录，能够看到 `Cargo.toml`。终端中的 `cargo` 命令默认在这里运行。`src/main.rs` 是已经接好的命令行外壳；本轮只修改 `src/lib.rs` 的 TODO(1)、TODO(2)、TODO(3)。`tests/milestones.rs` 是公开的验收合同，不能通过删除测试“通关”。

在完整 RustAppLab 根目录创建本工程的命令是 `cargo run --offline -p labquest -- studio start quizdeck my-quizdeck`，然后进入 `my-quizdeck`。本讲义也随学员包提供，包内已有工程就不必再次生成。Windows 可以用 `CONTINUE_WINDOWS.cmd` 选择作品。

三个里程碑依次是：**比较答案并处理空白 → 把文本解析为拥有数据的结构体 → 用切片、迭代器生成复习清单**。未实现的后两步不会阻止你运行第一步测试；语法错误仍会阻止整个库编译。

## 先看你要做出的软件

把自己的 Rust 术语、外语单词或常识写成题库，让程序逐题提问，最后生成需要复习的题目。完成后在本工程目录运行：

```powershell
cargo run --offline -- play samples/deck.tsv
```

终端显示一题就输入一个答案并回车。样例有 3 题；全部答对时显示 `score=3/3`。也可先使用 `cargo run --offline -- demo` 看内置模拟答案：完成后应为 `score=2/3`，不是把屏幕输出写死。

前置要求：完成 UnitLab，能够看懂函数的输入和返回值。这里第一次引入“一个值里有多个字段”。

## 第 1 步：先让答案比较公平

打开 `src/lib.rs`，只做 `is_correct` 中的 TODO(1)。目标：前后空白不影响结果，ASCII 英文字母大小写不影响结果，中文必须内容相同。`trim()` 返回借用的字符串切片，不需要复制整个字符串。`eq_ignore_ascii_case` 只处理 ASCII 大小写，不是通用 Unicode 大小写折叠。

```powershell
cargo test --offline --test milestones step1_
```

`cargo` 与 ` Cargo ` 应相等，`cargo` 与 `cargo build` 不相等。不能用 `contains()` 代替完整答案相等，否则只要答案里含有正确单词就能得分。

**预测再试**：中文答案“北京”和“ 北京 ”是什么结果？é 和 É 呢？解释为什么本作品没有声称支持全部语言的大小写归一化。

## 第 2 步：让题库真的来自文件

`Card` 是结构体，包含 `question: String` 和 `answer: String`；`Vec<Card>` 是多张卡片。TODO(2) 接收一整个文本，按行处理，每行必须恰好用一个 TAB 分出两个非空字段。注意 TAB 是制表符，不是连续空格，Rust 中写作 `'\t'`。

先使用已提供的 `samples/deck.tsv`，不要一开始用记事本手敲制表符。空白行忽略；第一行可能有 UTF-8 BOM；Windows 文件常用 CRLF 换行，`lines()` 能处理这种行结尾。遇到坏行应返回含行号的 `Err`，不能悄悄略过错误题目；完全空的题库也返回错误。

在解析循环里，`line`、`fields[0]` 都借用输入文本。可是返回的卡片可能比输入文本存在更久，因此字段要用 `.to_string()` 取得自己的字符串。这是有实际用途的所有权转移，不是为了消除编译错误随便 clone。

```powershell
cargo test --offline --test milestones step2_
```

测试会让原输入 `String` 在内部作用域结束，再检查返回的卡片依旧可读。这个场景说明为什么 `Card` 的字段不是指向短命局部变量的引用。

## 第 3 步：给成绩一个结构

TODO(3) 接收 `&[Card]` 和 `&[String]`，它们是只读借用的切片。先检查数量相同，再逐题比较。`Score` 保存总题数、正确数和错题的问题文本。不要在函数里读取终端；交互已经在 `main.rs` 中提供，业务函数只做计算，才能自动测试。

可以先用一个普通 `for` 循环；理解后再改成 `iter().zip()`。只要合同正确，学习阶段不要求用最短的一行迭代器。错误答案是正常成绩，答案数量不匹配才是 `Err`。

```powershell
cargo test --offline --test milestones step3_
cargo run --offline -- check samples/deck.tsv samples/answers.txt
cargo run --offline -- play samples/deck.tsv
```

第二条应为 `score=3/3`。复制题库为自己的文件，至少写 5 题，故意答错一题，确认复习清单真的包含那道题。不要修改官方样例来适配测试结果。

## 从函数过渡到终端交互

完成三个步骤后再读 `src/main.rs` 的 play 分支：`read_line(&mut answer)` 要把终端输入写进一个可变 String，所以使用可变借用；`flush()` 让 `>` 提示及时出现；读到 EOF 应退出并报告未完成，而不是无限循环等输入。它们是输入输出层的责任，和判分函数分开。

**故障实验**：给 TSV 某行追加第三列，确认提示具体行号；再恢复数据。终端模式遇到输入中断不会伪造满分。

## 交付与变体

交付自己的题库、Release 程序、一次实测成绩和错题说明。扩展任务是把错题保存为新的 TSV；先决定是否允许覆盖，再编码。当前基本版不保存历史成绩，也不随机抽题，不要据此宣称掌握概率抽样或数据库。

参考：Rust 官方书结构体、集合、所有权与借用。https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html

## 一次只检查当前阶段

本工程内可以运行 `cargo test --offline --test milestones step1_`。完成一步，再运行 step2_、step3_，最后执行全量 `cargo test --offline`。完整平台还提供权威检查：回到 RustAppLab 根目录执行 `cargo run --offline -p labquest -- studio check "my-quizdeck" 1`，把最后的 1 改成 2、3 或 all。它会用自带的测试检查当前源码，不信任你编辑后的本地 tests。第 2 步会重新检查第 1 步，第 3 步会重查前两步。

检查会真实编译并运行你写的 Rust，只用于你信任的自编代码，不是安全沙箱。报告按检查次数生成新文件，包含当前源文件摘要；旧报告不能证明修改后的代码仍正确。遇到错误先看第一个 FAIL 和对应函数，再看 HINTS.md。

## 完成标准

功能测试通过只是其中一项。还需用自己的数据运行一次，给一个错误输入，保存真实产物或控制台结果，并在 LEARNING_LOG.md 解释一个关键 Rust 概念。然后为未提供的变体自行增加一个测试。不要把参考程序已通过等同于自己已学会。
