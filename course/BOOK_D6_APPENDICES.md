# D6-A · Rust Book 附录 A–G 参考与实践

**基准版本**：本课对照 [rust.bootcss.com 中文版](https://rust.bootcss.com/)，其书稿基于 Rust 1.37.0 / Edition 2018。  
**当前工程基线**：Rust AppLab 采用 Edition 2021；D6 参考实验最低 Rust 1.74。  
**当前语法核查**：关键字与语法以 [Rust Reference](https://doc.rust-lang.org/reference/) 和当前 [官方 Rust Book](https://doc.rust-lang.org/book/) 为准。

附录是参考资料，不应把它们变成需要死记硬背的词表。本课的目标是：遇到一个语法符号、派生 trait、开发工具或 Edition 差异时，能够判断它的语义、找到权威资料，并用最小实验验证。

配套 [D6 Appendix Reference Lab](../projects/book-d6-appendices/README.md) 是独立 Workspace，不加入根 Workspace，也不增加第 18 个学员作品。

---

## A · 关键字（Keywords）

关键字在特定语法位置具有语言定义的含义，不能像普通标识符一样随意命名。当前 Rust Reference 将关键字分为三类：

| 类别 | 含义 | 代表项 |
|---|---|---|
| Strict keywords | 在语言规定的语法位置有固定含义，不能作为普通标识符 | `fn`、`let`、`match`、`struct`、`trait`、`async`、`await` |
| Reserved keywords | 暂未普遍启用，但为未来语法保留 | `abstract`、`become`、`box`、`gen`、`try`、`yield` |
| Weak keywords | 只在特定语法上下文具有关键字含义 | `union`、`macro_rules`、`raw`、`safe`、`'static` |

### 版本差异不能忽略

- Edition 2018 将 `async`、`await`、`dyn` 纳入 strict keywords。
- Edition 2018 将 `try` 加入 reserved keywords。
- Edition 2024 将 `gen` 加入 reserved keywords。
- 关键字表会随语言版本演进；旧中文版的表不能直接当作 2026 年工具链的完整清单。

### 当前 Edition 的完整关键词清单

下面按当前 Rust Reference 的词法分类列出完整集合。Edition 2015 的解析规则有历史差异；跨 Edition 迁移时仍要查看 Edition Guide，不能把当前清单倒推到旧 Edition。

| 类别 | 完整清单 |
|---|---|
| Strict | `_`, `as`, `async`, `await`, `break`, `const`, `continue`, `crate`, `dyn`, `else`, `enum`, `extern`, `false`, `fn`, `for`, `if`, `impl`, `in`, `let`, `loop`, `match`, `mod`, `move`, `mut`, `pub`, `ref`, `return`, `self`, `Self`, `static`, `struct`, `super`, `trait`, `true`, `type`, `unsafe`, `use`, `where`, `while` |
| Reserved | `abstract`, `become`, `box`, `do`, `final`, `gen`, `macro`, `override`, `priv`, `try`, `typeof`, `unsized`, `virtual`, `yield` |
| Weak | `'static`, `macro_rules`, `raw`, `safe`, `union` |

**Edition 注意事项：**
- `async`、`await`、`dyn` 在 Edition 2018 成为 strict keywords；Edition 2015 对 `dyn` 有上下文相关的旧规则。
- `try` 在 Edition 2018 加入 reserved keywords。
- `gen` 在 Edition 2024 加入 reserved keywords。
- `safe` 只在外部块的函数/静态项声明中具有弱关键字语义；`raw` 只在 raw borrow 等特定语法中具有该语义。
- `'static` 是生命周期标记，不可作为普通生命周期参数名；`union` 只在 union 声明上下文中是关键字。

### Raw identifiers

当外部 API 或旧 Edition crate 暴露了一个与新关键字同名的函数时，可以使用原始标识符：

```rust
fn r#match(text: &str, needle: &str) -> bool {
    text.contains(needle)
}

assert!(r#match("learn rust", "rust"));
```

`r#` 是源码语法前缀，不属于标识符本身。它不能用于所有特殊路径关键字；例如 `r#self`、`r#Self`、`r#super`、`r#crate` 和 `r#_` 都不是合法替代写法。

**动手实验**：运行 D6 lab 的 `raw_identifier_allows_a_keyword_as_a_function_name` 测试；再阅读源码中的两个 `compile_fail` doctest，观察哪些特殊名称不能被 raw identifier 转义。

参考：[中文附录 A](https://rust.bootcss.com/appendix-01-keywords.html) · [Rust Reference: Keywords](https://doc.rust-lang.org/reference/keywords.html) · [Rust Reference: Identifiers](https://doc.rust-lang.org/reference/identifiers.html)

---

## B · 运算符与符号（Operators and Symbols）

同一个符号可能在不同语境承担不同职责。阅读代码时，先判断它处于表达式、类型、模式、路径、宏还是属性上下文，再解释语义。

### 常见运算符

| 类别 | 符号 | 主要语义 | 可重载 trait（适用时） |
|---|---|---|---|
| 算术 | `+`、`-`、`*`、`/`、`%` | 加、减、乘、除、余数 | `Add`、`Sub`、`Mul`、`Div`、`Rem` |
| 比较 | `==`、`!=` | 相等/不等 | `PartialEq` |
| 比较 | `<`、`<=`、`>`、`>=` | 次序比较 | `PartialOrd` |
| 逻辑 | `!`、`&&`、`||` | 逻辑非、短路与、短路或 | `Not` 适用于一元 `!` |
| 位运算 | `&`、`|`、`^`、`<<`、`>>` | 按位与/或/异或、移位 | `BitAnd`、`BitOr`、`BitXor`、`Shl`、`Shr` |
| 复合赋值 | `+=`、`-=`、`*=` 等 | 运算并写回左操作数 | 对应的 `*Assign` trait |
| 借用/解引用 | `&`、`&mut`、`*` | 创建引用、可变引用、解引用 | 不是普通算术重载 |
| 类型转换 | `as` | 显式类型转换 | 语言语法 |
| 错误传播 | `?` | 成功时取出值，失败时提前返回 | 由 `Try` 相关语言机制支持；日常代码直接使用即可 |
| 范围 | `..`、`..=` | 右端排除/包含范围 | 构造 Range 类型 |

### 高频“同符号、不同语义”

| 符号 | 语境一 | 语境二 |
|---|---|---|
| `&` | `&value`：借用 | `a & b`：按位与 |
| `*` | `*reference`：解引用 | `a * b`：乘法 |
| `!` | `!condition`：逻辑非 | `macro!(...)`：宏调用标记 |
| `|` | `left | right`：按位或 | `|x| x + 1`：闭包参数分隔符；模式中也表示或模式 |
| `..` | `0..10`：右端排除的范围 | 结构体更新、模式中也可能出现 |
| `::` | `module::item`：路径分隔 | `::<T>`：显式指定泛型参数（turbofish） |
| `?` | `read_config()?`：错误传播 | 宏 matcher 和 trait bound 中也有其他语法用途 |

### 其他常见符号速查

| 符号 | 主要用途 |
|---|---|
| `::` | 路径分隔、关联项访问、显式泛型参数路径 |
| `->` | 函数/闭包返回类型 |
| `=>` | `match` 分支分隔符、宏规则转写符 |
| `@` | 模式绑定，同时匹配整个值与内部结构 |
| `?` | `Option`/`Result` 错误或空值传播 |
| `#` / `#!` | 外部属性 / crate 或模块内部属性 |
| `# D6-A · Rust Book 附录 A–G 参考与实践

**基准版本**：本课对照 [rust.bootcss.com 中文版](https://rust.bootcss.com/)，其书稿基于 Rust 1.37.0 / Edition 2018。  
**当前工程基线**：Rust AppLab 采用 Edition 2021；D6 参考实验最低 Rust 1.74。  
**当前语法核查**：关键字与语法以 [Rust Reference](https://doc.rust-lang.org/reference/) 和当前 [官方 Rust Book](https://doc.rust-lang.org/book/) 为准。

附录是参考资料，不应把它们变成需要死记硬背的词表。本课的目标是：遇到一个语法符号、派生 trait、开发工具或 Edition 差异时，能够判断它的语义、找到权威资料，并用最小实验验证。

配套 [D6 Appendix Reference Lab](../projects/book-d6-appendices/README.md) 是独立 Workspace，不加入根 Workspace，也不增加第 18 个学员作品。

---

## A · 关键字（Keywords）

关键字在特定语法位置具有语言定义的含义，不能像普通标识符一样随意命名。当前 Rust Reference 将关键字分为三类：

| 类别 | 含义 | 代表项 |
|---|---|---|
| Strict keywords | 在语言规定的语法位置有固定含义，不能作为普通标识符 | `fn`、`let`、`match`、`struct`、`trait`、`async`、`await` |
| Reserved keywords | 暂未普遍启用，但为未来语法保留 | `abstract`、`become`、`box`、`gen`、`try`、`yield` |
| Weak keywords | 只在特定语法上下文具有关键字含义 | `union`、`macro_rules`、`raw`、`safe`、`'static` |

### 版本差异不能忽略

- Edition 2018 将 `async`、`await`、`dyn` 纳入 strict keywords。
- Edition 2018 将 `try` 加入 reserved keywords。
- Edition 2024 将 `gen` 加入 reserved keywords。
- 关键字表会随语言版本演进；旧中文版的表不能直接当作 2026 年工具链的完整清单。

### 当前 Edition 的完整关键词清单

下面按当前 Rust Reference 的词法分类列出完整集合。Edition 2015 的解析规则有历史差异；跨 Edition 迁移时仍要查看 Edition Guide，不能把当前清单倒推到旧 Edition。

| 类别 | 完整清单 |
|---|---|
| Strict | `_`, `as`, `async`, `await`, `break`, `const`, `continue`, `crate`, `dyn`, `else`, `enum`, `extern`, `false`, `fn`, `for`, `if`, `impl`, `in`, `let`, `loop`, `match`, `mod`, `move`, `mut`, `pub`, `ref`, `return`, `self`, `Self`, `static`, `struct`, `super`, `trait`, `true`, `type`, `unsafe`, `use`, `where`, `while` |
| Reserved | `abstract`, `become`, `box`, `do`, `final`, `gen`, `macro`, `override`, `priv`, `try`, `typeof`, `unsized`, `virtual`, `yield` |
| Weak | `'static`, `macro_rules`, `raw`, `safe`, `union` |

**Edition 注意事项：**
- `async`、`await`、`dyn` 在 Edition 2018 成为 strict keywords；Edition 2015 对 `dyn` 有上下文相关的旧规则。
- `try` 在 Edition 2018 加入 reserved keywords。
- `gen` 在 Edition 2024 加入 reserved keywords。
- `safe` 只在外部块的函数/静态项声明中具有弱关键字语义；`raw` 只在 raw borrow 等特定语法中具有该语义。
- `'static` 是生命周期标记，不可作为普通生命周期参数名；`union` 只在 union 声明上下文中是关键字。

### Raw identifiers

当外部 API 或旧 Edition crate 暴露了一个与新关键字同名的函数时，可以使用原始标识符：

```rust
fn r#match(text: &str, needle: &str) -> bool {
    text.contains(needle)
}

assert!(r#match("learn rust", "rust"));
```

`r#` 是源码语法前缀，不属于标识符本身。它不能用于所有特殊路径关键字；例如 `r#self`、`r#Self`、`r#super`、`r#crate` 和 `r#_` 都不是合法替代写法。

**动手实验**：运行 D6 lab 的 `raw_identifier_allows_a_keyword_as_a_function_name` 测试；再阅读源码中的两个 `compile_fail` doctest，观察哪些特殊名称不能被 raw identifier 转义。

参考：[中文附录 A](https://rust.bootcss.com/appendix-01-keywords.html) · [Rust Reference: Keywords](https://doc.rust-lang.org/reference/keywords.html) · [Rust Reference: Identifiers](https://doc.rust-lang.org/reference/identifiers.html)

---

## B · 运算符与符号（Operators and Symbols）

同一个符号可能在不同语境承担不同职责。阅读代码时，先判断它处于表达式、类型、模式、路径、宏还是属性上下文，再解释语义。

### 常见运算符

| 类别 | 符号 | 主要语义 | 可重载 trait（适用时） |
|---|---|---|---|
| 算术 | `+`、`-`、`*`、`/`、`%` | 加、减、乘、除、余数 | `Add`、`Sub`、`Mul`、`Div`、`Rem` |
| 比较 | `==`、`!=` | 相等/不等 | `PartialEq` |
| 比较 | `<`、`<=`、`>`、`>=` | 次序比较 | `PartialOrd` |
| 逻辑 | `!`、`&&`、`||` | 逻辑非、短路与、短路或 | `Not` 适用于一元 `!` |
| 位运算 | `&`、`|`、`^`、`<<`、`>>` | 按位与/或/异或、移位 | `BitAnd`、`BitOr`、`BitXor`、`Shl`、`Shr` |
| 复合赋值 | `+=`、`-=`、`*=` 等 | 运算并写回左操作数 | 对应的 `*Assign` trait |
| 借用/解引用 | `&`、`&mut`、`*` | 创建引用、可变引用、解引用 | 不是普通算术重载 |
| 类型转换 | `as` | 显式类型转换 | 语言语法 |
| 错误传播 | `?` | 成功时取出值，失败时提前返回 | 由 `Try` 相关语言机制支持；日常代码直接使用即可 |
| 范围 | `..`、`..=` | 右端排除/包含范围 | 构造 Range 类型 |

### 高频“同符号、不同语义”

| 符号 | 语境一 | 语境二 |
|---|---|---|
| `&` | `&value`：借用 | `a & b`：按位与 |
| `*` | `*reference`：解引用 | `a * b`：乘法 |
| `!` | `!condition`：逻辑非 | `macro!(...)`：宏调用标记 |
| `|` | `left | right`：按位或 | `|x| x + 1`：闭包参数分隔符；模式中也表示或模式 |
| `..` | `0..10`：右端排除的范围 | 结构体更新、模式中也可能出现 |
| `::` | `module::item`：路径分隔 | `::<T>`：显式指定泛型参数（turbofish） |
| `?` | `read_config()?`：错误传播 | 宏 matcher 和 trait bound 中也有其他语法用途 |

 | 声明宏中的元变量与重复匹配 |
| `_` | 忽略值、类型推断占位、模式通配符 |
| `.` | 字段访问、方法调用、元组索引 |
| `()` | 分组、函数调用、元组与 unit 类型 |
| `[]` | 数组/切片类型、索引、数组表达式 |
| `{}` | 代码块、结构体表达式、宏分组 |
| `,` / `;` | 项目、参数、字段、语句分隔；二者不能任意互换 |

同一字符在不同语法位置可能完全不同。例如 `|x| x + 1` 中的竖线是闭包参数边界；`a | b` 是位或；`Some(x) | None` 是 or-pattern。判断符号含义时先确定语法上下文。

### 运算符优先级

以下顺序按当前 Rust Reference 从高到低排列。相同优先级的二元运算符按表中结合性分组；比较链和范围表达式在需要组合时必须显式使用括号。

| 优先级（高→低） | 表达式/运算符 | 结合性或约束 |
|---|---|---|
| 1 | 路径（Paths） | — |
| 2 | 方法调用（Method calls） | — |
| 3 | 字段访问（Field expressions） | 从左到右 |
| 4 | 函数调用、数组索引 | — |
| 5 | `?` 错误传播 | — |
| 6 | 一元 `-`、`!`、`*`、借用 `&` / `&mut` | — |
| 7 | `as` 类型转换 | 从左到右 |
| 8 | `*`、`/`、`%` | 从左到右 |
| 9 | `+`、`-` | 从左到右 |
| 10 | `<<`、`>>` | 从左到右 |
| 11 | 位与 `&` | 从左到右 |
| 12 | 位异或 `^` | 从左到右 |
| 13 | 位或 `|` | 从左到右 |
| 14 | `==`、`!=`、`<`、`>`、`<=`、`>=` | 比较链必须加括号 |
| 15 | 逻辑与 `&&` | 从左到右，短路求值 |
| 16 | 逻辑或 `||` | 从左到右，短路求值 |
| 17 | 范围 `..`、`..=` | 组合时必须加括号 |
| 18 | 赋值及复合赋值（`=`、`+=`、`-=` 等） | 从右到左 |
| 19 | `return`、`break`、闭包表达式 | — |

```rust
let value = 2 + 3 * 4;
assert_eq!(value, 14);

let explicit = (2 + 3) * 4;
assert_eq!(explicit, 20);

// Comparison chains require parentheses.
let a = 1;
let b = 2;
let c = 3;
assert!((a < b) == (b < c));
```

**动手实验**：运行 D6 lab 中的 precedence、range、bitwise 和 `Point + Point` 测试。注意 `+` 对自定义类型可以通过 `std::ops::Add` 重载；但借用、路径和控制流语法不能被当作普通算术运算符处理。对复杂表达式优先添加括号，而不是依赖读者记忆优先级。

参考：[中文附录 B](https://rust.bootcss.com/appendix-02-operators.html) · [Rust Reference: Expression precedence](https://doc.rust-lang.org/reference/expressions.html) · [Operator expressions](https://doc.rust-lang.org/reference/expressions/operator-expr.html)

---

## C · 可派生的 Trait（Derivable Traits）

`#[derive(...)]` 为类型生成常见 trait 的默认实现。它减少样板代码，但不会替开发者决定所有业务语义。

| Trait | 派生后获得的能力 | 使用条件与注意事项 |
|---|---|---|
| `Debug` | `{:?}` / `{:#?}` 调试输出 | 字段也必须实现 `Debug`；不等于面向用户的显示格式 |
| `PartialEq` | `==`、`!=` | 按字段比较；浮点数存在 NaN 不等于自身的情况 |
| `Eq` | 声明等价关系具有自反性 | 需要 `PartialEq`；浮点类型不能派生 `Eq` |
| `PartialOrd` | `<`、`<=`、`>`、`>=` 与 `partial_cmp` | 可能返回 `None`，例如 NaN |
| `Ord` | 全序比较与 `cmp` | 需要 `Eq`、`PartialEq`、`PartialOrd`；浮点类型不能直接派生 |
| `Clone` | 显式调用 `clone()` 复制值 | 可能复制堆数据；不等于廉价的位复制 |
| `Copy` | 赋值/传参时按位复制，不发生移动 | 所有字段都必须满足 Copy，类型不能实现 Drop |
| `Hash` | 将值映射为哈希输入 | HashMap/HashSet 的键通常还需 Eq；哈希碰撞仍可能发生 |
| `Default` | `Default::default()` 默认值 | 默认值必须符合业务语义，不能只为通过编译而派生 |

### 为什么没有可派生的 Display？

`Debug` 面向开发者检查内部状态；`Display` 面向用户展示，格式属于产品接口，需要开发者决定。D6 lab 中的 `Task` 派生 `Debug`，但手动实现 `Display`，请对照两种输出。

### 两个常见误区

1. `Clone` 不等于 `Copy`。拥有 `String` 的结构体可以派生 `Clone`，但不能派生 `Copy`。
2. `PartialOrd` 不保证任何两个值都能比较。浮点数的 NaN 与普通数比较时会得到 `None`，所以不能将浮点数直接当作 `Ord` 类型。

**动手实验**：在 D6 lab 中修改 `Task` 字段，观察派生约束；尝试给含 `String` 的类型增加 `Copy`，再运行对应的 compile-fail doctest；比较 `Debug` 和手写 `Display` 输出。

参考：[中文附录 C](https://rust.bootcss.com/appendix-03-derivable-traits.html) · [Derivable Traits](https://doc.rust-lang.org/book/appendix-03-derivable-traits.html)

---

## D · 实用开发工具（Useful Development Tools）

旧版附录列出的工具有些已经演进。学习时应理解工具职责，而不是照搬旧工具名称。

| 工具 | 当前用途 | 常用命令/入口 |
|---|---|---|
| rustfmt | 统一代码格式，不改变程序语义 | `cargo fmt`；CI 使用 `cargo fmt --check` |
| rustfix / cargo fix | 根据编译器建议修复代码、协助 Edition 迁移 | `cargo fix`、`cargo fix --edition`；必须审查 diff |
| Clippy | 检查常见错误、可读性和惯用写法 | `cargo clippy --all-targets -- -D warnings` |
| rust-analyzer | IDE 语言服务：补全、跳转、诊断、重构 | 在 VS Code 或其他支持 LSP 的编辑器安装 rust-analyzer |
| rustdoc | 从文档注释生成 API 文档，并执行 doctest | `cargo doc --open`、`cargo test --doc` |
| rustup | 管理 Rust 工具链、组件和目标平台 | `rustup show`、`rustup component list --installed` |

旧书中的 RLS（Rust Language Server）是历史工具；当前工程建议使用 rust-analyzer。IDE 提示是辅助信息，最终仍以实际 Cargo 检查和测试为准。

### 推荐的本地质量循环

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
cargo test --workspace --doc
cargo doc --workspace --no-deps --open
```

运行 `cargo fix` 前先提交或备份当前修改；它可能直接改写源码。Edition 迁移应在独立分支或临时副本进行，逐项检查 diff，并运行完整测试。

参考：[中文附录 D](https://rust.bootcss.com/appendix-04-useful-development-tools.html) · [rustfmt](https://github.com/rust-lang/rustfmt) · [Clippy](https://doc.rust-lang.org/clippy/) · [rust-analyzer](https://rust-analyzer.github.io/)

---

## E · Editions

Edition 是语言语法与惯用法的版本边界，不等于 Rust 编译器版本。Cargo.toml 的 `edition` 字段决定 crate 使用哪一版 Edition 规则；新版本编译器仍可编译较早 Edition 的 crate，不要求所有依赖同时升级。

| Edition | 代表变化（非完整清单） | 本课定位 |
|---|---|---|
| 2015 | 初始 Edition；旧中文书的早期示例基线 | 用于理解历史兼容 |
| 2018 | `async`/`await`/`dyn`、模块路径改进、raw identifiers | 中文版 Rust Book 1.37 基线 |
| 2021 | 数组 IntoIterator 行为、闭包捕获等 Edition 规则调整 | Rust AppLab 当前基础工程使用 |
| 2024 | `gen` 关键字保留、unsafe extern/属性相关规则、生命周期捕获等变化 | 需 Rust 1.85.0 或更新工具链 |

迁移不是“把 Cargo.toml 的数字改掉”这么简单。推荐流程：

1. 在独立分支记录当前测试基线。
2. 更新 `Cargo.toml` 的 `edition` 字段。
3. 使用当前工具链运行 `cargo fix --edition`，先检查建议再接受修改。
4. 阅读迁移警告，特别关注新关键字、unsafe 约束和生命周期捕获。
5. 运行 format、Clippy、单元/集成测试和 doctest。
6. 对依赖 crate 的 Edition、MSRV 和 API 行为分别核查。

**本仓库边界**：D1–D6 参考实验以 Edition 2021 / Rust 1.74 为基础；不要因为附录介绍 Edition 2024，就把整个根 Workspace 的 Edition 或 MSRV 一并升级。

参考：[中文附录 E](https://rust.bootcss.com/appendix-05-editions.html) · [Rust Edition Guide](https://doc.rust-lang.org/edition-guide/) · [官方 Edition 说明](https://doc.rust-lang.org/book/appendix-05-editions.html)

---

## F · 本书的翻译版本（Translations）

附录 F 是阅读导航，不是 Rust 语法规范，也不需要单独编写程序。翻译文本可能对应不同 Edition、Rust 版本和维护状态。

- 本轮逐节审计以 [rust.bootcss.com 中文版](https://rust.bootcss.com/) 为章节基线，其书稿声明 Rust 1.37 / Edition 2018。
- 当前语法和标准库行为以 [官方英文 Rust Book](https://doc.rust-lang.org/book/) 与 [Rust Reference](https://doc.rust-lang.org/reference/) 为准。
- 当翻译版本与当前工具链行为不一致时，保留历史文本作为概念参考，使用官方资料和可运行测试核对差异。
- 阅读译本时记录书稿基准版本，不把旧版章节编号直接套到当前官方书。

参考：[官方 Rust Book 的翻译入口](https://doc.rust-lang.org/book/)

---

## G · Rust 如何开发与 Nightly Rust

Rust 使用多个发布通道，让日常开发保持稳定，同时允许社区提前测试即将稳定的功能。

| 通道 | 内容 | 推荐用途 |
|---|---|---|
| Stable | 已稳定发布的工具链 | 日常开发、教学、CI、正式交付 |
| Beta | 下一次 Stable 发布候选 | 提前发现兼容性问题、验证依赖 |
| Nightly | 持续集成构建，包含尚未稳定的语言/编译器功能 | 语言实验、编译器开发、明确隔离的试验工程 |

### 查看本机工具链

```powershell
rustup show
rustup toolchain list
rustc +stable --version
rustc +beta --version
rustc +nightly --version
```

如需安装通道：

```powershell
rustup toolchain install stable
rustup toolchain install beta
rustup toolchain install nightly
```

在明确允许的实验工程中，可以用 `cargo +nightly check` 选择 Nightly。未稳定的语言功能通常需要 crate-level `#![feature(...)]` gate；这些功能不能在 Stable 上启用，也不应悄悄成为本仓库的基础依赖。

### 本课程的工具链纪律

- 基础教学、参考 Workspace 和正式 CI 以声明的 MSRV 与 Stable 证据为准。
- Beta 用于提前兼容性检查，不替代 Stable 验收。
- Nightly 实验必须单独隔离、写明功能 gate、工具链日期和退出条件。
- 不以“Nightly 能编译”证明 Stable/MSRV 可用，也不将不稳定 API 引入默认学员工程。

参考：[中文附录 G](https://rust.bootcss.com/appendix-07-nightly-rust.html) · [Rust Release Channels](https://rust-lang.github.io/rustup/concepts/channels.html) · [Nightly Rust](https://doc.rust-lang.org/book/appendix-07-nightly-rust.html)

---

## D6-A 实验与验收

从 RustAppLab 源码根目录运行：

```powershell
cargo fmt --manifest-path projects/book-d6-appendices/Cargo.toml --all -- --check
cargo clippy --locked --offline --manifest-path projects/book-d6-appendices/Cargo.toml --workspace --all-targets -- -D warnings
cargo test --locked --offline --manifest-path projects/book-d6-appendices/Cargo.toml --workspace --all-targets
cargo test --locked --offline --manifest-path projects/book-d6-appendices/Cargo.toml --workspace --doc
cargo test --locked --offline --manifest-path projects/book-d6-appendices/Cargo.toml --workspace --all-targets --release
```

### 练习任务

1. **关键字**：把实验函数名改为普通的 `match`，观察编译器错误；恢复为 `r#match`，解释 `r#` 的作用和限制。
2. **运算符**：预测 `2 + 3 * 4`、`(2 + 3) * 4`、`1..5` 的结果，再写断言验证。
3. **符号语境**：分别解释 `&value`、`a & b`、`*reference`、`a * b`、`module::item`、`expr?`。
4. **Trait**：为 `Task` 添加一个字段，判断哪些 derive 仍然成立；解释为什么 `Display` 需要手动实现。
5. **工具**：在 D6 lab 中制造一个 Clippy 可识别的问题，运行 Clippy，修复后比较输出。
6. **Edition**：复制 D6 lab 到临时目录，将副本迁移到 Edition 2024；记录工具链要求、迁移建议与测试结果，不修改原始工程。
7. **发布通道**：记录本机 Stable/Beta/Nightly 版本；解释为什么正式课程不能把 Nightly 作为隐含前提。

### 完成标准

- 能区分 strict/reserved/weak keywords，并说明 Edition 差异。
- 能解释常见运算符和同符号多语义，不靠猜测优先级。
- 能说明九个标准可派生 trait 的能力与约束，知道何时必须手写实现。
- 能实际运行格式化、lint、迁移、文档与 IDE 工具链。
- 能区分 Edition、编译器版本和 Stable/Beta/Nightly 发布通道。
- D6 Workspace 在 Rust 1.74、Linux 和 Windows 上通过格式、Clippy、debug、release 与 doctest 检查。

本课完成的是附录参考和最小可执行实验，不代表 D6 全部结束。剩余基础章节缺口（猜数字游戏、结构体/方法、HashMap、panic/Result 决策、环境变量、迭代器 I/O 与性能比较）进入 D6-B，单独逐项补齐。
