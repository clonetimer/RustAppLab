# D6 · 开发工具、Edition 迁移与 Nightly 实操

**从这里开始**：准备工具链，运行一个实验，再读日志、修改副本并解释结果。配套 [执行脚本](../scripts/book_d6_check.py) 使用 Python 3.10+ 标准库，Windows 与 Linux 共用；工具链安装需要网络，实验中的 Cargo 命令使用本地、零第三方依赖的参考工程。

所有可能改写源码的操作都在本次新建的临时副本中进行。每次使用一个新的证据目录；已存在的目录会被拒绝，避免覆盖上一次结果。命令失败会返回非零退出码、保存日志和失败副本。

## 0. 先分清三个版本

| 对象 | 本课使用的版本 | 含义 |
|---|---|---|
| 正式基础工程 | Edition 2021 / Rust 1.74 MSRV | 根 Workspace 的 17 个包及 D1–D6 基础实验继续保留原基线 |
| 工具与迁移实验 | Rust 1.85.0，迁移后 Edition 2024 | 选定可重放版本；只改变临时副本的 edition 和 rust-version |
| 不稳定功能实验 | nightly-2026-11-17 | 显式选修，不进入默认 CI、正式 Workspace 或 MSRV 结论 |

Edition 是每个 crate 选择的语言规则，MSRV 是它要求的最低编译器版本。用较新的编译器测试成功，并不自动改变正式工程的最低版本承诺。

## 1. 准备与证据目录

在完整源码包或学员包根目录执行。最小工具链默认不包含 rustfmt、Clippy、rust-docs 和 rust-analyzer，因此显式安装本课会使用的组件。实验命令始终指定同一个工具链，避免 IDE 或目录 override 改变实际版本。

### Windows PowerShell

~~~powershell
rustup toolchain install 1.85.0 --profile minimal --component rustfmt --component clippy --component rust-docs --component rust-analyzer
if ($LASTEXITCODE -ne 0) { throw "工具链准备失败，请先查看上面的错误" }
$evidence = Join-Path $env:TEMP ("RustAppLab-D6-" + [guid]::NewGuid().ToString("N"))
python scripts/book_d6_check.py --mode tools --toolchain 1.85.0 --output (Join-Path $evidence "tools") --keep-workspace
if ($LASTEXITCODE -ne 0) { throw "D 实验失败；日志和副本已保留" }
Write-Host "证据目录：$evidence"
~~~

### Linux / WSL shell

~~~sh
set -eu
rustup toolchain install 1.85.0 --profile minimal --component rustfmt --component clippy --component rust-docs --component rust-analyzer
d6_evidence=$(mktemp -d "${TMPDIR:-/tmp}/RustAppLab-D6-evidence.XXXXXX")
python3 scripts/book_d6_check.py --mode tools --toolchain 1.85.0 --output "$d6_evidence/tools" --keep-workspace
printf '证据目录：%s\n' "$d6_evidence"
~~~

后续实验继续使用当前终端中的证据目录变量。重新开终端时，先创建新的目录变量。`--keep-workspace` 会保留成功副本供手动练习。默认在命令成功并保存证据后清理本次副本。D/E 保留副本时，终端的 `Open in VS Code` 行给出可直接打开的工作区文件。也可在 `report.json` 中读取以下路径：

| 字段 | 对应位置 |
|---|---|
| `project_directory` | 含工作区 Cargo.toml 的练习副本目录 |
| `ide_workspace_file` | 可用 VS Code 打开的 `D6-IDE.code-workspace` |
| `practice_source` | 可编辑的 `appendix-lab/src/lib.rs` |
| `documentation_index` | 已生成的 crate 文档首页 |

清理后的字段值为 `null`，缺失的产物也显示 `null`。旧字段 `workspace` 记录本次临时目录位置，需结合 `workspace_retained` 判断目录是否保留。D/E 的证据目录保存初始工作区配置 `ide-workspace.json` 及其 SHA-256，便于清理副本后复查。VS Code 的打开入口仍是保留副本中的 `D6-IDE.code-workspace`。

## 2. 附录 D：工具必须实际运行

脚本记录 rustc、Cargo、rustfmt、Clippy、rustdoc、rust-analyzer 的版本，并在副本执行格式化与检查、Clippy、Debug/Release 测试、doctest 和文档生成。所有原始输出与退出码都保留在证据目录。格式检查成功只说明格式符合规则，不能代替业务测试。

**先预测**：一个变量从不修改却写了 `mut`，属于编译错误还是警告？`-D warnings` 会怎样改变命令退出码？

**再观察**：比较工具诊断与修复后的日志。`cargo fmt` 调整布局，`cargo clippy` 检查常见问题，`cargo fix` 根据编译器可自动应用的建议修改源码，`cargo doc` 生成 API 文档。它们解决不同问题。

