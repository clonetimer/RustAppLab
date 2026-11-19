# 基础作品的练习数据

这里是 QuizDeck 和 TextScope 的**可选输入样例**，不含业务代码答案。先完成 [UnitLab → QuizDeck → TextScope 基础路线](../../course/FOUNDATION_PATH.md) 中对应作品的三个检查点，再用这些文件检查自己的程序。

刚完成 FirstCard 时，先按 [后续指南](../../course/AFTER_FIRSTCARD.md) 从 UnitLab 开始，不必直接跳到文件解析。

## 文件里有什么、应该观察到什么

| 文件 | 内容 | 预期结果 |
|---|---|---|
| `my-deck.tsv` | 三道题，问题与答案用真正的 Tab 分隔 | 能解析为三张卡片 |
| `my-answers.txt` | 三条答案，其中 cargo 故意写成 rustc | QuizDeck 得到 `score=2/3`，提示复习“Rust 构建工具” |
| `my-notes.txt` | 三行中英文笔记 | TextScope 查找 `Rust` 时匹配第 1、2 行 |

用编辑器修改时保留 Tab 和 UTF-8，不要把列分隔改成空格。第三行的 `rustc` 不包含大小写完全一致的 `Rust`。

只有学员包、没有平台 `apps/` 和 LabQuest 时，直接到下方“换成自己的程序”，使用包内 `Studio/quizdeck` 与 `Studio/textscope` 的学员工程。

## 可以先用参考程序验证预测

在 **RustAppLab 源码根目录**运行；这些命令只读取样例：

```powershell
cargo run --locked --offline -p quizdeck -- check examples/foundation/my-deck.tsv examples/foundation/my-answers.txt
cargo run --locked --offline -p textscope -- find examples/foundation/my-notes.txt Rust
```

核对上表结果。这验证参考程序与样例的对应关系，不算你实现了评分或查找函数。

## 换成自己的程序

把题库和答案复制到自己 QuizDeck 工程的 `samples/`，把笔记复制到自己 TextScope 工程的 `samples/`。若已有同名个人文件，请保留并换新文件名，随后相应修改命令路径。

三个检查点通过后，在 **自己的 QuizDeck 工程目录**运行：

```powershell
cargo run --offline -- check samples/my-deck.tsv samples/my-answers.txt
cargo run --offline -- play samples/my-deck.tsv
```

`play` 会等待逐题输入答案，按 Enter 提交；等待输入不是程序卡死。在 **自己的 TextScope 工程目录**运行：

```powershell
cargo run --offline -- stats samples/my-notes.txt
cargo run --offline -- top samples/my-notes.txt 5
cargo run --offline -- find samples/my-notes.txt Rust
```

这里不用 `-p labquest`；运行的是你自己目录下的 Cargo 工程。普通新建 Studio 工程首次可能还没有 Cargo.lock，因此不要求 `--locked`。

## 做一个自己的变化

QuizDeck：将一题改为你自己的复习内容，配套修改答案并先预测分数；再少写一条答案，程序应报错而不是忽略剩余题目。TextScope：追加一行含 `Rust` 的笔记，预测新行号，再比较搜索 `Rust` 与 `rust`。

你应能解释题库为何用 `String` 保存，而查找函数为何能返回借用的行；也应能区分字节数、字符数、行数和词元数。记录一次预测错误及修正原因，比只复制成功输出更有价值。

卡住时回 [QuizDeck 讲义](../../course/studio/quizdeck.md) 或 [TextScope 讲义](../../course/studio/textscope.md)，编辑自己工程 `src/lib.rs`，用平台 `learn lesson/check` 定位对应步骤。

## 下一步

保存自己的题库、笔记、命令和一个新增测试。完成 TextScope 后按 [基础第二段](../../course/FOUNDATION_NEXT_STEPS.md) 进入 LedgerLite，让处理过的数据持久保存；不要把样例运行成功理解为所有基础作品都已完成。
