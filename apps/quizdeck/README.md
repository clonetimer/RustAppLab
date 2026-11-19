# QuizDeck · 用自己的题库做终端问答

**这次要做成什么：**从 TSV 文件读取题目，比较答案，计算分数并列出错题。成果是一份自己的题库、一份实测成绩和能重复运行的程序。

**开始条件：**完成 [UnitLab](../unitlab/README.md)，能说明函数参数、返回值和 `Result`。

本目录是已实现的参考软件；你的练习代码放在下面新建的 `my-quizdeck/` 中。

## 1. 看目标，创建自己的工程

在 **RustAppLab 根目录**（能看到根 `Cargo.toml`）运行：

```powershell
cargo run --offline -p quizdeck -- demo
cargo run --offline -p labquest -- studio start quizdeck my-quizdeck
cargo run --offline -p labquest -- learn lesson my-quizdeck 1
cargo run --offline -p labquest -- learn check my-quizdeck 1
```

参考演示应显示 `score=2/3` 和一道复习题。

学员首次检查因 TODO 未实现而 FAIL 是正常的。先看第一条失败；若是编译器或链接器找不到，则应先修好环境。

打开 `my-quizdeck/START.html` 或 `TASK_CARDS.html`；只编辑 **`my-quizdeck/src/lib.rs`**，每次一个 TODO。命令行外壳 `src/main.rs` 已提供，先不改。目录已存在就继续使用；新建另一份练习时换名称。

## 2. 三个检查点

| 步骤与函数 | 要完成的行为 | 本次概念 |
|---|---|---|
| TODO(1) · `is_correct` | 忽略首尾空白和 ASCII 大小写；完整答案相等才得分 | `&str`、`trim`、`bool`；相等不是包含 |
| TODO(2) · `parse_deck` | 每行读问题和答案两列，返回 `Vec<Card>`；坏行有错误 | `String` 所有权、结构体、`Vec`、逐行解析 |
| TODO(3) · `grade` | 先检查答案数量，再统计分数和错题问题文本 | 切片、`zip`、迭代器、输入边界 |

每步先写一组输入和预测，再修改、保存。第一步通过后，在 **平台根目录**继续：

```powershell
cargo run --offline -p labquest -- learn lesson my-quizdeck 2
cargo run --offline -p labquest -- learn check my-quizdeck 2
cargo run --offline -p labquest -- learn lesson my-quizdeck 3
cargo run --offline -p labquest -- learn check my-quizdeck 3
```

后一步会累计回归前面的步骤。提示用 `cargo run --offline -p labquest -- learn hint my-quizdeck 1`；进度用 `cargo run --offline -p labquest -- learn next my-quizdeck`。详细解释见 [quizdeck 讲义](../../course/studio/quizdeck.md)。

## 3. 用自己的输入完成一次作品

三步通过后，复制学员 `samples/deck.tsv` 为 `samples/my-deck.tsv`，先保留实际 Tab，只改成下面三题。再按自己的兴趣换题；代码块两列之间为实际制表符。

samples/my-deck.tsv：

```text
Rust 的构建工具	cargo
我想先做的工具	任务卡
2+3	5
```

samples/my-answers.txt（每行一个答案）：

```text
Cargo
任务卡
4
```

从 **平台根目录进入学员目录**执行：

```powershell
cd my-quizdeck
cargo run --offline -- check samples/my-deck.tsv samples/my-answers.txt
cargo run --offline -- play samples/my-deck.tsv
```

第一条应显示 `score=2/3`，复习清单包含 `2+3`；第二条会逐题等待输入并回车。再删除答案文件一行，应报数量错误，不能悄悄少算一题。

## 4. 交付与解释

在 **学员目录**运行 `cargo test --offline` 回归全部测试。保存个人题库、答案、一次有错题的成绩。把预测、错误和解释写入 `LEARNING_LOG.md`，再补一个自己的边界测试。

用自己的话回答：为什么 `Card` 存 `String`，比较函数却借用 `&str`？为什么 `zip` 前必须检查长度？

测试通过只说明已检查的行为符合合同；个人输入、真实产物和解释能帮助你判断学到了什么。

这里只忽略 ASCII 大小写，不做通用 Unicode 大小写折叠；不随机抽题、不保存历史成绩。

## 下一款：TextScope

接下来把逐题处理扩展到整篇文本，学习统计、词频排序和返回借用。在 **RustAppLab 根目录**运行（若仍在学员目录，先 `cd ..`）：

```powershell
cargo run --offline -p labquest -- studio start textscope my-textscope
cargo run --offline -p labquest -- learn lesson my-textscope 1
```

继续阅读 [TextScope README](../textscope/README.md)。
