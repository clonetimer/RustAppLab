# D4 · 高级特性

**对照原书**：[rust.bootcss.com 第 19 章](https://rust.bootcss.com/ch19-00-advanced-features.html)，对应当前英文版 Rust Book 的 [第 20 章 Advanced Features](https://doc.rust-lang.org/book/ch20-00-advanced-features.html)。本课覆盖 19.1–19.5 五节。

配套 [独立可运行 Workspace](../projects/book-d4-advanced-features/README.md)，Edition 2021、Rust 1.74、零第三方依赖、publish = false。过程宏只使用稳定 proc_macro API，不依赖 syn / quote，便于离线阅读 token 展开过程。

## 学习目标

完成本课后，你应能：

- 解释 unsafe 不会关闭借用检查器；它只开放五类需人工维护不变量的操作。
- 为裸指针、unsafe fn、unsafe trait、union 和可变静态数据写出可审查的安全契约。
- 使用关联类型、默认泛型参数、完全限定语法、超 Trait 和 Newtype 设计接口。
- 区分 Newtype 与类型别名，理解 Never、DST 和 ?Sized。
- 将函数指针与 Fn 闭包区分开，理解如何返回闭包。
- 读懂 macro_rules!，并实现 derive、属性和函数式过程宏。

## 0 · 运行实验

在 RustAppLab 源码根目录执行：

~~~powershell
cargo fmt --manifest-path projects/book-d4-advanced-features/Cargo.toml --all -- --check
cargo clippy --locked --offline --manifest-path projects/book-d4-advanced-features/Cargo.toml --workspace --all-targets -- -D warnings
cargo test --locked --offline --manifest-path projects/book-d4-advanced-features/Cargo.toml --workspace --all-targets
cargo test --locked --offline --manifest-path projects/book-d4-advanced-features/Cargo.toml --workspace --doc
cargo run --locked --offline --manifest-path projects/book-d4-advanced-features/advanced-lab/Cargo.toml -- unsafe
cargo run --locked --offline --manifest-path projects/book-d4-advanced-features/advanced-lab/Cargo.toml -- traits
cargo run --locked --offline --manifest-path projects/book-d4-advanced-features/advanced-lab/Cargo.toml -- types
cargo run --locked --offline --manifest-path projects/book-d4-advanced-features/advanced-lab/Cargo.toml -- functions
cargo run --locked --offline --manifest-path projects/book-d4-advanced-features/advanced-lab/Cargo.toml -- macros
~~~

先预测输出，再读 src/lib.rs。每个单元均有单元测试和 CLI 集成测试。测试验证参考实现的行为，不代表学员已经掌握概念。

# 19.1 · Unsafe Rust

## 五类 unsafe superpowers

unsafe 并不等于“这里可以随意写内存”。Rust 仍然检查普通引用、类型和借用规则；unsafe 只允许程序员承担编译器无法证明的五类不变量：

1. 解引用裸指针。
2. 调用 unsafe fn。
3. 访问或修改 static mut。
4. 实现 unsafe trait。
5. 访问 union 字段。

本实验分别展示裸指针、unsafe fn、extern C 函数与外部 ABI 声明、unsafe trait、union 与可变静态数据。每处 unsafe 都有局部 SAFETY 注释，说明不变量由谁保证。

## 先从安全抽象开始

sum_via_raw_pointer 从一个有效 slice 获取指针，循环边界严格限制在 0..values.len()。调用者只传 &[i32]，因此安全契约由函数内部封装；空 slice 也不会解引用指针。

**练习**：将循环改成错误的 0..=values.len()，观察测试或审查为何会指出越界。不要运行可能触发未定义行为的版本；通过代码审查和 Miri 等工具了解边界即可。

## unsafe 函数与调用者契约

add_to_raw_pointer 要求调用者提供非空、对齐、有效、已初始化且独占访问的 *mut i32。这些条件无法仅靠函数签名证明，所以函数标记为 unsafe，并在 Safety 文档中列明前置条件。

测试只传入局部变量的有效可变指针。课程的 compile-fail doctest 则验证：不使用 unsafe 块无法调用该接口。

## unsafe trait、union 与 static mut

ValidUtf8 是一个 unsafe trait：实现者承诺返回的字节在借用期内始终是合法 UTF-8。AsciiLabel 通过私有数组、ASCII 构造检查和长度边界维护该承诺；实现中才使用 from_utf8_unchecked。

extern C 声明让 Rust 调用遵循外部 C ABI；示例在同一 crate 中提供了签名完全匹配的导出符号，验证调用边界而不依赖系统库或额外 C 编译器。真实 FFI 还必须核对 ABI、指针有效期、布局、所有权和错误/异常边界。

NumberUnion 只能在调用者知道当前活动字段时读取。示例只读取刚刚写入的 integer 字段。

increment_single_thread_counter 展示可变静态变量的限制：调用者必须保证不存在并发读写。真实项目应优先使用 AtomicU32 或互斥锁；此 API 仅用于理解语言机制，测试中也只在单线程路径调用一次。

**练习**：
- 为 AsciiLabel 添加一个安全构造函数，说明为什么外部代码无法制造非法内部字节。
- 将全局计数器替换为 AtomicU32，比较两种 API 的调用约束。
- 给 read_union_integer 增加一个 float 读取接口，明确调用者必须满足的活动字段前置条件。

# 19.2 · 高级 Trait

## 关联类型

Measurement 将输出类型写在 trait 的关联类型 Unit 中。Pulse 实现时指定 Unit = u32，调用者不必在每次调用时重复提供泛型参数。

**练习**：增加 Temperature，让它输出 f32；解释为什么关联类型属于实现者与 trait 之间的接口契约。

## 默认泛型参数与运算符重载

std::ops::Add<Rhs = Self> 的右操作数默认与左操作数相同。Point 演示同类型相加，Millimeters 则额外实现 Add<Meters>，展示不同类型之间有明确语义的加法。

**练习**：增加 Seconds 和 Minutes，实现在量纲上有意义的加法。禁止直接把不同单位的底层 u32 混用。

## 完全限定语法、超 Trait 与 Newtype

Human::baby_name 与 Pilot::baby_name 同名，使用 <Human as Pilot>::baby_name() 明确选择 trait 实现。OutlinePrint: Display 表示实现者必须先实现 Display。Wrapper(Vec<String>) 则利用本地 Newtype 为外部容器实现外部 Display，遵守孤儿规则。

**练习**：为 Wrapper 实现 IntoIterator，再讨论它是否应该暴露内部 Vec 的所有方法。

# 19.3 · 高级类型

- **Newtype**：Meters(u32) 与 Millimeters(u32) 是不同类型，编译器能阻止单位混用。
- **类型别名**：UnitResult<T> 只是 Result<T, &'static str> 的另一个名字，不创建新类型。
- **Never 类型**：fail_fast() -> ! 表示函数不会正常返回，可用于 panic、退出或无限循环等发散路径。
- **DST**：str 在编译期没有固定大小，必须通过引用或智能指针等间接层使用。
- **Sized / ?Sized**：泛型默认隐含 Sized；T: ?Sized 放宽限制，允许 str 这样的动态大小类型通过引用传入。

**练习**：新增 Seconds Newtype，写一个 format_value<T: Display + ?Sized> 函数，并比较 type Seconds = u32 与 struct Seconds(u32) 的类型检查差异。

# 19.4 · 高级函数与闭包

fn(i32) -> i32 是函数指针类型，可以接收普通函数，也可以接收不捕获环境的闭包。Fn 是闭包 trait，不是函数指针的拼写变体。

make_adder 返回 impl Fn(i32) -> i32，闭包通过 move 捕获 amount。当不同分支返回不同闭包具体类型时，使用 Box<dyn Fn...> 进行类型擦除。

**练习**：新增一个接受 fn(i32) -> i32 的批量转换函数，再实现一个捕获阈值的闭包。解释二者的存储、捕获和动态分发差异。

# 19.5 · 宏

## 声明宏

make_pair! 与 make_record! 展示 macro_rules! 的模式匹配、片段说明符和重复展开。宏在编译期生成语法结构；函数则在运行时按调用参数执行。

**练习**：为 make_record! 增加可选的 derive 列表，或增加一个支持零字段结构体的分支。

## 三种过程宏

| 类型 | D4 实验 | 观察点 |
|---|---|---|
| 自定义 derive | #[derive(Describe)] | 根据输入类型名生成 trait 实现 |
| 属性宏 | #[audit_tag(telemetry)] | 保留原 item，并注入文档标记与关联常量 |
| 函数式宏 | token_count!(alpha beta gamma) | 接收 TokenStream 并展开成编译期整数 |

过程宏位于独立 d4-proc-macros crate。为了让 token 机制更直观，本实验直接遍历 proc_macro::TokenTree，不依赖 syn / quote。因此 D4 的 derive / attribute 教学宏只接受具名、非泛型 struct/enum；这不是生产级通用宏解析器。

**练习**：
1. 给 Describe 增加一个字段计数常量。
2. 让属性宏接受一个字符串字面量，并将它标准化为文档标记。
3. 让函数式宏统计输入中标识符的数量，而不是所有 token 的数量。
4. 对比宏和泛型函数：哪些任务必须在编译期改变语法树？

## D4 自动验收

| 单元 | 可复现证据 |
|---|---|
| 19.1 | 裸指针安全抽象、unsafe fn、unsafe trait、union、单线程 static mut |
| 19.2 | 关联类型、默认泛型参数、运算符重载、完全限定语法、超 Trait、Newtype |
| 19.3 | Newtype、类型别名、Never、DST、?Sized |
| 19.4 | 函数指针、枚举构造函数、返回闭包、Box<dyn Fn> |
| 19.5 | macro_rules!、derive、属性宏、函数式过程宏 |

自动化测试验证的是参考实现的行为。完成学习还需要自行扩展一个 unsafe 安全抽象、一个 trait 接口、一个闭包和一个过程宏，并为新增行为编写测试。

参考资料：
- [Unsafe Rust](https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html)
- [Advanced Traits](https://doc.rust-lang.org/book/ch20-02-advanced-traits.html)
- [Advanced Types](https://doc.rust-lang.org/book/ch20-03-advanced-types.html)
- [Advanced Functions and Closures](https://doc.rust-lang.org/book/ch20-04-advanced-functions-and-closures.html)
- [Macros](https://doc.rust-lang.org/book/ch20-05-macros.html)
