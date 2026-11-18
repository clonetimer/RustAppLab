# D6-B · Rust Book 基础知识缺口补齐

**范围**：原 D0 审计中的 12 项基础缺口，以及 1.3 的 Cargo 入门复习、3.2 的数组／元组专项；不据此声称完整覆盖原书。

**配套工程**：[D6 Foundations Gap Lab](../projects/book-d6-foundations/README.md)，Edition 2021 / Rust 1.74 / 零第三方依赖。

**边界**：它是独立参考 Workspace，不加入根 Workspace，也不增加原有 17 个学员作品。

本课不重新讲授整本 Rust Book，而是把仍缺少明确练习的知识点放到一个小型 CLI 与库中。每项实验都可以独立测试，再组合成一个可运行程序。

## 1. 缺口与证据映射

| 原书小节 | 补齐目标 | D6-B 实验 |
|---|---|---|
| 1.2 Hello World | 建立最小 crate、编译、运行和输出 | `foundation-lab hello` |
| 1.3 Hello, Cargo! | 辨认 package/crate/bin/lib、清单与锁文件，比较 check/build/run 和 debug/release | 本课 3.2 的三步练习 |
| 2 猜数字游戏 | 输入、解析、比较、循环、退出与无效输入 | `foundation-lab guess` |
| 3.2 数据类型：复合类型专项 | 混合类型元组、解构、单元与定长同类型数组 | `summarize_scores`、`compound_types` 集成测试及 doctest |
| 3.4 注释 | 普通注释、块注释、项文档和 crate 文档 | `lib.rs` 的 `///` / `//! ` |
| 5.2 结构体示例 | 用结构体表达矩形数据与不变量 | `Rectangle` |
| 5.3 方法语法 | `impl`、`self`、关联函数和方法调用 | `Rectangle::new` / `area` / `can_hold` |
| 6.3 if let | 只关心一个枚举/Option 分支时的简洁匹配 | `describe_optional_name` |
| 8.3 HashMap | 键值存储、Entry API 与计数 | `word_frequencies` |
| 9.1 panic! | 区分不可恢复的程序不变量与普通输入错误 | `require_non_empty` |
| 9.3 panic 还是 Result | 用返回类型表达可预期失败 | `parse_non_empty` / `parse_guess` |
| 12.5 环境变量 | 读取配置环境变量，并将环境依赖注入纯函数 | `config_dir_from` / `config` 子命令 |
| 13.3 迭代器改进 I/O | 通过 BufRead 与迭代器处理文件行，并传播 I/O 错误 | `count_non_empty_lines` |
| 13.4 循环与迭代器性能 | 相同输入、相同输出、release 模式下对照测量 | `bench` 子命令 |

## 2. 运行工程

在 RustAppLab 源码根目录执行：

~~~powershell
cargo fmt --manifest-path projects/book-d6-foundations/Cargo.toml --all -- --check
cargo clippy --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml --workspace --all-targets -- -D warnings
cargo test --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml --workspace --all-targets
cargo test --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml --workspace --doc
cargo test --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml --workspace --all-targets --release
~~~

试运行：

~~~powershell
cargo run --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab -- hello
cargo run --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab -- words "Rust rust Ownership"
cargo run --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab -- guess
cargo run --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab -- config
cargo run --release --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab -- bench 100000
~~~

## 3. 逐项学习

### 3.1 Hello World 与猜数字游戏（1.2、2）

先运行 `hello`。观察一个 Rust 二进制程序至少需要什么：crate、入口函数 `main`、输出宏和 Cargo 运行命令。

再运行 `guess`：
- 程序在 1 到 100 之间产生目标值。
- 使用 `stdin().read_line` 读取用户输入。
- 使用 `parse_guess` 将字符串转换成整数。
- 使用 `match` 根据大小关系给出反馈。
- 输入无效时继续，不把用户输入错误当成程序崩溃。
- 输入 `q` 或到达正确答案后退出。

本 lab 为了零依赖与离线测试，使用时间种子驱动的 xorshift 生成教学用目标值。它不是密码学安全随机数，也不替代原书使用 `rand` crate 的版本。重点是把输入、解析、比较和循环分开测试。

**练习**：将范围改为 1–500；增加“已经猜过”的提示；解释为什么输入 `abc` 应返回可恢复错误，而不是 panic。

### 3.2 Cargo 入门复习（1.3）

