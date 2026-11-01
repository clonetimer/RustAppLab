# TextScope · 文本观察镜

**本作品的中心思想**：用一款可运行的小软件，把三个连续能力连接起来，而不是完成三道无关语法题。

## 开始前只确认这件事

你当前应位于这个学员工程的目录，能够看到 `Cargo.toml`。终端中的 `cargo` 命令默认在这里运行。`src/main.rs` 是已经接好的命令行外壳；本轮只修改 `src/lib.rs` 的 TODO(1)、TODO(2)、TODO(3)。`tests/milestones.rs` 是公开的验收合同，不能通过删除测试“通关”。

在完整 RustAppLab 根目录创建本工程的命令是 `cargo run --offline -p labquest -- studio start textscope my-textscope`，然后进入 `my-textscope`。本讲义也随学员包提供，包内已有工程就不必再次生成。Windows 可以用 `CONTINUE_WINDOWS.cmd` 选择作品。

三个里程碑依次是：**分清字节、字符、行和词元 → 统计词频并建立确定性排序 → 返回借用原文的搜索结果**。未实现的后两步不会阻止你运行第一步测试；语法错误仍会阻止整个库编译。

## 先看你要做出的软件

你要阅读一篇文稿，快速知道行数、常用词，找到包含某个关键词的原文。完成后在本工程目录运行：

```powershell
cargo run --offline -- stats samples/notes.txt
cargo run --offline -- top samples/notes.txt 5
cargo run --offline -- find samples/notes.txt Rust
```

这是统计与检索工具，不做中文自然语言分词。我们把连续字母或数字视为一个词元，标点和空白用于分隔；连续中文“学习编程”算一个词元。搜索区分大小写，词频不区分大小写。这两项是明确不同的功能。

## 第 1 步：同一文本有不同的计量方式

打开 `src/lib.rs`，修改 `stats` 的 TODO(1)。`text.len()` 是 UTF-8 字节数，`text.chars().count()` 是 Unicode scalar 数，不一定等于用户眼中的一个显示字形。行数按 `lines()` 计算，结尾换行不额外产生一条空行；词元使用预置的 `tokens()`。

`Stats` 已定义四个字段。构造一个结构体并填入四项结果，不要用四个 println! 替代返回值，也不要把字符数当作字节数。`Stats::default()` 目前全是零，只是能编译的占位。

```powershell
cargo test --offline --test milestones step1_
```

对文本 `Rust 学习\n`，应为 12 字节、8 个 scalar、1 行、2 词元。这个示例能揭示中文处理和纯英文处理的区别。

## 第 2 步：词频不只是一个计数器

TODO(2) 使用 `HashMap<String, usize>` 保存“标准化词 → 次数”。每个词先转换成小写，`entry(...).or_insert(0)` 得到当前计数的可变引用，再递增。完成统计后把 Map 转成 Vec 并排序：次数从大到小；次数相同时词按字典序。最后截断到 `limit`。

为什么要排序？HashMap 的遍历顺序不应被当作用户看到的稳定顺序。相同输入若每次 Top 结果跳来跳去，既难阅读也难测试。`into_iter()` 在这里消耗已经不再需要的 Map，条目拥有的 String 交给结果 Vec，不必再 clone。

```powershell
cargo test --offline --test milestones step2_
```

`Rust rust! apple Banana banana` 的前两名是 banana:2 和 rust:2。limit 为零时返回空结果；超过词种数时返回全部，不添加虚假空项。

**练习**：先取消第二排序键观察失败，再补回。讲清楚“频次相同”的边界为什么不能留给随机遍历顺序。

## 第 3 步：返回原文的借用，而不是复制全文

现在修改 `search<'a>(text: &'a str, query: &str) -> Vec<(usize, &'a str)>`。`'a` 说明返回行的引用来自 text；它不会让 text 自动活得更久，也不是指定运行时间。query 只在查找时使用，返回结果不引用 query，因此不需要把它和 text 绑在同一个显式生命周期参数上。

使用 `lines().enumerate()` 获得从 0 开始的位置，筛选含关键词的行，再把显示行号加 1。空关键词在库函数返回空列表；CLI 会对空关键词报错。返回 `&str` 即可，不要创建局部 String 再返回其引用，也不要用 `Box::leak` 绕过生命周期约束。

```powershell
cargo test --offline --test milestones step3_
```

检查会确认结果确实指向原文，而不是每次新分配的字符串。试着解释：如果先把 text 所属的 String 丢弃，再打印结果，为什么编译器应该阻止这个程序？

## 交付与故障实验

用自己的小篇 UTF-8 文本生成统计和词频，在 PowerShell 可以用 `Out-File -Encoding utf8` 保存到一个全新文件名。当前 CLI 去掉开头 BOM 后统计内容，库函数则统计你传入的文本原样。不要把结果误称为精确的中文词数。

换成不存在文件时应有明确错误；输入二进制文件不能伪装成空文本。扩展任务：增加大小写不敏感搜索，并写清楚返回的行仍来自原始输入，不能返回临时 lowercase 字符串里的引用。

参考：Rust 官方生命周期讲解 https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html ，迭代器 https://doc.rust-lang.org/book/ch13-02-iterators.html 。

## 一次只检查当前阶段

本工程内可以运行 `cargo test --offline --test milestones step1_`。完成一步，再运行 step2_、step3_，最后执行全量 `cargo test --offline`。完整平台还提供权威检查：回到 RustAppLab 根目录执行 `cargo run --offline -p labquest -- studio check "my-textscope" 1`，把最后的 1 改成 2、3 或 all。它会用自带的测试检查当前源码，不信任你编辑后的本地 tests。第 2 步会重新检查第 1 步，第 3 步会重查前两步。

检查会真实编译并运行你写的 Rust，只用于你信任的自编代码，不是安全沙箱。报告按检查次数生成新文件，包含当前源文件摘要；旧报告不能证明修改后的代码仍正确。遇到错误先看第一个 FAIL 和对应函数，再看 HINTS.md。

## 完成标准

功能测试通过只是其中一项。还需用自己的数据运行一次，给一个错误输入，保存真实产物或控制台结果，并在 LEARNING_LOG.md 解释一个关键 Rust 概念。然后为未提供的变体自行增加一个测试。不要把参考程序已通过等同于自己已学会。
