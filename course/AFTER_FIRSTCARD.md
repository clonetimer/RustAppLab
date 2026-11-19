# FirstCard 做完了：我学到了什么，下一步做什么？

**下一课是 UnitLab（单位换算器）。** 你会从“把输入显示出来”走到“接收一个数、计算结果、处理错误”。先完成下面的小复盘，再按本页创建自己的换算器；不用重新安装 Rust，也不用重做 FirstCard。

本页的教学控制台命令使用**完整源码包或仓库**。如果只有 learner-kit，直接打开已有的 `FirstCard/README.md` 或 `Studio/unitlab/README.md`，使用里面的本地测试步骤；需要 `-p labquest` 的平台检查时，再在完整平台中传入该工程的实际路径。

本页的“平台根目录”指解压后的 **RustAppLab** 文件夹，里面同时有 `apps/`、`course/`、`Cargo.toml`、`CONTINUE_WINDOWS.cmd`。“学员目录”是你自己的 `my-first-card/` 或 `my-unitlab/`，也有 `Cargo.toml`。**只看到 Cargo.toml，还不能判断自己在哪一个目录。**

## 1. FirstCard 到底练到了什么？

| 你做过的事 | 对应能力 | 用什么证明自己理解了 |
|---|---|---|
| 打开、修改并保存 `src/main.rs` | 知道 Rust 源码在 `.rs` 文件里，保存后需要重新运行 | 指出改变输出的那一行；重新运行能看到改动 |
| 使用 `name` 和 `task` | 知道变量保存输入值，输出要随输入变化 | 不改源码，换两组姓名和任务，先预测再观察 |
| 使用 `println!` 和 `{name}` | 用格式化输出把变量插进文字 | 解释为什么把“小林”写死不能处理其他姓名 |
| 执行 `cargo run -- ...` | 知道 Cargo 会构建并运行程序，`--` 后面是给程序的参数 | 说明换参数与改源码有什么不同 |
| 运行 `beginner check` 并看“期待 / 实际” | 初步区分编译错误与行为不符合要求 | 描述自己一次失败的原因和修改方法 |

这节课已经提供了读取命令行参数的代码。你亲手完成的是**使用两个变量输出任务卡**，还没有独立实现参数解析，也还没有练完函数、分支、集合、所有权等内容。可以把本节成果说成：“我能修改并运行一个小程序，让输出随输入变化。”

### 离开 FirstCard 前，独立做一次

在 **平台根目录**运行下面三条；如果工程另有名字或位置，把引号里的路径换成你的实际路径：

```powershell
cargo run --offline --manifest-path "my-first-card/Cargo.toml" -- "小雨" "整理学习笔记"
cargo run --offline --manifest-path "my-first-card/Cargo.toml" -- "Alex Chen" "复习函数"
cargo run --locked --offline -p labquest -- beginner check "my-first-card"
```

前两条的输出应分别包含各自的姓名和任务，第三条应显示四项通过。用自己的姓名和任务再试一次，把命令和结果记在自己的笔记中。若只能照抄，还不能解释变量，请先回到工程中的 `跟着做.md`，不需要删除或重建工程。

**不看答案回答三句话：** 我改的是哪份文件？参数变化时哪一部分输出会变化？编译成功为什么还可能检查失败？能回答并能重新做出结果，就可以开始下一课。

## 2. 现在就开始 UnitLab：两种入口选一种

### Windows：用向导

1. 回到 **RustAppLab 文件夹**，双击 `CONTINUE_WINDOWS.cmd`。
2. 在作品菜单按 **Enter**，选择默认的 **UnitLab**。
3. 浏览器打开讲义，编辑器打开 `my-unitlab/src/lib.rs`。先只完成 `TODO(1)` 的 `celsius_to_fahrenheit`。
4. 按 Ctrl+S 保存，回到向导按 Enter 检查第 1 步。通过后读第 2 步，再编辑对应函数。

若已经有 `my-unitlab`，向导会继续使用它。下次也可双击 `LEARN_NEXT_WINDOWS.cmd`，输入这个工程的路径，再用 **L 看任务、E 编辑、C 检查**。退出后作品仍在原目录。

### Windows / Linux：用完整命令

下面所有命令都在 **平台根目录**运行。只有第一次创建才执行 `studio start`，随后先读任务卡，再打开自己的源文件：

```powershell
cargo run --locked --offline -p labquest -- studio start unitlab "my-unitlab"
cargo run --locked --offline -p labquest -- learn lesson "my-unitlab" 1
```

用编辑器打开 **`my-unitlab/src/lib.rs`**，找到 `celsius_to_fahrenheit` 的 `TODO(1)`。不要改 `apps/unitlab/src/lib.rs`，那是已经完成的参考实现。学习这一函数的解释在 [UnitLab 第 1 步讲义](studio/unitlab.md)。

保存后执行：

```powershell
cargo run --locked --offline -p labquest -- learn check "my-unitlab" 1
```

第一次检查未完成的 TODO 会失败。若失败提示来自 `celsius_to_fahrenheit`，说明已经进入了练习；如果提示找不到 `cargo`、链接器或文件路径，先修复环境或目录问题。

**已有工程不再执行 start。** 直接用 `learn lesson`、`learn next`、`learn check`；路径含中文或空格时保留双引号。`learn next` 只给建议，不会自动运行测试。

