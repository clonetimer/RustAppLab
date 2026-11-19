# Book D3 · 对象设计与模式匹配参考实验

这是 Rust Book 第 17–18 章的**可选参考实验**，通过集合封装、可替换的绘制组件、文章状态和模式匹配，解释如何组织有行为的数据。它不增加学员项目，不进入根工作区的 17 个包。

## 前置与目标

先理解 `struct`、`enum`、`match`、方法、trait 和基本借用；trait object 涉及的 `Box` 可参考 [D2](../book-d2-smart-pointers/README.md)。FirstCard 初学者应先按 [基础后续指南](../../course/AFTER_FIRSTCARD.md) 做自己的小作品。

实验使用 Edition 2021、最低 Rust 1.74、零 registry 依赖、`publish = false`。完成练习后，你应能保护结构体内部状态、用 `dyn Trait` 表达不同实现，并说明模式怎样匹配和绑定数据。完整课程见 [D3 讲义](../../course/BOOK_D3_OOP_PATTERNS.md)。

## 运行三种行为

在 **源码根目录或学员包根目录**（能看到 `projects/`）运行：

```powershell
cargo run --locked --offline --manifest-path projects/book-d3-oop-patterns/oop-pattern-lab/Cargo.toml -- patterns
cargo run --locked --offline --manifest-path projects/book-d3-oop-patterns/oop-pattern-lab/Cargo.toml -- oop
cargo run --locked --offline --manifest-path projects/book-d3-oop-patterns/oop-pattern-lab/Cargo.toml -- state
```

| 主题 | 应观察到什么 | 对应问题 |
|---|---|---|
| `patterns` | 包含 `zero`、`small-positive`、`y-axis`、`valid-id:42` | 范围、解构与 guard 怎样选择分支？ |
| `oop` | `average=15.0`、`components=2` 和两种组件输出 | 私有状态和 trait object 分别解决什么问题？ |
| `state` | 最终 `published=true` 并显示文章正文 | 哪个状态允许访问内容，状态如何转移？ |

状态演示中的 `draft-visible=true` / `review-visible=true` 当前实际打印的是 `content().is_empty()`；含义是这两个阶段的正文为空。结合源码理解，不把变量标签当作行为证明。

## 从哪些文件学

| 位置 | 阅读重点 |
|---|---|
| `oop-pattern-lab/src/lib.rs` | `AveragedCollection` 的不变量、`Draw` / `Screen`、文章状态和模式函数 |
| `oop-pattern-lab/src/main.rs` | 将三个主题接到 CLI，拒绝未知主题或额外参数 |
| `oop-pattern-lab/tests/cli.rs` | 用真实进程验证外部可见结果 |
| 库内 rustdoc | 可运行示例与编译期边界 |

在 **同一个根目录**检查参考实现：

```powershell
cargo test --locked --offline --workspace --all-targets --manifest-path projects/book-d3-oop-patterns/Cargo.toml
cargo test --locked --offline --workspace --doc --manifest-path projects/book-d3-oop-patterns/Cargo.toml
```

普通测试与文档测试检查不同目标。它们通过不自动说明你能独立设计状态或写出新模式。

## 做一次自己的设计变更

把整个实验复制到新的个人目录。在副本的 `oop-pattern-lab/src/lib.rs` 给 `AveragedCollection` 增加 `clear`，同时维护内部数据与总和；加入“清空后没有平均值，再添加新数据后重新计算”的测试。任务来自讲义中的封装练习。

在 **副本 Workspace 根目录**运行：

```powershell
cargo test --locked --offline --workspace --all-targets
cargo test --locked --offline --workspace --doc
```

再挑一个模式函数加入自己的边界输入，先预测它走哪个分支。记录修改、实际结果及解释：为什么字段私有有助于维护一致性？`Vec<Box<dyn Draw>>` 为什么能放不同具体类型？可反驳模式在什么位置需要处理匹配失败？

## 下一步

把封装或模式匹配用于自己的 TermGarden / TaskPocket，再看行为是否更容易测试。需要补关联类型、闭包返回值或宏时，进入 [D4 高级特性](../book-d4-advanced-features/README.md)；其他问题按 [Book 对照指南](../../course/BOOK_GUIDE.md) 选择，不必把参考测试当作通关列表。
