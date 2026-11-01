# 基础作品连续练习：从一张任务卡到自己的复习工具

这里不是四个互不相关的小作业。你会把“输入→处理→输出”的同一思路逐次扩展：任务卡替换文本，UnitLab 处理数值，QuizDeck 管理一组自己的题目，TextScope 分析自己留下的文本。每次只增加一小段能力。

**适用基础**：会新建文件、保存、打开终端即可起步。不会解释 `Vec` 或借用没有关系，到对应作品再学。安装只在开始做一次；普通学员无需 Python、Playwright 或发布验收组件。

## 0. 先识别文件，避免改错项目

平台目录里的 `apps/unitlab` 是完整参考软件；你真正要编辑的是 **my-unitlab/src/lib.rs**。`main.rs` 已连接命令行，`lib.rs` 放业务函数，`tests/milestones.rs` 是公开功能合同。平台会用内置测试重新检查源代码，删除自己的测试不会骗过平台。

同一条 Cargo 命令在不同目录含义不同：平台里用 `-p labquest` 找教学控制台；自己的工程里不用 `-p labquest`，直接运行自己的软件。课程说“在学员目录执行”时，先确认那里有自己的 `Cargo.toml`。

## 1. FirstCard：理解变量不是文字替换规则

创建并完成第一张任务卡后，在它的目录执行：

```powershell
cargo run --offline -- "小林" "复习温度换算"
cargo run --offline -- "小陈" "写三道题"
```

两次输出中，名字和任务都应变化。只把“小林”写在输出里，不代表已经处理了参数。

**现在用自己的话回答**：`name` 与 `task` 在哪里获得值？`println!` 做了什么？这一步不要求解释所有权。把一次自己的输出保存为 `我的任务卡.txt`，然后再创建 UnitLab。已有任务卡不用重建。

## 2. UnitLab：从能显示到能计算

在平台目录创建新工程：

```powershell
cargo run --offline -p labquest -- studio start unitlab my-unitlab
cargo run --offline -p labquest -- learn lesson my-unitlab
```

第一次运行检查会失败，因为核心函数尚未完成。**这不是安装失败**。先实现 `celsius_to_fahrenheit`，之后才做 `convert` 和 `describe`。每一项都可单独检查；第一步只调用函数，还不能期待整个命令行软件已经工作。

第一步卡在类型错误时，检查函数是否要求 `f64`，你却用了只负责显示的 `println!`；或把最后一个表达式写成了带分号的语句。不要把“返回”与“打印”混在一起。

第二步要回答：同样是数字，为什么摄氏度不能转换成米？用枚举标记单位，用 `Result` 表示不兼容，不用一个可能是合法结果的 `0` 假装错误。

三步都通过后，在自己的 `my-unitlab` 目录运行：

```powershell
cargo run --offline -- convert -10 c f
cargo run --offline -- convert 1500 m km
cargo run --offline -- convert 25 c m
```

前两条分别应为 `-10.00 c = 14.00 f`、`1500.00 m = 1.50 km`；第三条应拒绝并返回非零退出码。为自己选一个新数值，先算预期结果，再运行验证。留下两次成功和一次失败的记录。

**过渡问题**：如果输入不再是一个数，而是一份题库，原先的函数、错误处理还能不能复用思路？下一款只改变数据类型，不需要忘掉以前学过的结构。

## 3. QuizDeck：把自己想复习的知识变成软件输入

在平台目录创建 `my-quizdeck`。第 1 步只比较两个答案；第 2 步才把文件文本变成 `Vec<Card>`；第 3 步才把题目和答案一起评分。

`is_correct` 接收 `&str`，因为只要暂时读取答案。`Card` 里保存 `String`，因为源文本读完后题目还需要继续存在。这是借用和拥有数据之间的实际区别，而不是需要死记的两种字符串名字。

题库用 TSV：问题和答案中间必须是**真正的 Tab**。不确定如何输入时，复制已有 `samples/deck.tsv` 为 `samples/my-deck.tsv`，只改内容、保留列分隔。配套示例在本包 `examples/foundation/my-deck.tsv`，它只是数据，不含 Rust 答案。

把题库和答案样例复制到自己工程的 `samples`，三步完成后运行：

