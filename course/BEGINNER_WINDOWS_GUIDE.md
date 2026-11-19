# 第 0 阶段：Windows 学员从哪里开始（约 30–45 分钟）

**已经完成这节课？打开 [FirstCard 做完之后](AFTER_FIRSTCARD.md)，从 UnitLab 继续。** 已有任务卡无需重建。下面是第一次学习时的逐步说明。

**适用对象**：不知道终端、项目目录和 `cargo` 是什么的初学者。今天只做一款非常小的应用，不要求完成 32 关。

## 你会得到什么

让学员从「还没使用过 Rust」到能够自己修改 `println!`、运行不同输入、生成个人 TXT 任务卡。

| 阶段 | 学员做什么 | 学会什么 | 可以观察的证据 |
|---|---|---|---|
| 0–10 分钟 | 解压、安装 Rust、双击 `START_WINDOWS.cmd` | Rust 是工具链，程序要被编译 | `cargo --version` 显示版本；向导能启动 |
| 10–15 分钟 | 打开 `my-first-card/src/main.rs` | 代码保存在 `.rs`，命令运行的是程序 | 找到唯一的 TODO 输出行 |
| 15–25 分钟 | 修改一行 `println!` 并保存 | 变量、格式化输出、文件保存 | 控制台输出自己的名字和任务 |
| 25–30 分钟 | 执行 `beginner check` | 编译通过不等于功能正确 | 4/4 检查通过并生成 `FIRST_WIN.md` |
| 30–45 分钟 | 换两组输入，保存任务卡 TXT | 输入与输出、可交付文件 | 能打开含自己任务的 TXT 文件 |

这些时间仅供授课组织参考，**不是对学员完成速度的承诺**。

## 给学员的逐句指令

1. “先只打开根目录的 `START_HERE.html`，其它文件暂时不用看。”
2. “Windows 文件资源管理器中双击 `START_WINDOWS.cmd`。如果没有安装 Rust，向导会告诉你安装地址；安装完成后重新双击。”
3. “当记事本打开后，找到 `println!` 那一行，**只改这一行**。它用 `name` 和 `task` 表示两个不同的文本。”
4. “保存后回到黑色窗口按 Enter。若失败，先阅读‘期待’和‘实际’，不要删除工程、不要重新安装。”
5. “四项检查全部通过后，打开 `my-first-card/FIRST_WIN.md`，再尝试输出自己的真实任务卡。”

如果 Windows 不允许双击脚本，先在 RustAppLab 文件夹的地址栏输入 `powershell` 回车，然后依次运行：

```powershell
cargo --version
cargo run --offline -p labquest -- beginner start my-first-card
# 先运行未完成程序：
cargo run --offline --manifest-path my-first-card/Cargo.toml -- "小林" "写 Rust"
# 用记事本编辑 my-first-card\src\main.rs，保存 Ctrl+S
cargo run --offline -p labquest -- beginner check my-first-card
cargo run --offline -p labquest -- beginner hint 1
```

不要把 `# 用记事本编辑...` 当作需要执行的命令，它是说明文字。

## 学员最常见的四个卡点

| 现象 | 最可能的原因 | 下一步 |
|---|---|---|
| `cargo` 不是命令 | 未安装 / 重开终端之前的 PATH 未更新 | 按 `START_HERE.html` 安装官方 rustup，重开 PowerShell |
| `link.exe` 或 linker 报错 | 缺少 Windows MSVC C++ 工具 | 按 rustup 安装指引补齐 C++ Build Tools |
| 提示 `Cargo.toml` 找不到 | 终端不在正确文件夹 | 打开 RustAppLab 文件夹，在地址栏输入 `powershell` 再运行 |
| 功能检查没通过但编译成功 | 输出写死，没用变量 / 忘记保存 | 看“期待 / 实际”；只改一行，Ctrl+S，重试 |

**不要让学生通过删除目录来解决未知错误。** 初始工程和之前的成果都应受到保护；若目录存在，启动器直接继续而不是覆盖。

## 只需理解四个 Rust 概念

```rust
let name = std::env::args().nth(1).unwrap_or_else(|| "朋友".to_string());
let task = std::env::args().nth(2).unwrap_or_else(|| "学 Rust".to_string());
println!("今日任务卡 | {name} | {task}");
```

- `let`：给数据起名字。
- `name` / `task`：两个可以变化的文本数据，来自启动程序时的输入。
- `println!`：让终端显示信息，`{name}` 会插入变量的实际值。
- `cargo run -- ...`：将源码编译、启动程序，并把 `--` 后的文字作为程序参数。

## 完成后我学会了什么？

你已经练习了保存源码、使用变量生成文本、传入不同参数、观察期待与实际输出。命令行参数的读取代码是课程提供的，本节没有要求你独立编写它，也还没有要求掌握所有权、集合或复杂程序结构。

先不看提示，用两组自己的姓名和任务运行程序；解释 `name`、`task` 和 `println!` 各做什么。能指出自己改的文件、预测输出并重新运行，就有了可以进入下一课的基础。四项自动检查只验证程序行为，你自己的解释和新输入实验补充理解证据。

## 下一步：UnitLab，从显示文字到计算数值

回到 **RustAppLab 根目录**，双击 `CONTINUE_WINDOWS.cmd`，按 Enter 选择默认的 UnitLab。手动操作则在平台根目录执行：

```powershell
cargo run --locked --offline -p labquest -- studio start unitlab "my-unitlab"
cargo run --locked --offline -p labquest -- learn lesson "my-unitlab" 1
```

打开 `my-unitlab/src/lib.rs`，只做 `celsius_to_fahrenheit` 的 TODO(1)，保存后运行：

```powershell
cargo run --locked --offline -p labquest -- learn check "my-unitlab" 1
```

已有 UnitLab 工程就跳过 start，使用已有路径。详细的三步目标、自测和 QuizDeck 接续见 [FirstCard 完成后的衔接页](AFTER_FIRSTCARD.md)。[FileQuest 第 00 关](00_first_five_minutes.md) 保留为工具体验拓展；不是 FirstCard 后必须改走的路线。

## 授课效果的验收方式（尚未有真实学员统计）

不要以教程读完或 `cargo test` 全绿作为学会的证据。请观察：

1. 学员能否指出 `src/main.rs`，并解释是哪一行改变了输出？
2. 修改两个不同的名字/任务后，输出是否都随之变化？
3. 一次失败后，能否说出“期待输出与实际输出”的差别并自己重试？
4. 能否独立生成、找到并打开 `我的任务卡.txt`？
5. 学员最大的障碍是安装、编辑、读报错还是理解变量？记录**匿名**的障碍分类，不需要收集个人路径或用户名。

**验证状态**：本向导已经在 Linux 上进行 Rust 功能测试；Windows 原生运行需由 Windows 学员或 CI 真实执行后提供记录，不能把本指南当成 Windows 已通过的证明。