Cargo 管理 Rust 工程的构建与依赖。先在当前参考工程里对照术语，再运行命令；本练习不新增学员工程。原书对照：[1.3 Hello, Cargo!](https://doc.rust-lang.org/book/ch01-03-hello-cargo.html)；package 与 crate 的进一步定义见 [7.1 Packages and Crates](https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html)。

| 术语 | 含义 | 当前工程中的位置 |
|---|---|---|
| package（包） | Cargo 管理的工程单元，由 `Cargo.toml` 描述；可以包含多个 binary crate，最多一个 library crate，至少有一个 crate | `foundation-lab/Cargo.toml` 的 `[package]`；包名是 `book-d6-foundation-lab` |
| crate | Rust 编译器处理的编译单元；从一个 crate root 开始，内部可以有多个模块和源码文件 | 当前包的两个主要目标分别从 `src/lib.rs`、`src/main.rs` 开始 |
| binary crate / bin | 可编译成可执行程序，通常由 `main` 作为入口 | `src/main.rs`；清单中的 `[[bin]]` 将目标命名为 `foundation-lab` |
| library crate / lib | 提供供其他 Rust 代码调用的功能，不需要程序入口 `main` | `src/lib.rs`；代码中用 `book_d6_foundation_lab` 导入 |
| workspace | 将若干包放在一起管理构建、共享锁文件和产物目录 | `projects/book-d6-foundations/Cargo.toml` 只有 `[workspace]`，成员是 `foundation-lab`；它独立于仓库根 Workspace |

包名、二进制目标名、代码中的库名不必写成同一个字符串：这里 `-p book-d6-foundation-lab` 选择包，`--bin foundation-lab` 选择可执行目标，而 `use book_d6_foundation_lab::summarize_scores;` 导入库中的函数。不要把一个 `.rs` 文件等同于一个 package。

**清单与锁文件**：`Cargo.toml` 由开发者描述包信息、目标、依赖约束或 Workspace 配置；`Cargo.lock` 由 Cargo 记录实际解析的精确包版本。本工程没有第三方依赖，仍保留锁文件。不要手改锁文件；`--locked` 要求 Cargo 不改变它，`--offline` 要求这次操作不访问网络。包内清单声明 Edition 2021 和最低 Rust 1.74；即使在线原书的示例 edition 更新，也不要为本练习改动这些约束。

在 RustAppLab 源码根目录依次执行：

~~~powershell
cargo check --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab
cargo build --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab
cargo run --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab --bin foundation-lab -- hello
cargo run --release --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab --bin foundation-lab -- hello
~~~

| 命令或模式 | 要观察什么 |
|---|---|
| `cargo check` | 检查代码能否通过编译检查，但不生成可直接运行的可执行文件，也不运行 `hello` |
| `cargo build` | 构建可执行文件，不自动运行它 |
| `cargo run ... -- hello` | 必要时先构建，然后运行二进制；`--` 后的 `hello` 交给程序，预期输出 `Hello, world!` |
| 默认开发构建 | 使用 dev profile；本 Workspace 明确配置 `opt-level = 0`，通常利于快速迭代 |
| `--release` | 使用 release profile；本 Workspace 明确配置 `opt-level = 2`，优化程度较高，构建可能更慢；问候语仍应相同 |

没有配置 `CARGO_TARGET_DIR` 或其他产物目录覆盖时，可执行文件分别位于 `projects/book-d6-foundations/target/debug/foundation-lab` 和 `projects/book-d6-foundations/target/release/foundation-lab`；Windows 文件名带 `.exe`。目录名 `debug` 不代表另有一个叫 `debug` 的 Cargo profile。优化模式也不取消数组索引的边界检查。

**三步新手练习**：

1. **读结构**：打开 Workspace 和成员包的两个 `Cargo.toml`，指出 `[workspace]`、`[package]`、`[lib]`、`[[bin]]` 的职责，再指出两个主要 crate root 和锁文件的位置。写下三个名字：包名、二进制目标名、代码导入库名。
2. **跑命令**：按上表顺序运行 `check`、`build`、`run`，记录退出码和终端输出，解释为什么前两个命令没有打印问候语，以及 `-- hello` 中的参数交给了谁。
3. **辨认模式**：运行带 `--release` 的问候语命令，定位两种模式的产物目录，查看本 Workspace 的两个 `opt-level` 配置。说明为什么输出应相同、为什么一次 Hello World 计时不能证明优化效果。提交自己的命令记录和解释，不只提交“运行成功”。

### 3.3 数组与元组（3.2 的复合类型专项）

先用同一份“三次分数”数据区分两种复合类型。元组的位置可以有不同类型；数组的元素必须同类型，而且长度固定。两者都有固定的组成，但用途不同：`(&str, [u32; 3], bool)` 把姓名、三次分数和提交状态放在一起，`[u32; 3]` 则规定恰好三个 `u32`。原书对照：[3.2 Data Types · Compound Types](https://doc.rust-lang.org/book/ch03-02-data-types.html#compound-types)。

参考库只增加一个函数：`summarize_scores(scores: [u32; 3]) -> (u64, u32)`。它返回 `(总分, 最高分)`，不限制单次分数为 0–100。总分使用 `u64`，并在相加前转换每一项，因此三项都等于 `u32::MAX` 时，仍能在 Debug 和 Release 下得到精确总分 `12_884_901_885`。

~~~rust
use book_d6_foundation_lab::summarize_scores;

let entry: (&str, [u32; 3], bool) = ("Ada", [72, 95, 81], true);
let (name, scores, submitted) = entry; // 元组解构，为各位置命名
let summary = summarize_scores(scores);
assert_eq!(summary.0, 248); // .0 是第一个位置
assert_eq!(summary.1, 95);
let (total, highest) = summary;
assert_eq!((name, submitted, total, highest), ("Ada", true, 248, 95));
~~~

**单元 `()`** 是没有元素的元组；类型和值都写成 `()`。没有返回其他值的函数或代码块会得到单元值。下面的代码块最后是带分号的赋值语句，所以它的结果不是分数：

~~~rust
let mut scores: [u32; 3] = [0; 3]; // [类型; 长度] 与 [值; 长度] 要区分
let finished: () = {
    scores[0] = 90;
};
assert_eq!(finished, ());
assert_eq!(scores, [90, 0, 0]);
~~~

`[0; 3]` 用同一个 `u32` 值初始化三个元素；`mut` 允许修改元素，不会让数组变长。`[u32; 3]` 和 `[u32; 2]` 是不同类型，`[u32; 3]` 和 `[u64; 3]` 也不同。

**索引与越界**：数组从 `0` 开始索引，长度为三的数组只有 `0`、`1`、`2` 三个合法下标。`scores[index]` 在运行时发现越界会 panic；`scores.get(index)` 用 `Option<&u32>` 表示有值或没有值，越界返回 `None`。

~~~rust
let scores = [72_u32, 95, 81];
assert_eq!(scores[0], 72);
assert_eq!(scores.get(2), Some(&81));
assert_eq!(scores.get(3), None);
~~~

函数文档中的 `should_panic` 示例把下标作为函数参数传入，验证运行时越界；若直接写一个编译器能确定越界的常量索引，也可能在编译时就被拒绝。对来自用户输入的下标，应使用 `get` 并处理 `None`，不要依赖 panic 提供普通输入错误提示。

类型不匹配有两条独立的 `compile_fail` 文档测试：传入 `[72_u32, 95]` 会违反长度要求；把 `[72, 95.5, 81]` 赋给 `[u32; 3]` 会违反元素类型要求。它们都应在编译阶段失败，不是运行时错误。实际代码位于 `foundation-lab/src/lib.rs` 的 `summarize_scores` 文档中，可用以下命令检查：

~~~powershell
cargo test --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab --test compound_types
cargo test --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab --doc summarize_scores
~~~

**练习**：先手算 `[72, 95, 81]`、`[80; 3]` 和 `[0; 3]` 的结果，再对照测试；将最高分分别放在第一、第二、第三个位置；使用元组解构和 `.0` 各读取一次总分。随后在临时草稿中分别制造长度和元素类型错误，记录编译器指出的“期望类型／实际类型”，并各修复一次。最后解释为什么 `get(3)` 返回 `None`，而直接索引第三号位置会失败。不要仅以通过测试代替这些说明。

### 3.4 注释（3.4）

- `//`：普通行注释。
- `/* ... */`：块注释，可用于解释一段代码。
- `///`：紧邻项目的文档注释，会进入 rustdoc。
- `//!`：模块或 crate 级文档注释。

普通注释示例：

~~~rust
// A line comment explains the next statement.
/*
 * A block comment can span multiple lines.
 * Use it for a short explanation, not to hide large code sections.
 */
let answer = 42;
~~~

D6-B 的 crate 顶部使用 `//! `，公开类型与函数使用 `///`，并包含可运行 doctest。文档示例是代码合同，修改 API 时也要运行 `cargo test --doc`。

**练习**：为 `word_frequencies` 添加一个 doctest，证明输入大小写不同但词频键统一为小写。

### 3.5 结构体与方法（5.2、5.3）

`Rectangle` 用两个字段表示宽和高。方法放在 `impl Rectangle` 中：
- `new` 和 `square` 是关联函数，通过 `Rectangle::new(...)` 调用。
- `area` 和 `can_hold` 是方法，接收 `self`。
- `area` 返回 `u64`，避免两个 `u32` 相乘时发生溢出。

**练习**：添加 `perimeter` 方法；为零宽或零高定义业务语义；为每种边界情况写测试。

### 3.6 if let（6.3）

当只关心 `Some` 而不需要为 `None` 写完整 `match` 时，可以使用 `if let`。本例仍保留 `else` 的语义，说明“匹配一个分支”并不意味着忽略其他情况。

**练习**：将 `describe_optional_name` 改写为 `match`，比较两个版本的可读性；不要为了少写几行而丢失空字符串检查。

### 3.7 HashMap（8.3）

`word_frequencies` 使用 `HashMap<String, usize>`：
- `entry(key)` 获取或创建键的入口。
- `or_insert(0)` 提供初始计数。
- 对返回的可变引用递增计数。

HashMap 的遍历顺序不保证稳定，所以 CLI 在打印前显式排序。不能把某一次运行看到的键顺序当作接口合同。

**练习**：忽略标点、统计每行词频、找出最高频词；为并列最高频设计确定性的输出顺序。

### 3.8 panic 与 Result（9.1、9.3）

两种失败要分开：

| 场景 | 处理方式 | 示例 |
|---|---|---|
| 外部输入不合法、文件不可读、配置缺失 | 返回 `Result`，让调用者决定恢复或提示 | `parse_non_empty`、`parse_guess` |
| 程序内部不变量被破坏，继续运行没有意义 | 使用 `panic!` 或断言暴露程序错误 | `require_non_empty` |

不要用 panic 处理普通用户输入；也不要把所有错误都吞成默认值。对于可预期失败，返回错误类型能保留上下文，并让 CLI 选择重试、退出或提示。

**练习**：将 `require_non_empty` 的调用放在 CLI 的用户输入路径中，观察为什么设计不合理；再改用 `parse_non_empty` 并增加错误提示。

### 3.9 环境变量（12.5）

`config` 子命令优先读取 `RUSTAPPLAB_HOME`，再尝试 `HOME` 和 Windows 的 `USERPROFILE`。核心函数 `config_dir_from` 接收 `Option<&OsStr>`，不直接读取进程环境，因此测试不必修改全局环境变量，也不会互相污染。

PowerShell 示例：

~~~powershell
$env:RUSTAPPLAB_HOME = "$HOME\RustAppLabData"
cargo run --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab -- config
~~~

**练习**：添加 `RUSTAPPLAB_CONFIG_FILE` 覆盖配置文件路径；定义优先级并测试“变量缺失、空值、路径含空格”三种情况。

### 3.10 迭代器改进 I/O（13.3）

`count_non_empty_lines` 使用 `BufRead::lines` 读取行，并通过 `try_fold` 累计结果。每一行的 I/O 错误通过 `?` 向上传播，不会被当成空行或静默忽略。

**练习**：实现一个 TSV 汇总器：逐行解析、跳过空行、统计有效记录，并在遇到损坏记录时返回带行号的错误。

### 3.11 循环与迭代器性能（13.4）

`sum_with_loop` 与 `sum_with_iterator` 对同一切片完成相同的求和。基准函数使用 `Instant` 和 `std::hint::black_box`，减少优化器消除工作量的可能。

比较时必须：
1. 使用同一份输入和相同重复次数。
2. 先验证两种实现的结果完全一致。
3. 使用 `cargo run --release`，不要用 Debug 时间得出性能结论。
4. 重复多轮，报告中位数或分布，而不是只挑一次最快结果。
5. 不写“迭代器一定更快”或“循环一定更快”的结论。

**练习**：将求和改为过滤偶数后求和；改变数据规模和重复次数；记录 Debug/Release 的差异。性能结果受编译器、CPU、缓存、数据规模和测量噪声影响。

## 4. 完成标准

- 能独立运行 Hello World 与猜数字游戏，并解释输入解析和循环退出。
- 能在当前工程中指出 package、crate、bin、lib、两个清单和锁文件，解释 `check`、`build`、`run` 及 Debug/Release 的区别。
- 能用混合元组、解构、位置访问、单元值和定长数组表示三次分数，解释长度／类型错误与索引越界的区别。
- 能定义结构体、关联函数和方法，写出边界测试。
- 能正确选择 `if let`、`match`、`HashMap::entry`。
- 能解释哪些错误应返回 `Result`，哪些属于程序不变量。
- 能读取环境变量但不让测试依赖全局环境状态。
- 能使用迭代器处理 I/O 并传播错误。
- 能设计公平的 loop/iterator 基准，不用单次计时声称性能优劣。

历史基线 Run 215 的验收范围是原有 12 项基础缺口：14 项单元测试、6 项 CLI 集成测试和 2 项 Doctest，在 Linux、Windows 与实际 Rust 1.74 工具链下通过，Debug/Release 结果一致。该历史结果不包含本次新增的 Cargo 复习与数组／元组练习；当前代码的验证应重新运行本课命令并查看本次日志。

通过构建或测试只证明参考实现满足这些检查，不证明原书已全覆盖，也不证明学习者已经掌握。学员掌握情况保持“未评估”，需提交自己的输入、代码修改和设计解释后再评定。

接着学习：[D6 变量与模块专项](BOOK_D6_LANGUAGE_BASICS.md)，对应 3.1 和 7.2–7.5；它使用同一参考 Workspace，不增加学员工程。
