# D6 · 变量与模块：把学习计划卡拆成可验证的小程序

本课补充原书 **3.1 与 7.2–7.5** 的具体练习。先完成 FirstCard、UnitLab，能够运行一个 Cargo 工程，再做本课。配套代码属于 [D6 Foundations](../projects/book-d6-foundations/README.md)，使用 Edition 2021、最低 Rust 1.74，不需要第三方依赖。

你会用同一组学习计划卡完成两件事：解析每张卡的学习时长；把卡片类型拆进一个模块，同时保留清晰的公开入口。练习顺序始终是 **预测 → 跑测试 → 修改个人副本 → 解释**。

## 1. 找到代码与测试

以下路径以 `projects/book-d6-foundations/foundation-lab/` 为起点：

| 文件 | 作用 |
|---|---|
| `src/lib.rs` | crate 根，用 `pub mod language_basics;` 声明公开模块 |
| `src/language_basics.rs` | 常量、输入解析、累计时长、公开重导出与文档实验 |
| `src/language_basics/catalog.rs` | 私有子模块，定义 `StudyCard` 和公开枚举 `Activity` |
| `tests/language_basics.rs` | 从库外部验证输入、卡片输出和累计时长 |

先阅读代码，不急着改。需要修改时，将整个 `projects/book-d6-foundations` 目录复制到自己的新练习目录，保留 `Cargo.toml`、`Cargo.lock`、源码与测试。下列命令从 RustAppLab 源码根目录运行；使用个人副本时，将 `--manifest-path` 后的路径改为副本的 `Cargo.toml`。

```powershell
cargo test --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab --test language_basics
cargo test --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab --doc language_basics
```

第一条运行 5 个功能测试；第二条运行此模块的 7 个文档实验，其中 2 个应编译运行成功，5 个标记为 `compile_fail`，应被编译器拒绝。`compile_fail` 显示 `ok` 的含义是“这段错误用法确实不能编译”。不要删除这些反例来让输出变绿。

## 2. 变量、常量、可变性与遮蔽（3.1）

### 先预测

打开 `language_basics.rs`，找出 `MAX_MINUTES`、`parse_minutes` 和 `total_minutes`。回答后再运行：

1. `parse_minutes(" 25 ")`、`parse_minutes("0")`、`parse_minutes("121")` 各得到什么？
2. `parse_minutes` 里同名的 `input` 先后是什么类型？
3. 三张卡分别是 25、40、5 分钟，`total_minutes` 应返回多少？空计划呢？
4. 去掉 `let mut total` 中的 `mut`，还能执行 `total += ...` 吗？

### 跑测试，再对照代码

`MAX_MINUTES: u32 = 120` 是常量，使用 `const` 并显式标注类型。它约束每张卡允许的时长；常量不能被重新赋值。

`parse_minutes` 先用 `trim` 得到去除首尾空白的 `&str`，再用 `let input: u32 = ...` 把解析结果绑定给一个同名的新变量。这叫**遮蔽（shadowing）**。前后的变量名称相同，类型可以不同。`let mut input = "25"; input = 25_u32;` 则是给同一个变量赋不同类型的值，编译器会拒绝。

`total_minutes` 使用 `mut` 让同一个 `u64` 变量在循环中累加。这里需要改变值，类型始终是 `u64`。它读取卡片的时长，不修改卡片。

### 修改个人副本

先在 `total_minutes` 中去掉 `mut`，运行测试，保存第一条编译错误。补回 `mut` 后再次运行。接着把个人测试中的第三张卡改成 15 分钟，先手算新的总时长，再修改该条测试的预期值并运行。保留其他边界测试。

### 用自己的话解释

写出 `const`、普通 `let`、`let mut`、同名 `let` 各自表达什么。解释为什么“两个不同输入产生不同输出”不能单独证明你已经理解可变性；程序重新运行时获得新输入，与一次运行中给同一变量重新赋值是两件事。

## 3. 模块路径与导入（7.2、7.3、7.4）

### 先预测

`StudyCard` 定义在 `catalog.rs` 中。库外部代码能使用下面哪个路径？先记录答案，再运行文档测试。

```rust
use book_d6_foundation_lab::language_basics::StudyCard;
use book_d6_foundation_lab::language_basics::catalog::StudyCard;
```

### 跑测试，再找每个路径的起点

| 代码中的路径或语句 | 从哪里开始、做什么 |
|---|---|
| `crate::language_basics::catalog::StudyCard` | 从当前 crate 根开始的绝对路径 |
| `self::catalog` | 从当前 `language_basics` 模块开始的相对路径 |
| `super::parse_minutes` | 从 `catalog` 的父模块 `language_basics` 开始 |
| `use ...::StudyCard as Card` | 将同一类型以本地名称 `Card` 引入作用域 |
| `pub use self::catalog::{Activity, StudyCard};` | 通过父模块公开这两个类型的访问入口 |

