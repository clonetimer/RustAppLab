# D3 · 面向对象设计与模式匹配

**对照原书**：[Rust Book 第 17 章：面向对象的特性](https://rust.bootcss.com/ch17-00-oop.html) 与 [第 18 章：模式与匹配](https://rust.bootcss.com/ch18-00-patterns.html)，覆盖 17.1–17.3、18.1–18.3。

配套 [独立可运行 Workspace](../projects/book-d3-oop-patterns/README.md)，Edition 2021、Rust 1.74、零第三方依赖、`publish = false`。本课是参考实验，不增加学员项目数量，不进入冻结 RC 根 Workspace。

## 学习目标

完成本课后，你应能：

- 解释封装、行为抽象、多态与继承的差异，知道 Rust 用 trait、组合和泛型表达不同设计。
- 用 trait object 在运行时选择不同具体类型的行为，并理解动态分发的代价与边界。
- 用状态模式限制业务对象的可见行为，理解状态对象的所有权转移。
- 读写 Rust 的常见模式，判断一个模式是可反驳还是不可反驳。
- 正确使用解构、忽略模式、范围、match guard、`@`、切片模式和引用匹配。

## 0 · 运行实验

在 RustAppLab 源码根目录执行：

```powershell
cargo test --locked --offline --workspace --all-targets --manifest-path projects/book-d3-oop-patterns/Cargo.toml
cargo test --locked --offline --workspace --doc --manifest-path projects/book-d3-oop-patterns/Cargo.toml
cargo run --locked --offline --manifest-path projects/book-d3-oop-patterns/oop-pattern-lab/Cargo.toml -- patterns
cargo run --locked --offline --manifest-path projects/book-d3-oop-patterns/oop-pattern-lab/Cargo.toml -- oop
cargo run --locked --offline --manifest-path projects/book-d3-oop-patterns/oop-pattern-lab/Cargo.toml -- state
```

先预测输出，再阅读 `src/lib.rs`。测试是教师参考代码的行为合同，不是学员掌握程度认证。

# 第 17 章 · 面向对象特性

## 17.1 · 对象、封装与继承取舍

面向对象设计通常强调数据与行为的组织、封装边界和多态。Rust 可以把状态封装在 struct 私有字段中，通过公开方法维护不变量；这不要求继承。

本实验的 `AveragedCollection` 同时维护 `values` 和 `sum`。如果调用者能直接改字段，二者可能失去一致性；因此字段私有，修改只能经过 `add` 和 `remove`。平均值由受控状态计算。

```rust
let mut values = AveragedCollection::default();
values.add(10);
values.add(20);
assert_eq!(values.average(), Some(15.0));
```

**练习**：尝试把 `values` 或 `sum` 改成公开字段，再写一段能破坏总和不变量的代码；恢复私有字段后，解释为什么公开方法是稳定边界。再为集合增加 `clear`，保证两个字段同步。

Rust 没有传统类继承机制。共享行为通常通过 trait 表达，共享实现可以通过组合、泛型函数或委托实现。继承并非免费复用：基类状态与子类耦合、重写规则和可替换性都需要额外约束。

## 17.2 · Trait Object：运行时多态

`Draw` trait 定义行为，`Button` 与 `Text` 提供不同实现。屏幕保存 `Vec<Box<dyn Draw>>`，因此集合不要求所有元素具有同一个具体类型；调用 `draw` 时通过 trait object 进行动态分发。

```rust
let screen = Screen::new(vec![
    Box::new(Button { label: "save".into() }),
    Box::new(Text { value: "ready".into() }),
]);
assert_eq!(screen.run(), vec!["[button:save]", "[text:ready]"]);
```

对比：
- 泛型 `fn render<T: Draw>(item: &T)` 通常采用单态化静态分发。
- `&dyn Draw` / `Box<dyn Draw>` 通过虚表动态分发，适合运行时混合多种实现。
- trait object 有对象安全等约束；不是所有 trait 方法都能直接变成 `dyn Trait` 的对象接口。
- `Box` 负责拥有具体对象，`dyn Draw` 负责隐藏具体类型并提供行为接口。

**练习**：新增 `Image` 类型实现 `Draw`，不修改 `Screen::run` 就让它出现在屏幕中。再把容器改成同一种具体类型的泛型集合，比较两种设计分别适合什么场景。

## 17.3 · 状态模式：让状态决定可执行行为

`Post` 具有 Draft、PendingReview、Published 三种状态。状态对象实现同一 trait；调用 `request_review` 或 `approve` 时，旧状态对象被消费并返回新状态对象。未发布状态的 `content` 返回空字符串，只有 Published 才暴露正文。

```text
Draft --request_review--> PendingReview
PendingReview --approve--> Published
```

实验还约束了非法操作：Draft 直接 approve 不会跳过审核；重复 request_review 不会额外跳状态；Published 重复 approve 不会退回。

**练习**：新增 Rejected 状态，设计审核拒绝后的行为。明确拒绝后是否可以重新提交、是否保留正文、是否能直接发布；为每条状态边写测试。不要仅凭状态名称判断流程正确，要验证每个公开操作的前置条件和结果。

# 第 18 章 · 模式与匹配

## 18.1 · 模式可以出现在哪里

本实验展示多种位置：
- `match`：完整分支匹配。
- `if let`：只关心一个成功分支。
- `while let`：只要匹配成功就继续循环。
- `for`：每轮从迭代项中解构。
- `let-else`：匹配失败时提前退出。
- 函数参数：例如 `fn describe_point((x, y): (i32, i32))`。

`if let Some(x) = value` 是可反驳模式，因为输入也可能是 None；普通 `let Some(x) = value` 要求模式不可反驳，否则无法保证绑定一定成功。若失败路径需要处理，使用 `match`、`if let` 或 `let-else`。

**练习**：分别用 match、if let、let-else 处理同一个 `Option<String>`，比较分支表达方式；再将 `while let` 改成普通 loop + match，确认终止条件没有变化。

## 18.2 · 可反驳与不可反驳模式

- **不可反驳（irrefutable）**：对该类型的所有可能值都能匹配，例如 `let (x, y) = (1, 2)`、函数参数中的元组解构。
- **可反驳（refutable）**：只匹配部分值，例如 `Some(x)`、`1..=5`、`Message::Quit`。

普通 `let` 绑定要求不可反驳模式；`if let`、`while let`、`match` 分支可以使用可反驳模式。课程 doctest 故意包含一个不能编译的 refutable-let 例子，验证编译器会拒绝这种用法。

**练习**：把 `let (x, y) = point` 和 `let Some(x) = option` 分别编译；解释为什么前者覆盖所有 tuple 值，后者不覆盖 None。再使用 `let Some(x) = option else { return; }` 修复第二个例子。

## 18.3 · 模式语法工具箱

| 语法 | 示例 | 作用 |
|---|---|---|
| 字面量 | `0`、`"quit"` | 匹配特定值 |
| 多模式 | `1 | 2` | 任一模式命中 |
| 范围 | `3..=9`、`'a'..='z'` | 匹配闭区间 |
| 元组/枚举解构 | `(x, y)`、`Message::Move { x, y }` | 提取内部字段 |
| 结构体剩余字段 | `User { username, .. }` | 只取关心字段 |
| 忽略 | `_`、`(first, _, last)` | 不绑定某些值 |
| 切片 | `[first, .., last]` | 匹配首尾并忽略中间 |
| 引用匹配 | `match &option { Some(x) => ... }` | 借用式匹配，使用 match ergonomics |
| 显式 ref | `Some(ref text)` | 在按值匹配时借用内部值 |
| 匹配守卫 | `Some(x) if x >= limit` | 在结构匹配后追加条件 |
| @ 绑定 | `id @ 1..=100` | 同时检查范围并绑定原值 |

**练习**：
1. 新增 `Message::Resize { width, height }` 并解构两个字段。
2. 对 `&[i32]` 增加“空、单项、多项”三种切片匹配。
3. 使用 `id @ 100..=999` 实现三位编号校验。
4. 使用 match guard 区分超过阈值、低于阈值和缺失值。
5. 对比 `Some(ref text)` 与直接移动 String 的区别。

## D3 自动验收

| 小节 | 可复现证据 |
|---|---|
| 17.1 | 私有字段维护聚合不变量，add/remove/average 测试 |
| 17.2 | 多种 Draw 实现放入一个 trait-object Screen 并动态分发 |
| 17.3 | Draft、PendingReview、Published 的合法与非法状态转换 |
| 18.1 | match、if let、while let、for、let-else、函数参数 |
| 18.2 | irrefutable/refutable 对照与 compile_fail doctest |
| 18.3 | 字面量、多模式、范围、解构、忽略、切片、ref、guard、@ |

参考实验目标是让你能解释设计取舍，而不是只通过固定输出。请在个人副本增加一个新 Draw 类型、一个 Post 状态或一个模式分支，并新增测试。

官方资料：[Object-Oriented Features](https://doc.rust-lang.org/book/ch17-00-oop.html) · [Using Trait Objects](https://doc.rust-lang.org/book/ch18-02-trait-objects.html) · [State Pattern](https://doc.rust-lang.org/book/ch18-03-oo-design-patterns.html) · [Patterns](https://doc.rust-lang.org/book/ch19-00-patterns.html)。中文版 Rust 2018 与当前官方书的章节编号可能不同，以主题对照为准。
