# 0.11.1 · 修复 Windows 首次创建练习目录失败

## 你看到的错误

```text
Finished `dev` profile ...
Running `target\debug\labquest.exe beginner start my-first-card`
LabQuest 错误：上级文件夹不存在：my-first-card。
```

**这是旧版 LabQuest 的路径判断问题，不是 Rust、Cargo 或 MSVC 安装失败。** `unused_mut` 编译警告也不是导致退出的原因。

## 最推荐：升级到 0.11.1

- 如果你还没做过练习：将 0.11.1 完整源码 ZIP 解压到新目录，进入其中的 `RustAppLab`，双击 `START_WINDOWS.cmd`。
- 如果已在 0.11.0 开始练习：不要删除 `my-first-card` 或任何个人资料。把 0.11.1 Windows 热修复包内的 3 个文件，按原有相对目录覆盖到当前 RustAppLab 根目录：
  1. `apps/labquest/src/beginner.rs`
  2. `apps/labquest/src/transfer.rs`
  3. `scripts/start_windows.ps1`

随后双击原来的 `START_WINDOWS.cmd`。它会通过 Cargo 自动重新编译修改过的 LabQuest。

## 立即绕过旧版错误（不修改源码）

如果你暂时不能下载新包，可在 **RustAppLab 根目录** 打开 PowerShell，使用绝对路径创建：

```powershell
cargo run --offline -p labquest -- beginner start "$((Get-Location).Path)\my-first-card"
```

成功后使用以下命令继续（旧版 Windows 双击向导仍使用相对路径，建议升级后重新双击）：

```powershell
cargo run --offline --manifest-path my-first-card/Cargo.toml -- "小林" "写 Rust"
notepad my-first-card\src\main.rs
# 修改 println! 后按 Ctrl+S 保存
cargo run --offline -p labquest -- beginner check "$((Get-Location).Path)\my-first-card"
```

如果目标目录已经存在，不要尝试删除。`beginner start` 默认拒绝覆盖已有工程，你可以继续编辑原有 `src/main.rs`。

## 开发者验证

旧版错误已在 Linux 上使用与 Windows 启动器相同的单层相对参数复现。0.11.1 以后增加对应集成测试，并在 Windows CI 冒烟脚本中补充该场景。本地 Linux 测试不等于 Windows 原生实机已通过。