源码用 `crate::` 路径导入 `Card`，用 `self::` 重导出类型，子模块通过 `super::parse_minutes` 复用输入校验。集成测试作为库的使用者，以 crate 名 `book_d6_foundation_lab` 开头导入公开 API。

`pub use` 让外部代码能通过 `language_basics::StudyCard` 使用类型；`catalog` 模块本身仍是私有的，直接经过它的外部路径会被拒绝。`use` 只引入名称；`as` 只改变本地名称，不会创建一个新类型。

### 修改个人副本

在集成测试中把本地别名 `Card` 改成 `PlanCard`，同步修改该测试里的调用，再运行测试。然后暂时移除模块里的 `pub use`，观察外部导入为什么失败；恢复重导出并确认测试重新通过。

### 用自己的话解释

指出 `crate`、`self`、`super` 分别从哪里寻找名称。说明 Cargo.toml 中的 `path = "../other-crate"` 管理的是另一个包的依赖位置，而这里的 `::` 路径引用的是 Rust 模块树中的项。

## 4. 私有字段、公开枚举与拆文件（7.2、7.3、7.5）

### 先预测

`StudyCard` 的类型是公开的，但 `title`、`minutes`、`activity` 字段没有 `pub`。外部调用者能否直接写 `card.minutes`？为什么 `card.minutes()` 可以调用？`Activity::Read`、`Activity::Practice` 能否从库外部使用？

### 跑测试，再解释边界

`StudyCard::new` 拒绝空标题和不在 1–120 范围内的时长。私有字段让外部调用者通过这个入口创建卡片；公开的 `minutes()` 方法提供读取。公开结构体不会自动公开字段，公开枚举的变体则可以被外部使用。`summary` 会根据 `Read` 或 `Practice` 生成不同输出，测试验证两条分支。

`language_basics.rs` 中的 `mod catalog;` 声明子模块，并让编译器加载 `language_basics/catalog.rs`。两个文件属于同一个 crate。拆文件是代码组织方式；文件移动以后，只要模块结构和公开入口保持一致，调用者仍可使用相同的 API。

### 修改个人副本

临时给 `minutes` 字段加上 `pub`，再运行文档测试。原来访问字段的 `compile_fail` 反例会编译成功，因此这项测试反而失败：它在提示公开边界发生了变化。恢复字段私有性并重跑。

再暂时注释 `mod catalog;`，观察为什么只保留 `use` 无法加载这个模块。恢复声明，不需要新建 package 或修改 Cargo 依赖。

### 用自己的话解释

解释私有字段保护了哪个输入约束，为什么类型公开不代表所有内部实现都公开；说明 `mod`、`use`、`pub use` 的区别，并在两个源码文件中指出它们实际起作用的位置。

## 5. 保存自己的完成证据

完成实验后运行整个参考 Workspace 的检查，确保个人修改没有破坏其他功能或文档示例：

```powershell
cargo fmt --manifest-path projects/book-d6-foundations/Cargo.toml --all -- --check
cargo clippy --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml --workspace --all-targets -- -D warnings
cargo test --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml --workspace --all-targets
cargo test --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml --workspace --doc
```

在自己的学习记录中保留：预测与实际结果、一次预期编译失败及修复、修改后的源码、一组个人输入，以及关于遮蔽和公开入口的解释。测试验证这些输入和编译边界；个人掌握仍需依据独立修改与解释判断。本课只补充列明的知识点，不代表原书全部小节已完整覆盖。

## 原书对照

- [3.1 变量与可变性](https://rust.bootcss.com/ch03-01-variables-and-mutability.html)：不可变重赋值、`mut`、常量与遮蔽。
- [7.2 模块作用域与私有性](https://rust.bootcss.com/ch07-02-defining-modules-to-control-scope-and-privacy.html)：crate 内的模块树。
- [7.3 路径用于引用模块树中的项](https://rust.bootcss.com/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html)：绝对／相对路径、`super` 与公开边界。
- [7.4 use 关键字](https://rust.bootcss.com/ch07-04-bringing-paths-into-scope-with-the-use-keyword.html)：导入、`as` 别名与 `pub use` 重导出。
- [7.5 将模块分割进不同文件](https://rust.bootcss.com/ch07-05-separating-modules-into-different-files.html)：`mod name;` 与独立模块文件。

原书站点采用 Edition 2018；本实验保留课程的 Edition 2021／Rust 1.74 基线。
