# D2 · 智能指针：所有权、间接层与运行时借用

**对照原书**：[Rust Book 第 15 章：智能指针](https://rust.bootcss.com/ch15-00-smart-pointers.html)，覆盖 15.1–15.6。建议先完成 [所有权与借用](FOUNDATION_PATH.md)、[进阶 A](ADVANCED_A.md) 中的生命周期和 Trait 内容。

本课配套 [独立可运行 Workspace](../projects/book-d2-smart-pointers/README.md)。它不加入冻结 RC 的根 Workspace，不依赖 crates.io，Edition 2021 / Rust 1.74。命令与检查均离线。

## 先建立统一模型

智能指针不是“更高级的引用”的同义词。它是拥有数据、控制访问或定义生命周期语义的类型。理解时始终回答四个问题：

1. **谁拥有数据？** 单一所有者、共享所有者，还是只有非拥有观察者？
2. **数据放在哪里？** 内联、堆上，还是通过指针间接访问？
3. **借用规则何时检查？** 编译期，还是运行时？
4. **所有者消失时发生什么？** 自动析构、引用计数归零，还是弱引用升级失败？

| 类型 | 主要职责 | 所有权语义 | 关键约束 |
|---|---|---|---|
| `Box<T>` | 堆分配与固定大小间接层 | 唯一所有者 | 释放 Box 时释放其拥有的数据 |
| `Deref<Target = U>` | 自定义解引用与强制转换 | 不创造新所有权 | 访问目标，不改变目标生命周期 |
| `Drop` | 自定义清理 | 与所有者生命周期绑定 | 离开作用域时自动执行 |
| `Rc<T>` | 单线程共享所有权 | 多个强引用 | 最后一个强引用消失时释放 |
| `RefCell<T>` | 内部可变性 | 仍由外层所有者管理 | 借用规则在运行时检查 |
| `Weak<T>` | 非拥有关系 | 不增加强引用计数 | `upgrade()` 可能返回 `None` |

## 0 · 运行真实实验

在 RustAppLab 源码根目录执行：

```powershell
cargo test --locked --offline --workspace --all-targets --manifest-path projects/book-d2-smart-pointers/Cargo.toml
cargo test --locked --offline --workspace --doc --manifest-path projects/book-d2-smart-pointers/Cargo.toml
```

逐项观察：

```powershell
cargo run --locked --offline --manifest-path projects/book-d2-smart-pointers/smart-pointer-lab/Cargo.toml -- box
cargo run --locked --offline --manifest-path projects/book-d2-smart-pointers/smart-pointer-lab/Cargo.toml -- deref
cargo run --locked --offline --manifest-path projects/book-d2-smart-pointers/smart-pointer-lab/Cargo.toml -- drop
cargo run --locked --offline --manifest-path projects/book-d2-smart-pointers/smart-pointer-lab/Cargo.toml -- rc
cargo run --locked --offline --manifest-path projects/book-d2-smart-pointers/smart-pointer-lab/Cargo.toml -- refcell
cargo run --locked --offline --manifest-path projects/book-d2-smart-pointers/smart-pointer-lab/Cargo.toml -- weak
```

先预测输出与所有权变化，再运行。不要只看最终数字；打开 `smart-pointer-lab/src/lib.rs` 对照每个类型的实现和测试。

## 15.1 · Box<T>：把递归大小问题变成固定大小指针

枚举的每个变体都必须具有编译期已知的大小。下面这种递归结构若直接内嵌 `List`，其大小会无限递归；`Box<List>` 把递归部分放到堆上，枚举本身只保存一个固定大小的指针。

```rust
enum List {
    Cons(i32, Box<List>),
    Nil,
}
```

实验还使用 `Box<dyn Drawable>`：Box 拥有具体值，trait object 隐藏具体类型并通过动态分发调用方法。它与 `Rc<dyn Trait>` 的共享所有权不是一回事。

**练习**：在自己的副本中增加 `len()`，测试空链表、单元素、多元素三种情况；画出每个 `Cons` 与 Box 指针的关系。解释为什么 Box 解决的是递归类型大小，而不是让所有 Rust 数据都必须放在堆上。

## 15.2 · Deref：定义解引用行为与 Deref coercion

`Deref` 允许 `*value` 访问目标；`DerefMut` 允许在可变上下文中修改目标。实验中的 `MyBox<T>` 只包裹一个值，并不分配堆内存——这点与 `Box<T>` 不同。

```rust
let name = MyBox::new(String::from("Rust"));
let message = greet(&name); // &MyBox<String> -> &String -> &str
```

强制转换由编译器在满足类型关系时自动插入解引用，不等于把任意类型都转换成任意引用。不要为了语法便利而随意为类型实现 `Deref`：它会改变调用者看到的 API 行为。

**练习**：移除 `Deref` 实现，观察调用失败；恢复后再移除 `DerefMut`，观察 `push_str` 失败。写出两次错误的目标类型差异。

## 15.3 · Drop：确定性清理与析构时机

实现 `Drop` 后，值离开作用域时 Rust 自动运行析构逻辑。需要提前释放资源时，调用 `std::mem::drop(value)`；不要直接调用 `value.drop()`。所有权被移动到 `drop` 后，原绑定不再可用。

实验使用 `Arc<AtomicUsize>` 记录析构次数，避免依赖控制台输出或时间推断。这里使用 Arc 只是为了让测试能共享计数器，不代表 Drop 本身要求多线程。

**练习**：创建两个资源，观察作用域结束后的析构次数；将其中一个显式 `drop`，预测第二次计数变化。进一步用带标签的记录器验证局部变量按离开作用域的逆序析构。

## 15.4 · Rc<T>：单线程共享所有权

`Rc::clone(&value)` 克隆的是引用计数句柄，不是深拷贝底层数据。强引用计数大于零时，分配继续存活；计数归零后，值被释放。

```rust
let first = SharedText::new("telemetry");
let second = first.clone_shared();
assert_eq!(first.strong_count(), 2);
drop(second);
assert_eq!(first.strong_count(), 1);
```

`Rc<T>` 不是线程安全的共享所有权类型，不能跨线程发送。多线程共享所有权应考虑 `Arc<T>`，但 Arc 本身不自动提供内部可变性。

**练习**：增加第三个 owner，画出 strong_count 从 1 到 3 再回到 1 的变化；解释为什么 `Rc::clone` 与 `String::clone` 的成本和语义不同。

## 15.5 · RefCell<T>：运行时借用检查

`RefCell<T>` 将借用规则检查从编译期移动到运行时：可以有多个不可变借用，或者一个可变借用，但不能同时存在冲突借用。违反规则的 `borrow_mut()` 会 panic；`try_borrow_mut()` 则把冲突作为 `Result` 返回。

`Rc<RefCell<T>>` 常用于单线程场景中的共享可变状态：Rc 管理多个所有者，RefCell 管理内部借用。两者职责不同，组合并不会消除运行时 panic 风险。

**练习**：先持有一个不可变借用，再调用 `try_increment`；观察它返回错误。释放借用后再次调用，验证成功。尝试持有两个不可变借用，确认这属于允许状态；再写一项测试验证可变借用期间第二个借用失败。

## 15.6 · 引用循环与 Weak<T>

若两个对象都用 Rc 强引用对方，strong_count 永远不会归零，析构无法发生，形成引用循环。下面是**反例结构示意，不要在长生命周期程序中照抄**：

```rust
// A -> Rc<B>，B -> Rc<A>：双方互相持有，可能永久保留。
```

树结构通常让父节点强拥有子节点，而子节点用 `Weak<Parent>` 回指父节点：

```text
Parent --Rc--> Child
Parent <--Weak-- Child
```

`Weak` 不保持目标存活；使用 `upgrade()` 时得到 `Option<Rc<T>>`。若最后一个强引用已经释放，upgrade 就返回 None。实验会在释放父节点后验证子节点仍然存在，但其 parent_name 变成 None。

**练习**：把实验中 child 的 parent 字段改成 `Rc<TreeNode>`，思考这是否能编译、会产生什么所有权结构；恢复为 Weak 后，再增加 `parent_is_alive` 测试。不要通过泄漏内存来“验证”循环，使用 strong_count 与 Drop 观察器说明生命周期。

## 总结：如何选择

- 需要单一所有者和堆间接层：先考虑 Box。
- 需要自定义类型表现得像指针：谨慎实现 Deref/DerefMut。
- 需要作用域结束时清理资源：使用 Drop/RAII。
- 需要单线程多个所有者共享不可变数据：考虑 Rc。
- 需要在共享引用下修改内部状态：评估 RefCell 的运行时借用风险。
- 需要对象图中的非拥有回边：使用 Weak，避免强引用循环。
- 需要跨线程共享：比较 Arc、Mutex、RwLock，而不是把 Rc/RefCell 直接搬到多线程。

## D2 验收证据

| 小节 | 最低可复现证据 |
|---|---|
| 15.1 | 递归 List 的大小间接层、Box 数值与 trait object 测试 |
| 15.2 | MyBox 的 Deref/DerefMut、两级 coercion 测试 |
| 15.3 | 自动 Drop 与显式 drop 的计数测试 |
| 15.4 | Rc clone、strong_count 增减测试 |
| 15.5 | 多个共享借用、冲突可变借用、Rc<RefCell> 测试 |
| 15.6 | Weak upgrade 成功/失败、父子图生命周期测试 |

自动化通过证明参考代码的行为合同，不证明学员已掌握。请在自己的副本中完成每节练习、添加至少一条测试，并在 `LEARNING_LOG.md` 记录预测、实际结果和解释。

官方资料：[Box](https://doc.rust-lang.org/book/ch15-01-box.html) · [Deref](https://doc.rust-lang.org/book/ch15-02-deref.html) · [Drop](https://doc.rust-lang.org/book/ch15-03-drop.html) · [Rc](https://doc.rust-lang.org/book/ch15-04-rc.html) · [RefCell](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html) · [Reference Cycles](https://doc.rust-lang.org/book/ch15-06-reference-cycles.html)。
