# 第 10 章动手练习：泛型、trait 与生命周期

从仓库根目录运行以下命令。打开源码，先遮住断言里的预期结果，逐题预测，再运行测试确认。修改时使用临时副本，保留原始参考实现。

```bash
cargo test --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab --test generic_lifetimes
cargo test --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab --doc generic_lifetimes
```

[参考源码](../projects/book-d6-foundations/foundation-lab/src/generic_lifetimes.rs)把可运行代码与 `compile_fail` 示例放在一起。文档测试中，预期被编译器拒绝的例子被拒绝后计为通过。[行为测试](../projects/book-d6-foundations/foundation-lab/tests/generic_lifetimes.rs)还检查了引用指向哪里、并列时选谁，以及空输入的行为。

## 10.1 从同一个 T 到独立的 T 和 U

1. **预测。** `pair<T>(left: T, right: T)` 能否接受 `pair(7, "Rust")`？`Card<T>` 中哪个字段使用 `T`？
2. **运行。** 在 `Card` 的文档示例中观察 `E0308` 和独立使用两个类型参数的成功对照。运行 `Card<String>` 映射为 `Card<usize>` 的示例。
3. **修改。** 把映射函数从 `|text| text.len()` 改成 `String::into_bytes`，更新类型和断言。随后尝试访问原卡，预测编译器会怎样处理。
4. **解释。** `impl<T> Card<T>` 的第一个 `<T>` 为实现块声明类型参数。方法的 `U` 是一次调用的输出值类型，由映射函数决定。方法接收 `self`，标签移入新卡，旧值交给映射函数消费。

把 `Pair<T>` 的两个字段先都填成整数，再换一个成字符串。用 `Pair<T, U>` 修复后，显式写出 `Pair<i32, &str>`，交换初始化值并预测哪一处会报错。再给泛型枚举 `Input<T, U>` 的 `match` 尝试删掉一个分支，观察编译器如何提示遗漏的载荷。

类型参数可以出现在函数、结构体、枚举和方法中。`Option<T>` 用一个类型参数描述 `Some` 中的值，`Result<T, E>` 分别描述成功值和错误值的类型。编译器通过单态化为实际使用的具体类型生成代码。

## 10.2 用 trait 约束可用操作

1. **预测。** 分别尝试为 `Card<Vec<i32>>`、`Card<usize>`、`Card<i32>` 调用 `new`、`describe` 和 `is_positive`。先列出哪些调用能通过。
2. **运行。** 对照 `impl<T>`、`impl<T: Display>` 与 `impl Card<i32>`。读 `E0599` 的调用位置和约束说明。查看 `Summary` 如何通过必需的 `label` 生成默认摘要，以及 `Notice` 覆盖后的输出。
3. **修改。** 在 `summaries` 的示例中去掉 `Notice::summarize`，调整断言。再把函数写成接受两个 `&T` 的版本，预测哪些调用会得到 `E0308`。在返回 `impl Summary` 的例子中让两个分支构造相同类型的卡，验证修复。
4. **解释。** 两处参数位置的 `impl Summary` 各自接受一种实现了 `Summary` 的类型。共享 `T` 的两个参数都采用同一类型。返回位置的 `impl Summary` 隐藏一个具体类型，各返回分支必须一致。运行 `format_inline` 和 `format_where`，比较同一输入的结果。再将其中一个签名的 `U: Summary` 搬到另一种位置，保持两个函数等价。尝试只移除一个函数的 `Debug` bound，并找到函数体里依赖该约束的格式化表达式。

在 `largest` 的例子中，先看无约束比较产生的 `E0369`。实现用 `T: Ord` 支持全序，并且只在找到严格更大的值时更换候选。试着把比较改为 `>=`，用两个内容相同的 `String` 检查返回的地址。再比较最高分实验中 `largest_copy`、`largest_clone` 和 `largest` 的返回类型。用有姓名的记录替换整数分数，预测哪个拥有值版本还能调用，并解释克隆字符串可能带来的分配。最后把输入换成空切片，确认三个版本都返回 `None`。

## 10.3 追踪输出引用的来源

1. **预测。** `choose_longer("中", "ab")` 会返回哪一项？一个汉字可占多个 UTF-8 字节，`str::len()` 计算字节数。预测同字节数时输出引用的来源。
2. **运行。** 看 `E0597` 对应的短作用域例子。虽然左边的实际内容更长，`choose_longer<'a>(&'a str, &'a str) -> &'a str` 允许返回任一输入，所以输出的使用范围必须落在两个输入引用都有效的范围内。类型签名中的 `'a` 是两者都能承诺的借用范围，标注本身无法延长拥有者的生存时间。
3. **修改。** 把失败示例的最后一次使用移到内层作用域，或调用契约只借第一个输入的 `first_label`。再让 `Excerpt` 的原文在内层创建，观察逃逸失败后把原文移到外层。
4. **解释。** `Excerpt<'a>` 的参数约束它存放的原文引用。`without_prefix` 的输出跟随此次对 `self` 的借用，因此临时前缀可以先离开作用域。参考源码还演示了如何在同一个函数签名中使用 `'a`、`T` 与 `T: Summary`，并让仅用于生成摘要的参数提前离开作用域。

### 生命周期省略的三条规则

| 规则 | 练习里的对应关系 |
| --- | --- |
| 每个省略的输入引用生命周期获得独立参数。 | 单独出现的两个 `&str` 默认是两个借用参数。 |
| 输入生命周期恰好一个时，将它赋给所有省略的输出生命周期。 | `largest(&[T]) -> Option<&T>` 的输出借自切片。 |
| 有多个输入生命周期且其中一个是 `&self` 或 `&mut self` 的生命周期时，将接收者的生命周期赋给省略的输出生命周期。 | `without_prefix(&self, prefix: &str) -> &str` 可展开为 `without_prefix<'s, 'p>(&'s self, prefix: &'p str) -> &'s str`。 |

当规则无法确定输出借自谁时，编译器要求补充标注。源码中的双输入自由函数例子产生 `E0106`，`first_label` 只标注第一个输入与输出的关系。字符串字面量可以直接用作 `&'static str`。函数结束时本地 `String` 会被销毁，试图返回它的 `&'static str` 会产生 `E0515`。需要把本地构造的字符串交给调用者时，可以返回拥有的 `String`。

## 迁移练习

- 在 [D2 智能指针](BOOK_D2_SMART_POINTERS.md) 找 `boxed_label`，比较单参数 `impl Into<String>` 与 `summaries` 的双参数约束。
- 在 [D4 高级特征](BOOK_D4_ADVANCED_FEATURES.md) 找 `Wrapper`、`OutlinePrint` 和 `make_adder`，对照孤儿规则、`Display` 带来的 `ToString` 实现，以及为什么不同闭包类型需要另一种返回方案。
- 在 [MiniKV](studio/minikv.md) 找 `Database<J>` 和 `PrefixScan<'a>`，追踪日志类型约束和迭代结果的借用来源。在 [TextScope](studio/textscope.md) 找 `search`，解释为何查询词可以比返回的原文视图更早失效。

原书章节为 [10.1 泛型数据类型](https://rust.bootcss.com/ch10-01-syntax.html)、[10.2 trait](https://rust.bootcss.com/ch10-02-traits.html) 与 [10.3 生命周期](https://rust.bootcss.com/ch10-03-lifetime-syntax.html)。