## 3. UnitLab 每一步要学什么？

| 顺序 | 只改这个函数 | 这一步的用途与概念 | 检查与理解目标 |
|---|---|---|---|
| 1 | `celsius_to_fahrenheit` | 把摄氏度变成华氏度；函数输入、`f64`、返回值 | 运行 `learn check "my-unitlab" 1`；解释“返回数值”和“打印文字”的区别 |
| 2 | `convert` | 在温度和长度单位之间选对转换；`enum`、`match`、`Result` | 检查第 2 步；解释为什么摄氏度转米应该失败，而不是返回 0 |
| 3 | `describe` | 把文本输入解析为数值，再格式化结果；解析、`?`、`String` | 检查第 3 步；解释哪一步可能出错，以及错误怎样交给调用者 |

表中的短命令是阅读提示，**终端中请使用下面完整命令**。第 1 步通过后，依次执行“读下一步 → 修改对应 TODO → 保存 → 检查”：

```powershell
cargo run --locked --offline -p labquest -- learn lesson "my-unitlab" 2
cargo run --locked --offline -p labquest -- learn check "my-unitlab" 2
cargo run --locked --offline -p labquest -- learn lesson "my-unitlab" 3
cargo run --locked --offline -p labquest -- learn check "my-unitlab" 3
```

不要一次复制这四条跳过编辑。第 2 步会重新验证第 1 步，第 3 步会重新验证前两步。**只有三步完成后，才要求整个命令行换算器可用。**

卡住时，在平台根目录先查看当前建议，再按当前步骤看第一层提示，例如第 2 步：

```powershell
cargo run --locked --offline -p labquest -- learn next "my-unitlab"
cargo run --locked --offline -p labquest -- learn hint "my-unitlab" 1 2
```

最后两个数字依次是“提示级别、步骤”。先尝试第一层提示，仍不会再把提示级别换成 2 或 3。看过提示不影响继续学习，把用了什么帮助写入学习记录即可。

## 4. 三步通过后，怎么判断自己会了？

仍在 **平台根目录**，先用纸笔预测，再运行自己的程序：

```powershell
cargo run --offline --manifest-path "my-unitlab/Cargo.toml" -- convert -10 c f
cargo run --offline --manifest-path "my-unitlab/Cargo.toml" -- convert 1500 m km
cargo run --offline --manifest-path "my-unitlab/Cargo.toml" -- convert 25 c m
```

| 输入 | 预期结果 | 你要能解释的内容 |
|---|---|---|
| `-10 c f` | `-10.00 c = 14.00 f` | 参数如何进入函数、公式如何计算 |
| `1500 m km` | `1500.00 m = 1.50 km` | 为什么这一分支除以 1000 |
| `25 c m` | 明确报错，退出码非零 | 这次报错为什么是正确行为 |

再做一个没有给出答案的实验：选一个自己常见的气温，先算出华氏度，再运行；然后把数字换成 `abc`，观察解析失败。能独立完成并解释，比只看到三步通过多了一份理解证据。

在 **`my-unitlab/LEARNING_LOG.md`** 写下：

- 我做出的软件：能完成哪些换算，不能完成什么。
- 我亲手实现的三个函数，各自的输入和返回值。
- 我自己的正常输入、错误输入、预测和实际结果。
- 我用过哪层提示，以及仍不能独立解释的地方。

如果已经能独立预测和运行，再参考 `my-unitlab/tests/milestones.rs` 的写法，在新文件 **`my-unitlab/tests/my_cases.rs`** 中新增一个自己选取数值的测试；不要改掉已有断言。保持在平台根目录，先运行自己的本地测试，再复验平台的固定合同：

```powershell
cargo test --offline --manifest-path "my-unitlab/Cargo.toml"
cargo run --locked --offline -p labquest -- learn check "my-unitlab" 3
```

第一条会运行你新增的测试；第二条使用平台自带的验收测试，不会代你运行 `my_cases.rs`。两者都通过后，再记录这次扩展的输入与结果。

## 5. UnitLab 完成后，明确去哪里？

下一款是 **QuizDeck（自己的题库）**。UnitLab 处理单个数值，QuizDeck 开始处理一组文本：比较答案 → 读取题目 → 评分。前一课的函数、返回值和错误处理会继续用到。

在 **平台根目录**执行；如果已有该工程，跳过第一条：

```powershell
cargo run --locked --offline -p labquest -- studio start quizdeck "my-quizdeck"
cargo run --locked --offline -p labquest -- learn lesson "my-quizdeck" 1
```

接着打开 `my-quizdeck/src/lib.rs`，只做 `is_correct` 的 TODO(1)，保存后：

```powershell
cargo run --locked --offline -p labquest -- learn check "my-quizdeck" 1
```

完成 QuizDeck 后去 **TextScope**，分析自己的题库或笔记。近三课的连贯练习见 [基础作品路线](FOUNDATION_PATH.md)；想了解远期方向再看 [课程地图](CURRICULUM.md)。Book 专题是遇到相关概念时的补充阅读，FileQuest 等原有章节是拓展练习，不是进入 UnitLab 前必须清完的任务。

每一课都按同一节奏：**知道要做什么 → 改一个函数 → 看真实结果 → 用自己的输入复现 → 解释学到的能力 → 进入下一课。** 自动检查负责验证程序行为，你的预测、解释和新输入实验负责帮助你判断理解程度。
