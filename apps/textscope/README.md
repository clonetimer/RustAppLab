# TextScope · 让笔记告诉你它包含什么

**这次要做成什么：**统计文本的字节、字符、行和词元，列出高频词，按关键词返回原文行。成果是一份自己的文本及可解释的分析结果。

**开始条件：**完成 [QuizDeck](../quizdeck/README.md)，已接触 `String`、`&str`、结构体和迭代器。

本目录是已实现的参考软件；你的练习代码放在下面新建的 `my-textscope/` 中。

## 1. 看目标，创建自己的工程

在 **RustAppLab 根目录**（能看到根 `Cargo.toml`）运行：

```powershell
cargo run --offline -p textscope -- demo
cargo run --offline -p labquest -- studio start textscope my-textscope
cargo run --offline -p labquest -- learn lesson my-textscope 1
cargo run --offline -p labquest -- learn check my-textscope 1
```

演示文本 `Rust rust 学习` 加末尾换行，应得到 `bytes=17 chars=13 lines=1 words=3`。

学员首次检查因 TODO 未实现而 FAIL 是正常的。先看第一条失败；若是编译器或链接器找不到，则应先修好环境。

打开 `my-textscope/START.html` 或 `TASK_CARDS.html`；只编辑 **`my-textscope/src/lib.rs`**，每次一个 TODO。命令行外壳 `src/main.rs` 已提供，先不改。目录已存在就继续使用；新建另一份练习时换名称。

## 2. 三个检查点

| 步骤与函数 | 要完成的行为 | 本次概念 |
|---|---|---|
| TODO(1) · `stats` | 分别统计字节、Unicode scalar、行和词元 | 同一文本的四种计量；`len/chars/lines` |
| TODO(2) · `top_words` | 词转小写后统计；次数降序，同频按字典序 | `HashMap`、拥有型键、确定性排序 |
| TODO(3) · `search` | 返回从 1 开始的行号和原文行的引用 | 生命周期、切片、借用；避免返回临时字符串 |

每步先写一组输入和预测，再修改、保存。第一步通过后，在 **平台根目录**继续：

```powershell
cargo run --offline -p labquest -- learn lesson my-textscope 2
cargo run --offline -p labquest -- learn check my-textscope 2
cargo run --offline -p labquest -- learn lesson my-textscope 3
cargo run --offline -p labquest -- learn check my-textscope 3
```

后一步会累计回归前面的步骤。提示用 `cargo run --offline -p labquest -- learn hint my-textscope 1`；进度用 `cargo run --offline -p labquest -- learn next my-textscope`。详细解释见 [textscope 讲义](../../course/studio/textscope.md)。

## 3. 用自己的输入完成一次作品

三步通过后，用编辑器保存 UTF-8 文件 `samples/my-notes.txt`，下面两行各保留末尾换行。

samples/my-notes.txt：

```text
Rust 学习
Rust helps
```

从 **平台根目录进入学员目录**执行：

```powershell
cd my-textscope
cargo run --offline -- stats samples/my-notes.txt
cargo run --offline -- top samples/my-notes.txt 3
cargo run --offline -- find samples/my-notes.txt Rust
cargo run --offline -- find samples/my-notes.txt rust
```

若使用 LF 换行，统计应为 `bytes=23 chars=19 lines=2 words=4`；CRLF 会增加字节和 scalar 数，但仍是两行。词频 `rust` 为 2；`find Rust` 返回两行，`find rust` 没有匹配。再写自己的笔记，先预测某词出现几次。

## 4. 交付与解释

在 **学员目录**运行 `cargo test --offline` 回归全部测试。保存个人文本、统计、词频与搜索结果。把预测、错误和解释写入 `LEARNING_LOG.md`，再补一个自己的边界测试。

用自己的话回答：为什么 `len()` 与 `chars().count()` 不同？为什么 HashMap 遍历顺序不能直接用作排行榜？返回的 `&str` 为什么不能比原文活得更久？

测试通过只说明已检查的行为符合合同；个人输入、真实产物和解释能帮助你判断学到了什么。

字符指 Unicode scalar，不保证等于显示字形；连续中文串按既定规则成为词元，不是中文分词。CLI 去掉开头 BOM 后统计；`top` 数量限 1–100，会将 UTF-8 文件读入内存。

## 下一款：LedgerLite

接下来把文本解析用于会保存到磁盘的账目，学习整数金额与溢出处理。在 **RustAppLab 根目录**运行（若仍在学员目录，先 `cd ..`）：

```powershell
cargo run --offline -p labquest -- studio start ledgerlite my-ledgerlite
cargo run --offline -p labquest -- learn lesson my-ledgerlite 1
```

继续阅读 [LedgerLite README](../ledgerlite/README.md)。
