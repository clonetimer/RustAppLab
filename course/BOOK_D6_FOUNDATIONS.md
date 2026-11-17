# D6-B · Rust Book 基础知识缺口补齐

**范围**：补齐 D0 审计矩阵中仍为部分覆盖或缺口的 12 个基础小节。  
**配套工程**：[D6 Foundations Gap Lab](../projects/book-d6-foundations/README.md)，Edition 2021 / Rust 1.74 / 零第三方依赖。  
**边界**：它是独立参考 Workspace，不加入根 Workspace，也不增加原有 17 个学员作品。

本课不重新讲授整本 Rust Book，而是把仍缺少明确练习的知识点放到一个小型 CLI 与库中。每项实验都可以独立测试，再组合成一个可运行程序。

## 1. 缺口与证据映射

| 原书小节 | 补齐目标 | D6-B 实验 |
|---|---|---|
| 1.2 Hello World | 建立最小 crate、编译、运行和输出 | `foundation-lab hello` |
| 2 猜数字游戏 | 输入、解析、比较、循环、退出与无效输入 | `foundation-lab guess` |
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

### 3.2 注释（3.4）

- `//`：普通行注释。
- `/* ... */`：块注释，可用于解释一段代码。
- `///`：紧邻项目的文档注释，会进入 rustdoc。
- `//!`：模块或 crate 级文档注释。

D6-B 的 crate 顶部使用 `//! `，公开类型与函数使用 `///`，并包含可运行 doctest。文档示例是代码合同，修改 API 时也要运行 `cargo test --doc`。

**练习**：为 `word_frequencies` 添加一个 doctest，证明输入大小写不同但词频键统一为小写。

### 3.3 结构体与方法（5.2、5.3）

`Rectangle` 用两个字段表示宽和高。方法放在 `impl Rectangle` 中：
- `new` 和 `square` 是关联函数，通过 `Rectangle::new(...)` 调用。
- `area` 和 `can_hold` 是方法，接收 `self`。
- `area` 返回 `u64`，避免两个 `u32` 相乘时发生溢出。

**练习**：添加 `perimeter` 方法；为零宽或零高定义业务语义；为每种边界情况写测试。

### 3.4 if let（6.3）

当只关心 `Some` 而不需要为 `None` 写完整 `match` 时，可以使用 `if let`。本例仍保留 `else` 的语义，说明“匹配一个分支”并不意味着忽略其他情况。

**练习**：将 `describe_optional_name` 改写为 `match`，比较两个版本的可读性；不要为了少写几行而丢失空字符串检查。

### 3.5 HashMap（8.3）

`word_frequencies` 使用 `HashMap<String, usize>`：
- `entry(key)` 获取或创建键的入口。
- `or_insert(0)` 提供初始计数。
- 对返回的可变引用递增计数。

HashMap 的遍历顺序不保证稳定，所以 CLI 在打印前显式排序。不能把某一次运行看到的键顺序当作接口合同。

**练习**：忽略标点、统计每行词频、找出最高频词；为并列最高频设计确定性的输出顺序。

### 3.6 panic 与 Result（9.1、9.3）

两种失败要分开：

| 场景 | 处理方式 | 示例 |
|---|---|---|
| 外部输入不合法、文件不可读、配置缺失 | 返回 `Result`，让调用者决定恢复或提示 | `parse_non_empty`、`parse_guess` |
| 程序内部不变量被破坏，继续运行没有意义 | 使用 `panic!` 或断言暴露程序错误 | `require_non_empty` |

不要用 panic 处理普通用户输入；也不要把所有错误都吞成默认值。对于可预期失败，返回错误类型能保留上下文，并让 CLI 选择重试、退出或提示。

**练习**：将 `require_non_empty` 的调用放在 CLI 的用户输入路径中，观察为什么设计不合理；再改用 `parse_non_empty` 并增加错误提示。

### 3.7 环境变量（12.5）

`config` 子命令优先读取 `RUSTAPPLAB_HOME`，再尝试 `HOME` 和 Windows 的 `USERPROFILE`。核心函数 `config_dir_from` 接收 `Option<&OsStr>`，不直接读取进程环境，因此测试不必修改全局环境变量，也不会互相污染。

PowerShell 示例：

~~~powershell
$env:RUSTAPPLAB_HOME = "$HOME\RustAppLabData"
cargo run --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab -- config
~~~

**练习**：添加 `RUSTAPPLAB_CONFIG_FILE` 覆盖配置文件路径；定义优先级并测试“变量缺失、空值、路径含空格”三种情况。

### 3.8 迭代器改进 I/O（13.3）

`count_non_empty_lines` 使用 `BufRead::lines` 读取行，并通过 `try_fold` 累计结果。每一行的 I/O 错误通过 `?` 向上传播，不会被当成空行或静默忽略。

**练习**：实现一个 TSV 汇总器：逐行解析、跳过空行、统计有效记录，并在遇到损坏记录时返回带行号的错误。

### 3.9 循环与迭代器性能（13.4）

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
- 能定义结构体、关联函数和方法，写出边界测试。
- 能正确选择 `if let`、`match`、`HashMap::entry`。
- 能解释哪些错误应返回 `Result`，哪些属于程序不变量。
- 能读取环境变量但不让测试依赖全局环境状态。
- 能使用迭代器处理 I/O 并传播错误。
- 能设计公平的 loop/iterator 基准，不用单次计时声称性能优劣。

本阶段覆盖审计中的 12 个基础缺口；通过 CI 只证明参考实现可编译、可测试，不证明学习者已经掌握。最终掌握仍需完成自己的输入、修改代码并解释设计理由。
