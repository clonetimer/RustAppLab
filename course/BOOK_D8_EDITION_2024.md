# D8 · 用 Edition 2024 写一个告警筛选器

[现代化路线](MODERN_RUST_ROADMAP.md) · [新版 Book 映射](BOOK_MODERN_COVERAGE_D7.md) · [独立实验室](../projects/modern-d8-edition/README.md)

完成经典基础课程（所有权、借用、枚举、模式、Cargo）后再学习。本课不是把 rustc 升级等同于切换 Edition；同一个编译器允许不同 crate 使用不同的语言规则。使用 Rust 1.100.0，Book 基线 1.98+，Edition 2024。原工程 Edition 2021 / MSRV 1.74 不变。

## 0. 进入自己的工程

Windows PowerShell 与 Linux 的 Cargo 命令一致。首次安装：

```text
rustup toolchain install 1.100.0 --profile minimal --component rustfmt --component clippy
```

从 `projects/modern-d8-edition/learner` 复制整个目录到自己的空目录，例如 my-edition-lab。进入个人副本运行 `rustc --version`、`cargo --version`；带有 rust-toolchain.toml，因此固定本课版本，不修改全局默认。此课没有第三方依赖，工具链安装完成即可离线学习。打开 [学员操作说明](../projects/modern-d8-edition/learner/README.md)。

## 1. 在同一编译器下观察两种 Edition

完整源码包用户进入 `projects/modern-d8-edition/reference`，运行：

```text
cargo run --locked --offline -p d8-edition2021
cargo run --locked --offline -p d8-edition2024
cargo test --locked --offline --workspace --all-targets
```

两款程序的 shared.rs 相同。先预测 `drop` 和 `else` 哪个先发生、`const {2+2}` 命中哪个宏规则，再运行。期望 2021 输出 `drop=else,drop`、`macro=const`；2024 输出 `drop=drop,else`、`macro=expr`。这是合同的期望值，不是本次机器运行报告。不要通过给两款程序分别写死输出解释差异。

2024 缩短 if let 未匹配临时值的寿命，因此进入 else 前可释放临时借用；宏的 expr 片段可以匹配 const 块。把 expr 改为 expr_2021，可以保留原匹配行为。let chains 需要 Edition 2024，并且从 Rust 1.88 起稳定，并非 1.85 的所有功能都已包含它。

## 2. 三个学员任务

在自己的工程目录逐项执行，而不是在 RustAppLab 根目录运行。预测后修改 `src/lib.rs`，测试失败不要改 tests/contracts.rs。

| 任务 | 目标 | 测试 |
|---|---|---|
| 1 | select_alarm：Option 有值、非负、达到阈值才产生告警；用 let chain，不使用 unwrap | cargo test --locked --offline --test contracts step1_ |
| 2 | ensure_threshold：已存在值保持不变，缺失时填默认值；用 if let 与 RefCell，不能 panic 或丢失原值 | cargo test --locked --offline --test contracts step2_ |
| 3 | legacy_kind：普通表达式与 const 块分别命中预期规则；在 2024 中保留 2021 的片段匹配 | cargo test --locked --offline --test contracts step3_ |

提示分三级：先画输入与状态；再尝试 if let 与条件组合/观察临时借用；最后查官方 Edition Guide 中的 let chains、if let temporary scope、macro fragment specifiers。不要先查看参考答案。

全部通过后运行 `cargo test --locked --offline --all-targets`，用 `cargo run --locked --offline --example alarm -- 45 40` 查看告警。用 missing、负值、临界值及非法输入做迁移实验。添加一个自己的测试，用 LEARNING_LOG.md 写出旧 Edition 为什么不同。

## 3. 真正迁移一个旧 crate

在自己的新空目录创建 `cargo new --bin migration-lab --edition 2021`。用 `let gen = 7; println!("{gen}");` 替换 main 函数体，先 `cargo +1.100.0 run --offline`。观察直接改到 2024 时 gen 的保留字诊断，再恢复 2021，在这个练习副本中执行 `cargo +1.100.0 fix --edition --offline --allow-no-vcs`；若在 Git 仓库中，先提交或确保干净。查看源码是否变为 r#gen，随后手动改 Cargo.toml 的 edition，再运行。不要对 RustAppLab 旧工程执行迁移。

完整源码包维护者脚本还会真实编译同一份 let chain / gen / extern 片段的成功与失败版本；extern 声明改为 unsafe 不自动证明 ABI 签名安全，实验不调用外部符号。

## 4. Cargo resolver 3

虚拟 Workspace 需要显式选择 resolver。resolver 3 默认根据 package.rust-version 优先选择兼容依赖；这不是强制拒绝一切不兼容依赖。维护者脚本建立一个**合成的本地依赖目录源**，同一个虚构 crate 的两个版本分别声明 Rust 1.100 与 1.101；比较 resolver 2 和 3 的实际 Cargo.lock，绝不连接 crates.io，也不把虚构 crate 称为真实发布包。预期 resolver 2 选择 1.1.0，随后因最低版本过高而构建失败；resolver 3 选择并构建 1.0.0。迁移题：改变可用版本，解释没有兼容项时 fallback 为什么不能保证构建成功。

完整源包的维护者验收（不要求普通学员安装 Python）：

```text
python scripts/modern_d8_check.py --output d8-evidence
```

从平台根目录执行；使用 Python 3.11+，保留命令日志、返回码和报告，所有迁移、依赖和答案注入均在临时目录，不写回教材。输出目录必须不存在。

## 5. 验收边界

D8 只覆盖 2024 独立教学环境、选定规则和 resolver/迁移行为；不宣称所有 Edition 差异或 1.75–1.100 新特性已完整覆盖（D9/D10）。当前入口为独立编辑器 + Cargo，不宣称已接入网页工作台或固定 LabQuest 成绩。CI 与学员个人记录分离。

官方参考：[Edition Guide](https://doc.rust-lang.org/edition-guide/rust-2024/index.html) · [if let scope](https://doc.rust-lang.org/edition-guide/rust-2024/temporary-if-let-scope.html) · [macro fragments](https://doc.rust-lang.org/edition-guide/rust-2024/macro-fragment-specifiers.html) · [resolver](https://doc.rust-lang.org/edition-guide/rust-2024/cargo-resolver.html) · [unsafe extern](https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-extern.html) · [Rust 1.100.0](https://blog.rust-lang.org/2026/11/12/Rust-1.100.0/)。