### 在 VS Code 中亲手练习

1. 手动准备 VS Code 和 `rust-lang.rust-analyzer` 扩展，显式安装实验工具链的标准库源码。默认工具链的命令为 `rustup component add rust-src --toolchain 1.85.0`。通过 WSL 运行实验时，在 VS Code 的 WSL 窗口打开副本，并确认 Rust 工具链、标准库源码和扩展都在对应环境可用。
2. 使用 **File → Open Workspace from File…** 打开 `ide_workspace_file`。该文件使用相对路径 `lab`，搬移时应移动整个保留目录。搬移后从新位置打开工作区文件，旧报告仍记录运行当时的路径。工作区为 rust-analyzer 的 Cargo 命令指定本次工具链及 `--locked --offline`，并关闭 build script 和 proc macro 支持。
3. 在 **D6 练习副本**打开 `appendix-lab/src/lib.rs`，找到 `task_priorities` 里的 `Task::new(7, "document")`。把光标放在 `new` 上，按 **F12** 或使用 **Go to Definition**，记录 `impl Task` 中构造函数的文件位置和参数类型。
4. 返回 `task_priorities`，在函数体内加入 `let _d6_id: u32 = "7";` 并保存。在 **Problems** 面板观察类型不匹配诊断（rustc 代码为 `E0308`）。如果 IDE 尚未显示诊断，在 `project_directory` 中运行下面的命令确认编译器报错，再检查扩展是否加载了项目。
5. 把 `"7"` 改成整数 `7` 并保存，确认诊断消失。在同一目录复跑命令确认退出码为 0。Quick Fix 显示的操作取决于具体诊断，这里可直接手动修复。若指定过其他工具链，把命令中的 `1.85.0` 换成本次报告的 `toolchain` 值。

~~~sh
cargo +1.85.0 check --locked --offline --workspace --all-targets
~~~

从报告的 `documentation_index` 打开生成的 API 文档，找到 `Task` 的字段名称和类型。把版本、跳转位置、诊断与修复结果写入工作区文件旁的 `IDE_RECORD.md`。它初始为 `NOT_RUN`，由你按实际操作填写，位于 CLI 证据目录之外。

