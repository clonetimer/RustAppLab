# Rust AppLab 0.19.0-rc.1 · 从下载到第一次使用

## 1. 先选对包

**完整学习推荐 `RustAppLab-0.19.0-rc.1-source.zip`。** 解压后包含 `apps/`、`course/`、`START_HERE.html` 和 Windows 向导。GitHub 自动生成的 Source code 归档是仓库快照；正式交付用带版本号的 source.zip 及其校验值。

只体验成品，选择本机对应的 `windows-x64-msvc-runtime.zip` 或 `linux-x86_64-runtime.zip`。解压后程序在 `bin/`，先读其中 `START_RUNTIME.md`；它不包含完整源码目录，也不把参考程序的成功视为学员成绩。

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

在完整源码包根目录打开 PowerShell 或终端：

```powershell
cargo run --locked --offline -p labquest -- beginner start "my-first-card"
cargo run --locked --offline -p labquest -- beginner check "my-first-card"
```

第二条命令初次失败是预期：打开 `my-first-card/src/main.rs`，按照同目录 `跟着做.md` 完成任务卡的一行 TODO，保存后再次执行 `beginner check`。通过后从平台根目录运行自己的作品：

```powershell
cargo run --offline --manifest-path "my-first-card/Cargo.toml" -- "小林" "写 Rust"
```

预期输出包含昵称和任务。Windows 也可双击 `START_WINDOWS.cmd`，按照提示编辑自己的源码。完成后用 `CONTINUE_WINDOWS.cmd` 创建 UnitLab，或执行：

```powershell
cargo run --locked --offline -p labquest -- studio start unitlab "my-unitlab"
cargo run --locked --offline -p labquest -- learn lesson "my-unitlab"
cargo run --locked --offline -p labquest -- learn check "my-unitlab" 1
```

每次仅完成任务卡指定函数。全部三步通过后，才进入学员目录运行完整 CLI：`cargo run --offline -- convert -10 c f`，预期 `-10.00 c = 14.00 f`。

## 4. 已有工程：不重建、不覆盖

使用完整平台的 `LEARN_NEXT_WINDOWS.cmd`，输入已有工程的实际路径。L看任务卡、E编辑、C真实检查、1/2/3分级提示；Q退出不删除作品。命令行也可以：

```powershell
cargo run --locked --offline -p labquest -- learn next "D:/学习/my-unitlab"
cargo run --locked --offline -p labquest -- learn check "D:/学习/my-unitlab"
```

`start` 只用于全新目录。已经解压的 C1/C2 学员包带工程标记，直接传入路径检查，不要再次 start。路径有中文或空格时加引号。

## 5. 完成基础后再学 B4、C1、C2

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

## 试用与反馈

使用 [试用记录](course/USER_TRIAL.md) 保存包名、版本、系统、实际命令、原始错误和自己的结果。它是空白记录模板，未代表真人验收通过。当前候选版先保持范围稳定，后续只根据真实问题修复。