```powershell
cargo run --offline -- check samples/my-deck.tsv samples/my-answers.txt
cargo run --offline -- play samples/my-deck.tsv
```

提供的数据有三题，答案故意错一题；第一条应显示 `score=2/3`，并提示复习“Rust 构建工具”。第二条会等待你在终端逐题输入，这是交互程序的正常行为，不是卡死。按提示输入答案后按 Enter。

然后把其中一题改成你自己的知识。不要只运行内置 demo；你的作品产物应包括自己的题库、答案和复习清单。再试删除一份答案，解释为什么评分应失败，而不是静默忽略剩下的问题。

**过渡问题**：复习清单已经是一份文本；能否用下一款软件找出你最常提到的词，或者搜索所有含 Rust 的行？

## 4. TextScope：分析自己的笔记，不只分析演示字符串

创建 `my-textscope`。第一步统计四种数量，第二步聚合词频，第三步返回原文中匹配的行。

从一个已知文本开始预测：`Rust 学习` 后面有一个换行时，UTF-8 字节为 12、Unicode scalar 为 8、行数为 1、词元为 2。四个数字不一样，原因在于统计规则不同。`chars` 也不是完整的视觉字形划分；连续中文在当前词元规则下不被自动切成中文词语。本项目没有冒充自然语言分词器。

第二步先计数，再排序，同频词按字典序排列。第三步使用 `&str` 借用原文的行，不能返回已释放局部 `String` 的引用，也不要通过内存泄漏绕开问题。

把自己的复习笔记放在 `samples/my-notes.txt`，三步完成后在该工程运行：

```powershell
cargo run --offline -- stats samples/my-notes.txt
cargo run --offline -- top samples/my-notes.txt 5
cargo run --offline -- find samples/my-notes.txt Rust
```

补做两个实验：把 `Rust` 改为 `rust` 后搜索结果是否变化；追加一行含 Rust 的内容，结果的行号是否正确。记录你预测错误的一次原因，比只保存成功截图更有用。

## 5. 怎样形成一份真正的作品

提交自己的源码、实际使用的数据、运行命令、一次成功输出、一次异常输入的处理、一个新增测试和简短使用说明。原始样例可以作为起点，但要明确哪些是自己修改的。

建议在 `LEARNING_LOG.md` 中填写：

| 项目 | 你要写什么 |
|---|---|
| 我解决的问题 | 哪种换算、哪门知识的复习、哪份文本分析 |
| 我的输入与输出 | 使用的文件名、命令及真实结果 |
| 我改的函数 | 函数名，以及它接收和返回什么 |
| 我遇到的错误 | 第一条 error 或 FAIL，怎样定位和修复 |
| 我使用的帮助 | 提示第几层、是否看过参考源码、是否请人协助 |
| 我的新测试 | 与公开样例不同的一个边界或正常输入 |

这些记录不自动等于教学合格。教师可以让学员解释一次改变输入后的行为，观察是否能迁移思路；软件测试只证明已测试的功能行为。

后续先进入 LedgerLite，把数据保存成持久记录；再考虑 trait、线程和网络，不需要因新版本出现就跳过当前基础。

## 术语与依据

`str::len` 返回 UTF-8 字节数，`chars` 遍历 Unicode scalar，`eq_ignore_ascii_case` 的大小写忽略范围是 ASCII；说明依据 Rust 官方 [字符串 API](https://doc.rust-lang.org/std/primitive.str.html)。类型错误方向可查官方 [E0308](https://doc.rust-lang.org/stable/error_codes/E0308.html)。本课示例结果均可用随包源码和样例重新运行核对。


## 6. 下一段：可保存、可打开、可交互的作品

完成 TextScope 后，按 `course/FOUNDATION_NEXT_STEPS.md` 继续 LedgerLite → BadgeForge → TermGarden：分别交付真实账本、SVG 作品卡和可重放任务记录。支持和前面一样的三级任务卡，`learn lesson` 能定位到具体函数。

做过多个工程后，用 `learn overview <练习父目录>` 查看历史记录是否过期；它不替代重新运行测试，也不认证学习成果。详见 `course/PROGRESS_OVERVIEW.md`。