`rust-analyzer-version.txt` 记录 Rustup 组件的版本。VS Code 扩展页面显示扩展版本，扩展还可以使用随附的 rust-analyzer server。查看 VS Code 的 Rust Analyzer 输出或状态信息，记录实际 server 版本。若加载失败，先核对工作区入口、扩展的运行环境、Rustup 工具链和 `rust-src`。当前扩展与旧版工具链的标准库源码可能不兼容，可在确认兼容的扩展和 server 后复试。[rust-analyzer 安装](https://rust-analyzer.github.io/book/installation.html) · [rust-analyzer 配置](https://rust-analyzer.github.io/book/configuration.html) · [VS Code Rust 指南](https://code.visualstudio.com/docs/languages/rust) · [VS Code 工作区](https://code.visualstudio.com/docs/editing/workspaces/multi-root-workspaces)

CLI `PASS` 只涵盖报告中的实际命令。D 的跳转、诊断和修复由学员在 IDE 中完成并记录。

## 3. 附录 E：2021 → 2024 的迁移顺序

### 运行

PowerShell：

~~~powershell
python scripts/book_d6_check.py --mode edition --toolchain 1.85.0 --output (Join-Path $evidence "edition") --keep-workspace
if ($LASTEXITCODE -ne 0) { throw "E 实验失败；请检查日志，失败副本仍保留" }
~~~

Linux / WSL：

~~~sh
python3 scripts/book_d6_check.py --mode edition --toolchain 1.85.0 --output "$d6_evidence/edition" --keep-workspace
~~~

### 读懂实验

1. 先在 Edition 2021 副本运行原有测试。
2. 加入旧 Edition 合法的 `gen` 标识符探针，让迁移必须产生可观察修改。
3. **保持旧 Edition**，执行 `cargo +1.85.0 fix --edition`，让编译器修复向下一版迁移时的不兼容代码。
4. 再把副本 Cargo.toml 改为 `edition = "2024"`、`rust-version = "1.85"`。
5. 在副本执行格式化，重新检查 Clippy、Debug/Release、doctest，并保存迁移前后的 manifest、源码与 diff。
6. 核对正式工程的源码和配置快照没有变化。

**先预测**：2024 把 `gen` 保留为关键字，旧函数名应怎样表达？在日志和 diff 中找到 `r#gen`，解释它为什么能保留原来的函数含义。

**亲手练习**：在保留副本中修改一个测试输入，重跑该测试；比较迁移前后结果。解释为什么本实验不能作为“正式工程 MSRV 已升级到 1.85”的依据。

PowerShell 可直接从报告读取正确的 manifest 路径后重跑：

~~~powershell
$report = Get-Content -Raw (Join-Path $evidence "edition/report.json") | ConvertFrom-Json
$manifest = Join-Path $report.project_directory "Cargo.toml"
cargo +1.85.0 test --locked --offline --manifest-path $manifest --workspace --all-targets
if ($LASTEXITCODE -ne 0) { throw "个人修改后的测试失败，请阅读诊断" }
~~~

Linux / WSL：

~~~sh
d6_manifest=$(python3 -c 'import json,pathlib,sys; print(pathlib.Path(json.load(open(sys.argv[1]))["project_directory"])/"Cargo.toml")' "$d6_evidence/edition/report.json")
cargo +1.85.0 test --locked --offline --manifest-path "$d6_manifest" --workspace --all-targets
~~~

失败时先区分组件未安装、路径错误、迁移错误和依赖/MSRV 问题。不要修改正式工程来绕过副本失败。官方顺序见 [Edition Guide](https://doc.rust-lang.org/edition-guide/editions/transitioning-an-existing-project-to-a-new-edition.html)。

## 4. 附录 G：Nightly 与两个 Stable 对照

此实验单独选择、单独安装，普通基础课不需要 Nightly。使用固定日期，不使用随时间变化的 `+nightly`。

PowerShell：

~~~powershell
rustup toolchain install nightly-2026-11-17 --profile minimal
if ($LASTEXITCODE -ne 0) { throw "指定 Nightly 安装失败；请记录原因" }
python scripts/book_d6_check.py --mode nightly --toolchain nightly-2026-11-17 --stable-toolchain 1.85.0 --output (Join-Path $evidence "nightly") --keep-workspace
if ($LASTEXITCODE -ne 0) { throw "G 对照实验失败；请检查报告中的具体步骤" }
~~~

Linux / WSL：

~~~sh
rustup toolchain install nightly-2026-11-17 --profile minimal
python3 scripts/book_d6_check.py --mode nightly --toolchain nightly-2026-11-17 --stable-toolchain 1.85.0 --output "$d6_evidence/nightly" --keep-workspace
~~~

脚本在唯一临时 crate 内创建 `Square<const N: usize>`，使用 `generic_const_exprs` 表达长度 `N * N`。检查三件事：

| 对照 | 预期结果 | 证明的边界 |
|---|---|---|
| 指定 Nightly + feature gate | 测试通过，N=3 的存储长度是 9 | 此日期工具链能运行该实验 |
| Rust 1.85.0 + feature gate | 编译失败并包含 E0554 | Stable 拒绝不稳定功能门控 |
| Rust 1.85.0，去掉 feature gate | 泛型常量表达式仍编译失败 | 失败不只是遗留了一个无用的 feature 属性 |

负对照的编译失败是预期行为，必须匹配相应诊断；网络失败、工具缺失或任意非零退出不能冒充正确的负对照。不要设置 `RUSTC_BOOTSTRAP` 绕过通道限制。

根 Cargo.toml、Cargo.lock 和可能存在的 rust-toolchain 配置在实验前后必须一致；原本不存在的配置也不能凭空出现。Nightly 结果**不计入 Stable**、Rust 1.74 MSRV 或真人学习证据。指定构建不可用时保留安装失败信息，不能改用别的通道后仍声称同一实验通过。

## 5. 个人完成记录与清理

| 项目 | 保留什么 | 你需要解释什么 |
|---|---|---|
| D · 工具 | 版本、命令日志、IDE 跳转/诊断记录 | format、Clippy、fix、test、doc 各解决什么问题 |
| E · Edition | 迁移前后 manifest、源码 diff、测试结果 | 为什么先 fix 后改 edition；Edition 与 MSRV 的区别 |
| G · Nightly | 日期化版本、成功日志、两个 Stable 失败日志 | 为什么需要两个负对照，为什么不影响正式工程 |
| 个人修改 | 你改过的输入、测试和产物 | 哪一项修改让你发现原来的理解有误 |

先检查 report.json 的结果、命令退出码和源码一致性，再复制证据到自己的学习记录。需要清理时，核对报告中的唯一路径，只删除本次实验副本；证据目录另行保留。工具成功不是个人掌握，IDE 操作、修改练习与解释题仍由学习者完成。

返回 [D6 附录讲义](BOOK_D6_APPENDICES.md) · [基础复习与数组/元组](BOOK_D6_FOUNDATIONS.md) · [变量与模块路径](BOOK_D6_LANGUAGE_BASICS.md)
