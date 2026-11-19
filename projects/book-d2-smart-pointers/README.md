# Book D2 · 智能指针参考实验

这是 Rust Book 第 15 章的**可选参考实验**：通过可运行的小例子比较 `Box`、`Deref`、`Drop`、`Rc`、`RefCell` 和 `Weak`。它不创建新的学员评分工程，也不属于根工作区的 17 个包。

## 先具备什么

先能解释普通变量的所有权、`&T` / `&mut T`、结构体和 trait。还在 FirstCard 阶段时，按 [基础后续指南](../../course/AFTER_FIRSTCARD.md) 学习；需要补借用时，先看 [所有权、接收者与切片](../../course/BOOK_OWNERSHIP_PRACTICE.md)。

本实验使用 Edition 2021、最低 Rust 1.74、零 registry 依赖，`publish = false`。有独立锁文件，以下命令可离线运行。完整解释和逐节练习在 [D2 讲义](../../course/BOOK_D2_SMART_POINTERS.md)。

## 先预测两个结果

在 **源码根目录或学员包根目录**（能看到 `projects/`）运行：

```powershell
cargo run --locked --offline --manifest-path projects/book-d2-smart-pointers/smart-pointer-lab/Cargo.toml -- box
cargo run --locked --offline --manifest-path projects/book-d2-smart-pointers/smart-pointer-lab/Cargo.toml -- weak
```

`box` 应包含 `sum=6`、`boxed=42`；`weak` 应包含 `parent_before_drop=root` 与 `parent_after_drop=none`。后者的重点是父对象释放后，弱引用能自然变为无对象。

把命令最后的主题替换为表中任意一项，一次只做一个实验：

| 主题 | 观察点 | 先回答的问题 |
|---|---|---|
| `box` | 递归结构、堆分配、trait object | 为什么递归数据需要固定大小的间接层？ |
| `deref` | 输出 `Hello, Rust!` | 自定义指针怎样被借用为目标类型？ |
| `drop` | 输出 `drops=1` | 哪个所有者结束时执行清理？ |
| `rc` | 输出包含 `owners=2` | clone 增加的是句柄还是独立数据副本？ |
| `refcell` | `value=2`、`conflict_handled=true` | 为什么借用冲突在运行时处理？ |
| `weak` | 父对象释放前后的 upgrade | 哪条关系不应该延长对象生命？ |

## 阅读与检查的入口

核心类型、demo 和单元测试在 `smart-pointer-lab/src/lib.rs`；参数入口在 `src/main.rs`，真实进程输出合同在 `tests/cli.rs`。先用表中的问题读相关片段，不需要一次读完整文件。

在 **同一个根目录**运行：

```powershell
cargo test --locked --offline --workspace --all-targets --manifest-path projects/book-d2-smart-pointers/Cargo.toml
cargo test --locked --offline --workspace --doc --manifest-path projects/book-d2-smart-pointers/Cargo.toml
```

参考代码应通过普通测试和文档测试；这只是行为证据，不是你的掌握程度记录。

## 用自己的例子检验理解

复制整个实验到新的个人目录。在副本 `smart-pointer-lab/src/lib.rs` 中增加一条 `Rc` 生命周期测试：创建两个拥有句柄，丢掉一个后数据仍在，再丢最后一个后 `Weak::upgrade` 返回 `None`。运行前写下 strong count 与 upgrade 结果的预测。

在 **副本 Workspace 根目录**执行：

```powershell
cargo test --locked --offline --workspace --all-targets
cargo test --locked --offline --workspace --doc
```

把预测、实际结果和新增测试写入学习日志。你应能说明何时用普通借用已经足够，何时需要共享所有权，并区分编译期借用检查与 RefCell 的运行时检查。不要制造泄漏来“证明”强引用环；用计数和 Drop 观察器即可。

`Rc` / `RefCell` 的例子面向单线程；跨线程共享需要另行理解 `Arc`、`Mutex` 等类型，不能直接照搬。

## 下一步

已有 TermGarden / BatchMeter 基础，可回到 [进阶 A](../../course/ADVANCED_A.md)，把强引用、弱视图和快照用于自己的作品。希望继续补动态分发与状态转换时，阅读 [D3 对象设计与模式](../book-d3-oop-patterns/README.md)；不要求把所有参考实验按编号刷完。
