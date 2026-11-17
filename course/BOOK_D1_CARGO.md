# D1 · Cargo 进阶：从自己的代码到可安装工具

**对照原书**：[第 14 章：更多关于 Cargo 和 Crates.io 的内容](https://rust.bootcss.com/ch14-00-more-about-cargo.html)，对应 **14.1–14.5**。本课使用仓库中真正可编译、可测试的 [独立 Workspace](../projects/book-d1-cargo/README.md)，并复习原有 [进阶 B](ADVANCED_B.md)。建议完成 B1–B3 后学习。

**目标**：你能解释 Workspace、package、crate、profile、构建产物、安装根目录和自定义 Cargo 子命令的区别，能够在**不上传真实 crate、不修改全局环境**的条件下完成一次打包/安装演练，并为自己的工具添加一项经过测试的功能。

本课是现有 0.19.0-rc.1 的**可选补充**，不是宣称 0.19.0-rc.1 已经通过 D1 验收。独立项目使用 Edition 2021、最低 Rust 1.74、纯标准库，不自动联网；书中示例是 Edition 2018，两者在基本 Cargo 命令上具有可迁移性。Cargo 具体选项以你的版本及官方 Cargo Book 为准。

## 0 · 认识真实目录

在完整 RustAppLab 源码根目录执行以下命令；路径中的 `book-d1-cargo` 是**另一个** Workspace，不是根目录 17 个 app 的一部分：

```powershell
cargo metadata --locked --offline --no-deps --format-version 1 --manifest-path projects/book-d1-cargo/Cargo.toml
cargo test --locked --offline --workspace --all-targets --manifest-path projects/book-d1-cargo/Cargo.toml
cargo test --locked --offline --workspace --doc --manifest-path projects/book-d1-cargo/Cargo.toml
cargo run --locked --offline --manifest-path projects/book-d1-cargo/cargo-applab-map/Cargo.toml -- profiles
```

工作区内只有两个 package：`rustapplab-book-card`（业务库，`book-card/src/lib.rs`）和 `cargo-applab-map`（命令入口，`cargo-applab-map/src/main.rs`）。两者共用 `Cargo.lock`；CLI 使用 `path = "../book-card"` 依赖库。每一个 crate 的编译入口与 package/workspace 概念不可混为一谈。

**预测题**：运行 `cargo test` 时，为什么必须指定该独立 Workspace 的 `--manifest-path`？如果错误地在 RustAppLab 根目录执行 `cargo test --workspace`，你究竟验证了哪一组包？

## 14.1 · 构建 Profile：验证配置的归属

打开 [独立 Workspace 根 Cargo.toml](../projects/book-d1-cargo/Cargo.toml)，定位 `[profile.dev]` 与 `[profile.release]`。这两个表定义编译配置：`opt-level` 表示编译器优化等级，`lto` 是链接时优化，`codegen-units` 会影响构建并行度和优化机会。它们不构成业务正确性的证明，也不保证 release 一定更快。

```powershell
cargo build --locked --offline --manifest-path projects/book-d1-cargo/Cargo.toml --workspace
cargo build --locked --offline --manifest-path projects/book-d1-cargo/Cargo.toml --workspace --release
```

**任务 1**：复制整个 `projects/book-d1-cargo` 到自己的全新练习目录，把副本中的 `[profile.release]` 从 `opt-level = 2` 改为 `opt-level = "s"`，对比二进制体积和构建耗时，再恢复数值；不要在仓库教学基准里直接改。结论写为“在此工具链/此目标/此样例观察到”，不可推断普遍规律。只在 Workspace 根配置 profile；依赖包里写同名 profile 不会产生同样的效果。

## 14.2 · crate 打包与发布准备（不实际上传）

检查 `book-card/Cargo.toml`：`description`、`license`、`version`、`edition`、`rust-version` 都有意义。`publish = false` 是实验的安全护栏；**不可为了课程演示把公共 registry 写入凭据，也不要执行 `cargo publish`**。

```powershell
cargo package --list --allow-dirty --manifest-path projects/book-d1-cargo/book-card/Cargo.toml
```

这个命令只列出准备打包的文件，不会上传到 registry；`--allow-dirty` 仅允许查看带未提交修改的工作树，不等于允许正式发布。观察 `src/lib.rs`、manifest 是否进入包。现实项目的准备流程应包括许可证、README、敏感文件排除、公开 API、版本与 clean tree 复核。只有在自己的、明确可发布的测试项目上，才可单独考虑 `cargo publish --dry-run`；它与实际发布是不同操作，且 `publish=false` 的演示包会限制此流程。

**任务 2**：在个人副本新建 `internal-notes.txt`，通过 `package.include` 或 `package.exclude` 规则控制打包内容。再次运行 `cargo package --list`，证明内部文件没有进入清单。不要把真实私密文件放进练习目录。

## 14.3 · Workspace：不是多个目录碰巧放一起

对照本实验与 [进阶 B](ADVANCED_B.md)：本实验一个库供一个 CLI 复用；B1–B3 的 `taskcore` 则供 CLI 与 report 两个程序复用。两者都由顶层 Workspace 共享构建与测试；依赖版本解析由共享锁文件记录。某个 package 的 `publish=false` 也不会自动传播给其它包，各包须单独审查。

```powershell
cargo test --locked --offline --workspace --all-targets --manifest-path projects/book-d1-cargo/Cargo.toml
cargo test --locked --offline --workspace --doc --manifest-path projects/book-d1-cargo/Cargo.toml
```

**任务 3**：为 `book-card` 中的一个新主题增加单元测试，随后让 `cargo-applab-map` 在 CLI 中使用它，运行累计测试。解释为什么不应在 CLI 中复制业务匹配逻辑，而应该通过 crate 公共接口复用。

## 14.4 · cargo install：只安装到自己的临时目录

`cargo build` 生成当前工程构建产物；`cargo install` 会将二进制放入安装根目录的 `bin/`。用独立临时路径避免修改日常 `~/.cargo/bin` 或 Windows 默认工具目录。

**Windows PowerShell**（完整源码根目录）：

```powershell
$InstallRoot = Join-Path $env:TEMP ("rustapplab-book-d1-" + [guid]::NewGuid().ToString("N"))
cargo install --locked --offline --path projects/book-d1-cargo/cargo-applab-map --root "$InstallRoot"
& (Join-Path $InstallRoot "bin/cargo-applab-map.exe") profiles
$env:PATH = (Join-Path $InstallRoot "bin") + ";" + $env:PATH
cargo applab-map workspace
```

**Linux Bash**：

```bash
install_root=$(mktemp -d)
cargo install --locked --offline --path projects/book-d1-cargo/cargo-applab-map --root "$install_root"
"$install_root/bin/cargo-applab-map" profiles
PATH="$install_root/bin:$PATH" cargo applab-map workspace
```

这里的 PATH 仅影响当前 shell 进程，不是永久修改系统变量。练习结束后可手动清理你创建的唯一临时目录；不要使用 `--force` 或覆盖其他 Cargo 安装目录。本课不需要 `cargo install <crate-name>` 访问 crates.io，实际公共 crate 安装须另行检查来源、版本和缓存。

## 14.5 · 自定义 Cargo 子命令：观察真实参数

Rust 的外部 Cargo 子命令可以是 PATH 上名为 `cargo-<name>` 的可执行文件。在本例中，`cargo applab-map workspace` 找到 `cargo-applab-map`，再由二进制解析参数并调用 `rustapplab-book-card::describe`。

```powershell
cargo run --locked --offline --manifest-path projects/book-d1-cargo/cargo-applab-map/Cargo.toml -- workspace
cargo run --locked --offline --manifest-path projects/book-d1-cargo/cargo-applab-map/Cargo.toml -- invalid
```

第二条应得到非零退出码、stderr 中的中文提示，stdout 不应出现假成功内容。Cargo 作为外部子命令调用时会传入子命令名；直接运行该二进制则不一定有此前缀，代码须覆盖这两种路径。

**扩展挑战**：在你自己的 Workspace 副本中为 `book-card` 新增 `lint` 主题，文案必须解释 `cargo clippy` 的用途。为合法输入、不认识的主题、多余参数分别增加测试。先写测试让其失败，再实现并确认通过。

## D1 自查：必须真实交付，不靠读完讲义判定通过

| 验收点 | 能交付的证据 |
|---|---|
| 14.1 | 个人副本 profile 变化前后两次构建和对比记录 |
| 14.2 | `cargo package --list` 输出，证明私人笔记不在包内 |
| 14.3 | 两个 crate 的依赖图、单元测试和 CLI 测试 |
| 14.4 | 隔离安装目录的实际二进制运行输出 |
| 14.5 | `cargo applab-map` 成功命令、非法命令、自己新增的测试 |

**自动回归范围**：固定参考代码的 unit / integration / doctest、实际离线本地安装及 Cargo 外部子命令调用；个人任务的成果需人工提交，自动测试通过不等于独立理解，更不等于 registry 发布成功。

**官方资料**：[Cargo Profiles](https://doc.rust-lang.org/cargo/reference/profiles.html) · [cargo package](https://doc.rust-lang.org/cargo/commands/cargo-package.html) · [cargo install](https://doc.rust-lang.org/cargo/commands/cargo-install.html) · [Cargo Workspace](https://doc.rust-lang.org/cargo/reference/workspaces.html)。下一阶段回到 [D0 逐节审计](BOOK_COVERAGE_AUDIT.md)，优先补智能指针 15.2、15.3、15.6，而不是重复 C1/C2。
