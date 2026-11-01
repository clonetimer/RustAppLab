# 第 0 阶段：Windows 学员从哪里开始（约 30–45 分钟）

**0.11.1 更新**：已修复 0.11.0 Windows 向导创建 `my-first-card` 时误报父目录不存在的问题。保留已有学习文件，更新源码或应用热修复包即可。

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

## 下一步怎样衔接现有应用

通过后先回到 [第 00 关：FileQuest 文件雷达](00_first_five_minutes.md)，生成 `report.html`。此时学员已经知道什么是源文件、命令、输入、输出，所以不会突然面对多份看不懂的工程代码。随后按顺序走 01–07。CASE 01、TaskPocket 和 PageMint 都是进阶目标，**不是入门前置条件**。

## 授课效果的验收方式（尚未有真实学员统计）

不要以教程读完或 `cargo test` 全绿作为学会的证据。请观察：

1. 学员能否指出 `src/main.rs`，并解释是哪一行改变了输出？
2. 修改两个不同的名字/任务后，输出是否都随之变化？
3. 一次失败后，能否说出“期待输出与实际输出”的差别并自己重试？
4. 能否独立生成、找到并打开 `我的任务卡.txt`？
5. 学员最大的障碍是安装、编辑、读报错还是理解变量？记录**匿名**的障碍分类，不需要收集个人路径或用户名。

**验证状态**：本向导已经在 Linux 上进行 Rust 功能测试；Windows 原生运行需由 Windows 学员或 CI 真实执行后提供记录，不能把本指南当成 Windows 已通过的证明。
