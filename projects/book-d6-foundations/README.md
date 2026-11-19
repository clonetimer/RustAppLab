# Book D6-B · 基础知识参考实验

这是 Rust Book 第 1–13 章的**可选补充 Workspace**：用一个小 CLI、可复用函数、集成测试和编译失败示例，补齐 Cargo、语言基础及核心概念。它不是新的学员作品，不属于根工作区的 17 个包。

## 怎样选用本实验

刚完成 FirstCard 时，主线仍是 [基础后续指南 → UnitLab](../../course/AFTER_FIRSTCARD.md)。只有当你不清楚 Cargo、数组 / 元组或正在卡某个概念时，选下面的一小节来补；不需要读完本目录才回到自己的作品。

Edition 2021、最低 Rust 1.74，零 registry 依赖、`publish = false`。完整起步说明在 [D6-B 讲义](../../course/BOOK_D6_FOUNDATIONS.md)，变量与模块另见 [专项讲义](../../course/BOOK_D6_LANGUAGE_BASICS.md)。

| 你的问题 | 源码 / 测试入口 | 配套阅读 |
|---|---|---|
| check、build、run 有何区别 | `Cargo.toml`、`foundation-lab/Cargo.toml`、`src/main.rs` | D6-B 讲义的 Cargo 三步练习 |
| 元组、数组、返回值怎么用 | `foundation-lab/src/lib.rs` 的 `summarize_scores`、`tests/compound_types.rs` | D6-B 复合类型部分 |
| 变量、常量、模块和路径 | `src/language_basics.rs`、`src/language_basics/catalog.rs` | 变量与模块专项 |
| move、接收者、切片 | `src/ownership.rs`、`tests/ownership.rs` | [所有权练习](../../course/BOOK_OWNERSHIP_PRACTICE.md) |
| 集合、错误传播 | `src/collections.rs`、`tests/collections.rs` | [集合与错误](../../course/BOOK_COLLECTIONS_ERRORS.md) |
| 泛型、trait、生命周期 | `src/generic_lifetimes.rs`、对应集成测试 | [泛型与生命周期](../../course/BOOK_GENERICS_LIFETIMES.md) |

表中 `src/`、`tests/` 均在 `foundation-lab/` 包内。各模块包含可运行或编译失败示例，按问题读取，不以目录数量判断学习进度。

## 先弄清 Cargo 在操作什么

| 名词 | 本实验中的对应关系 |
|---|---|
| Workspace | 本目录 `Cargo.toml` 管理 `foundation-lab` 成员，与平台根 Workspace 分开 |
| Package | 包名 `book-d6-foundation-lab`，用 `-p` 选择 |
| Library crate | `foundation-lab/src/lib.rs`，Rust 中导入为 `book_d6_foundation_lab` |
| Binary crate | `foundation-lab/src/main.rs`，目标名 `foundation-lab`，用 `--bin` 选择 |
| Cargo.toml | Workspace 管成员与 profile，包清单管包、库及二进制目标 |
| Cargo.lock | 记录解析结果；保留版本控制，不手工改写 |

在 **源码根目录或学员包根目录**（能看到 `projects/`）依次检查、构建、运行：

```powershell
cargo check --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab
cargo build --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab
cargo run --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab --bin foundation-lab -- hello
cargo run --release --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab --bin foundation-lab -- hello
```

后两条都应输出 `Hello, world!`。`check` 不产生可运行程序；`build` 产生程序；`run` 必要时构建后执行。`--` 后的 `hello` 交给程序。

未覆盖 target 目录时，程序位于本独立 Workspace 的 `target/debug/` 或 `target/release/`，Windows 带 `.exe`。dev 的 opt-level 是 0，release 是 2；优化配置改变不应改变问候行为。

## 再选一个能预测输出的小实验

仍在 **同一个根目录**运行：

```powershell
cargo run --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab -- words "rust ownership rust"
cargo run --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab -- guess
cargo run --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab -- bench 100000
```

| 命令 | 自查结果 |
|---|---|
| `words` | 按键排序输出 `ownership: 1`、`rust: 2` |
| `guess` | 等待终端输入；合法范围为 1–100，输入 q 退出；非法文本不会当成正确数字 |
| `bench 100000` | loop 与 iterator 总和应相等；耗时仅供观察，没有某一种必胜的承诺 |

`config` 还可显示从 `RUSTAPPLAB_HOME` 或系统用户目录推导出的配置路径。猜数用时间种子 xorshift 保持零依赖，不是密码学安全随机数。

## 数组与元组：把结果说清楚

`summarize_scores([u32; 3]) -> (u64, u32)` 接收恰好三个分数，返回总和与最大值。例如 `[72, 95, 81]` 得到 `(248, 95)`。它接受所有 u32 值，不限定 0–100；逐项扩宽为 u64 后相加，因此三个 u32::MAX 在 debug / release 下都有精确结果。

在 **同一个根目录**运行聚焦检查：

```powershell
cargo test --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab --test compound_types
cargo test --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab --doc summarize_scores
```

rustdoc 演示混合元组、解构、`.0` / `.1`、unit `()`、`[0_u32; 3]`，以及索引与 `.get()` 的区别。错误数组长度和浮点元素的 `compile_fail` 示例通过，意味着编译器按预期拒绝了它们。

## 做一次个人迁移，再回作品

复制整个实验到新的个人目录。在副本的 `foundation-lab/tests/compound_types.rs` 增加自己的三项成绩，先手算总和与最大值，再验证；换成两项时解释类型为何不匹配，不修改接口来躲过这个问题。

在 **副本 Workspace 根目录**运行：

```powershell
cargo test --locked --offline -p book-d6-foundation-lab --test compound_types
cargo test --locked --offline -p book-d6-foundation-lab --doc summarize_scores
```

把输入、预测、实际结果和一次编译诊断写进学习日志。能解释“数组长度属于类型、元组各位置可有不同类型、函数返回和打印不同”，再回自己的 UnitLab / QuizDeck / TextScope 使用这些概念。

## 参考实现的完整验证

在 **同一个根目录**执行；fmt / clippy 需要本机对应工具组件：

```powershell
cargo fmt --manifest-path projects/book-d6-foundations/Cargo.toml --all -- --check
cargo clippy --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml --workspace --all-targets -- -D warnings
cargo test --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml --workspace --all-targets
cargo test --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml --workspace --doc
cargo test --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml --workspace --all-targets --release
```

这些检查证明参考实现的行为，不自动记录学员掌握情况。[概念审计](../../course/BOOK_CONCEPT_AUDIT.md) 单独列教学目标；后续按当前作品的问题选所有权、集合或泛型专项，避免把补课材料当成新的必修主线。
