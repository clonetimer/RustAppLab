# Rust AppLab 0.19.0-rc.6 · 从下载到第一次使用

**已经完成 FirstCard？直接打开 [FirstCard 做完之后](course/AFTER_FIRSTCARD.md)**：先确认自己能解释变量和参数，再进入 UnitLab。下面第 1–3 节是首次安装和起步步骤，不必每次重做。

## 先看懂你要走的前三步

| 作品 | 做出什么 | 学到什么 | 完成后去哪里 |
|---|---|---|---|
| FirstCard | 随姓名和任务变化的任务卡 | 编辑保存、变量、参数、格式化输出 | [复盘并创建 UnitLab](course/AFTER_FIRSTCARD.md) |
| UnitLab | 能计算、能拒绝非法换算的单位工具 | 函数、返回值、枚举、分支、错误处理 | [QuizDeck](apps/quizdeck/README.md) |
| QuizDeck | 使用自己题库的复习工具 | 结构体、文本解析、集合、借用与评分 | [TextScope](apps/textscope/README.md) |

每一课先跟着任务做，再用自己的输入验证并解释；不要求第一天读完所有课程。

## 1. 先选对包

**网页工作台从 0.19.0-rc.4 开始提供。** 在 [Releases](https://github.com/clonetimer/RustAppLab/releases) 选择 rc.6 的 `source.zip` 或本机对应的 `runtime.zip`，核对同目录校验文件；是否已发布以实际 Release 为准。[当前 main 源码 ZIP](https://github.com/clonetimer/RustAppLab/archive/refs/heads/main.zip) 也包含合入主线的修订。已有 `v0.19.0-rc.3` 下载包保持原样。

已下载旧包时，把新平台解压到另一个文件夹，保留已有 `my-...` 目录和个人记录。新平台可直接对旧工程路径执行第 4 节的 next/check，不需要重新完成 FirstCard。正式发布包仍以 Releases 中带版本号的 source.zip 及其校验值为准。

选择本机对应的 `windows-x64-msvc-runtime.zip` 或 `linux-x86_64-runtime.zip`，可直接打开已编译的网页工作台和参考程序。解压后程序在 `bin/`，先读 `START_RUNTIME.md`；FirstCard/Studio 的中文任务卡和编辑界面已内置，但运行、检查自己的源码仍需 Rust/Cargo。参考程序的成功不会记为学员成绩。

`learner-kit.zip` 是17个基础/进阶A/B学员工程的源码骨架。C1/C2另有 `learner-offline.zip`（核心 TODO）和 `reference-offline.zip`（完整参考），均带 vendor 依赖源码。**这些源码包都不包含 Rust 编译器、MSVC 或 Linux C 工具链。**

## 2. 准备开发环境

按照 [Rust 官方安装说明](https://rust-lang.org/tools/install/) 安装 Rust/Cargo。Windows 选择 x64 MSVC 工具链，并安装 C++ 构建工具和 Windows SDK；安装后重新打开 PowerShell。Linux 需要本机 C 编译/链接工具。

```powershell
rustc --version
cargo --version
```

基础课和进阶A/B最低Rust1.74。首次进入C1/C2，再安装固定附加工具链，不必修改全局默认版本：

```powershell
rustup toolchain install 1.85.0 --profile minimal
```

StoreDesk 是后续数据库课程，另需系统 SQLite；新手开始 FirstCard/UnitLab 不要求先安装 SQLite。详情见 [SQLite 准备](course/SQLITE_SETUP.md)。

## 3. 零基础：FirstCard → UnitLab

**推荐使用同页学习流程：** Windows 双击 `START_WORKBENCH_WINDOWS.cmd`；Linux 执行 `./START_WORKBENCH.sh`。保留启动窗口，浏览器会打开本机地址。在左侧选择 FirstCard，点击代码区的“开始本课”，阅读当前中文任务卡，直接在编辑区修改自己的源码，按“保存”后点击“检查本步”。失败时底部输出区会展开并显示原检查器的错误；通过后换自己的输入运行，再进入 UnitLab。完整操作见 [网页工作台指南](course/WORKBENCH.md)。

原有终端流程继续可用。在完整源码包根目录打开 PowerShell 或终端：


```powershell
cargo run --locked --offline -p labquest -- beginner start "my-first-card"
cargo run --locked --offline -p labquest -- beginner check "my-first-card"
```

第二条命令初次失败是预期：打开 `my-first-card/src/main.rs`，按照 **`my-first-card/跟着做.md`** 完成任务卡的一行 TODO，保存后再次执行 `beginner check`。通过后从平台根目录运行自己的作品：

```powershell
cargo run --offline --manifest-path "my-first-card/Cargo.toml" -- "小林" "写 Rust"
```

预期输出包含昵称和任务。Windows 也可双击 `START_WINDOWS.cmd`，按照提示编辑自己的源码。

**完成本课不只是看到“通过”。** 不改源码，再换两个名字和任务；说清楚 `name` / `task` 保存了什么，`println!` 做了什么，以及 `--` 后的文字给谁使用。记录自己的运行结果。你已经练习了“输入变化，输出也变化”，函数、分支和错误处理会在下一课继续学。

完成后用 `CONTINUE_WINDOWS.cmd` 创建 UnitLab；Windows 菜单按 Enter 选默认的 UnitLab。手动命令仍在**平台根目录**执行：

```powershell
cargo run --locked --offline -p labquest -- studio start unitlab "my-unitlab"
cargo run --locked --offline -p labquest -- learn lesson "my-unitlab" 1
```

然后打开 **`my-unitlab/src/lib.rs`**，只修改 `celsius_to_fahrenheit` 的 `TODO(1)`，按 Ctrl+S 保存，再执行：

```powershell
cargo run --locked --offline -p labquest -- learn check "my-unitlab" 1
```

每次仅完成任务卡指定函数。第 1 步学计算和返回值；第 2 步用枚举和分支选择单位；第 3 步解析文本并处理错误。全部三步通过后，才运行完整 CLI。保持在平台根目录：

```powershell
cargo run --offline --manifest-path "my-unitlab/Cargo.toml" -- convert -10 c f
```

预期 `-10.00 c = 14.00 f`。然后试自己的数值和一组非法换算，记录结果，再去 QuizDeck。详细的第 2、3 步、提示命令、自测与下一课创建步骤都在 [FirstCard 完成后的衔接页](course/AFTER_FIRSTCARD.md)。

## 4. 已有工程：不重建、不覆盖

打开网页工作台，先选择对应课程，再在已有项目目录输入框中输入原来的 `my-...` 完整路径。路径可以含中文和空格。已有工程缺少新版讲义时，工作台仍显示平台内置任务卡。源码在外部编辑器中变化后，旧草稿保存会被拒绝并保留，避免覆盖外部修改；旧通过记录会标记为历史结果，需重新检查当前源码。

使用完整平台的 `LEARN_NEXT_WINDOWS.cmd`，输入已有工程的实际路径。L看任务卡、E编辑、C真实检查、1/2/3分级提示；Q退出不删除作品。命令行也可以：

```powershell
cargo run --locked --offline -p labquest -- learn next "D:/学习/my-unitlab"
cargo run --locked --offline -p labquest -- learn check "D:/学习/my-unitlab"
```

`start` 只用于全新目录。已经解压的 C1/C2 学员包带工程标记，直接传入路径检查，不要再次 start。路径有中文或空格时加引号。

## 5. 当前作品完成后，按什么顺序继续？

近三课先走 **UnitLab → QuizDeck → TextScope**。然后依次学习 LedgerLite（持久记录）、BadgeForge（trait 与输出）、TermGarden（交互状态）；再做线程、存储、网络、事务和异步等待，最后独立完成 TaskPocket / PageMint。每个应用的 README 都有具体“下一步”和自测要求，完整入口见 [课程地图](course/CURRICULUM.md)。

**每次结课留下四样东西：自己的源码、自己的输入、真实结果、两三句概念解释。** 能完成后再进入下一课；只运行过参考程序，尚不能算完成自己的实现。

## 6. 完成基础与进阶前置后，再学 B4、C1、C2

B4先看 [进阶B讲义](course/ADVANCED_B.md)，显式执行 `advanced-b prepare-json <工程>` 准备固定依赖；已有B1–B3工程先按讲义执行 `add-json`。

C1/C2依次看 [C1](course/ADVANCED_C1.md) 与 [C2](course/ADVANCED_C2.md)。完整平台的 `advanced-c1 prepare <工程>` / `advanced-c2 prepare <工程>` 首次允许联网；之后固定评分使用离线依赖。单独的离线包在其 `Cargo.toml` 目录使用 `cargo +1.85.0 test --frozen --all-targets`。初始学员 TODO 失败不表示依赖包损坏。

参考C2离线源码包可直接体验：

```powershell
cargo +1.85.0 run --frozen -- init "tickets.db"
cargo +1.85.0 run --frozen -- serve "tickets.db" --port 3000
```

打开 `http://127.0.0.1:3000/`。重新启动已有数据只执行 serve，不重复 init。C2关闭等待已进入的请求，未提供强制关闭截止时间。

## 常见问题

| 现象 | 处理 |
|---|---|
| 找不到 cargo 或链接器 | 完成Rust/MSVC安装，重新打开终端；不要先改项目源码 |
| 提示目录已存在 | 旧工程用 next/check；新作品换一个未使用的目录 |
| 提示缺少固定依赖 | 按该课显式 prepare；离线C1/C2固定评分设置相应 `RUSTAPPLAB_C1_VENDOR` / `RUSTAPPLAB_C2_VENDOR` |
| 检查报 TODO 或断言失败 | 阅读任务卡，只改指定函数，保存后重试；不删除测试 |
| 进入了错误目录 | `cargo ... -p labquest` 在完整平台根目录；学员自己的 `cargo run/test` 在学员目录 |
| Rust源码包能解压但不能双击运行 | 源码需要编译；直接体验用对应平台runtime包 |
| 浏览器未自动打开 | 保留启动窗口，复制其中的 `http://127.0.0.1:端口/` 地址到浏览器 |
| 网页提示保存冲突或工程已在另一页面切换 | 先保留编辑区草稿，重新打开正确工程并核对磁盘内容后保存 |
| 查看结果时连接中断 | 使用页面的重试入口继续查看原操作；不要因等待而反复提交检查 |

## 试用与反馈

使用 [试用记录](course/USER_TRIAL.md) 保存包名、版本、系统、实际命令、原始错误和自己的结果。它是空白记录模板，未代表真人验收通过。当前候选版先保持范围稳定，后续只根据真实问题修复。

## 按需补课：不会哪个概念，再打开相应材料

主线仍是 FirstCard → UnitLab。若分不清工作目录或 check/build/run，可补做 [Cargo 三步复习](course/BOOK_D6_FOUNDATIONS.md)；若变量与模块概念不清，再看 [变量与模块路径](course/BOOK_D6_LANGUAGE_BASICS.md)。这些是已有参考实验，不要求先全部做完才能继续作品。

接着按顺序做三组短实验：[所有权、借用与切片](course/BOOK_OWNERSHIP_PRACTICE.md) → [集合与错误传播](course/BOOK_COLLECTIONS_ERRORS.md) → [泛型与生命周期](course/BOOK_GENERICS_LIFETIMES.md)。每组都能单独运行，先预测，再修改自己的副本，最后解释成功或编译失败的原因。它们复用已有参考工作区，不需要重新生成已完成的学员工程。

工具、Edition 迁移和选修 Nightly 使用 [D6 实操指南](course/BOOK_D6_PRACTICAL.md)。需要 Python 3.10+ 和明确列出的 Rust 组件；所有改写只在独立副本进行。参考脚本成功不会自动记为个人课程通过。
