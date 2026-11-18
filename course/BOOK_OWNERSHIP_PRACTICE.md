# D6 · 所有权、结构体方法与字符串切片

本课补充原书第 4–6 章。每轮按 **预测 → 运行 → 修改个人副本 → 解释** 的顺序练习。打开 [ownership.rs](../projects/book-d6-foundations/foundation-lab/src/ownership.rs) 阅读 `ReadingNote` 和文档实验，再打开 [tests/ownership.rs](../projects/book-d6-foundations/foundation-lab/tests/ownership.rs) 查看输入和断言。做修改练习时，把整个 `projects/book-d6-foundations` 复制到自己的目录，保留工作区清单、锁文件、源码和测试。

从源码根目录运行以下命令。针对个人副本运行时，把清单路径替换为副本的 `Cargo.toml` 路径。

```powershell
cargo test --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab --test ownership
cargo test --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab --doc ownership
```

第一条检查公开 API 的行为，第二条运行本模块的文档实验。标记为 `compile_fail` 的实验显示 `ok`，表示编译器拒绝了这段用法。若要亲眼看到诊断，先在个人副本的 `foundation-lab` 中创建 `examples` 文件夹（已有则直接使用），再创建 `ownership_probe.rs`，将相应代码块放进文件的 `fn main() { ... }` 中，然后运行：

```powershell
cargo check --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab --example ownership_probe
```

这条命令里的清单路径也要指向个人副本。一次只试一个反例，记下诊断编号和指向的语句，再换成它的成功对照重跑。

## 1. 结构体更新、部分移动和方法接收者

先预测 `let next = ReadingNote { minutes: 30, ..base };` 之后，`base.title`、`base.minutes` 和 `base.completed` 哪些还能使用。运行测试，再试着对 `base` 调用 `title()`。结构体更新使用同一类型的基值。例中 `minutes` 由新值显式提供，`completed` 是从基值复制的 `bool`，`title` 是从基值移动出来的 `String`。部分移动之后还能访问未移动字段，借用整个结构体则会得到 E0382。

在个人副本中显式写 `title: base.title.clone()`，然后给新笔记的标题追加文字。再把 `base.minutes` 赋给一个可变局部变量，修改这个变量。用断言确认两次修改都没有改变原笔记。解释 `u32` 赋值何时隐式 `Copy`，以及 `String::clone` 怎样得到独立的拥有值。

找到定义 `new` 和定义三个方法的两个 `impl ReadingNote` 块。预测以下调用之后还能对笔记做什么：

| 调用 | 接收者 |
| --- | --- |
| `ReadingNote::new(String::from("Rust"), 20)` | 关联函数，没有 `self` |
| `note.title()` | 自动共享借用 `&self`，返回标题切片 |
| `note.append_to_title(" Book")` | 自动独占借用 `&mut self`，需要可变笔记 |
| `note.into_title()` | 按值接收 `self`，消耗笔记并返回拥有的标题 |

把一个方法调用改写成 `ReadingNote::title(&note)` 或 `ReadingNote::append_to_title(&mut note, " Book")`，运行并比较。把 `into_title()` 移到第一次读取标题之前，观察新报错并解释哪个值被移动。

用 `let nested_borrow = &&note;` 取得 `&&ReadingNote`，再调用 `nested_borrow.title()`。方法查找会自动解引用，直到找到接受 `&ReadingNote` 的方法。共享引用仍然无法提供可变接收者。

先预测 `String::with_capacity(16)` 之后追加 `"中🦀"` 时，`len()` 和 `capacity()` 满足哪些条件。运行文档实验，比较移动前后 `as_ptr()` 取得的缓冲区地址和 `len()`。容量只检查至少为 16。这个实验只比较堆缓冲区地址，没有解引用裸指针。解释观察结果时只描述缓冲区，Rust 的内存布局和 ABI 由这个实验无法确定。

找到接收 `String` 的 `consume` 和接收 `&str` 的 `bytes`，先预测两次调用各由谁拥有字符串。把个人副本里的输入改成 `"中🦀"`，先算出 UTF-8 字节数，再更新长度与文本断言并运行。说明为什么 `consume` 需要把字符串作为元组第一项交回，而 `bytes` 返回长度后调用者仍然可以使用原字符串。

## 2. 借用看到最后一次使用

先预测两个 `&mut text` 都留在同一块里时，怎样移动一行才能让代码通过。运行 E0499 反例和对照。将第一次引用的最后一次使用移到第二次借用之前，非词法生命周期（NLL）允许在同一代码块内开始第二次借用。

接着试 E0596 和 E0502。E0596 的例子通过 `&note` 调用需要 `&mut self` 的方法。E0502 的例子在 `text.clear()` 之后再次使用共享切片，导致清除时共享借用仍然有效。分别试着改走独占借用和先使用切片再清除，解释两种错误各发生在哪一行。

最后预测返回局部 `String` 的引用是否能通过。运行 E0515 反例，再改为返回 `String`。说明局部拥有者何时被释放，以及按值返回如何把所有权交给调用方。

## 3. 字节索引、切片与 `Option`

先给 `a中🦀` 标出 UTF-8 字节边界 0、1、4、8，再分别预测 `preview(text, 1, 4)`、`preview(text, 2, 4)`、`preview(text, 8, 8)` 和 `preview(text, 4, 1)` 的结果。运行测试后，加入一个由你选择的合法范围和一个落在字符中间的范围。`preview` 用字节下标检查半开区间 `start..end`，在合法边界上的空区间返回 `Some("")`。倒序、越界或端点位于字符内部时返回 `None`。

在文档示例中对照 `&text[start..end]`、`&text[..end]`、`&text[start..]` 和 `&text[..]`。省略起点时从 0 开始，省略终点时到字节长度为止。直接切片要求范围合法，遇到非法范围会 panic，可以用 `preview` 检查来自输入的范围。

运行保存旧字节偏移的示例。在个人副本中将替换文本改为 `"Rustacean Book"`，同时按实际新输入更新“旧偏移预览”和“当前首词”两条断言：旧偏移仍为 4，预览是 `Some("Rust")`，当前首词是 `"Rustacean"`。观察旧偏移恰好合法时，预览仍然可能截到当前词的一部分。对照重新调用 `first_word(&text)` 的结果，解释为什么整数偏移会过时。再试带全角空格或制表符的输入，确认 `first_word` 跳过起始空白，按 Unicode 空白分词，空输入和仅空白输入都得到空切片。

先把 `Option<usize>` 直接赋给 `usize`，观察 E0308，再用 `match` 取出 `Some(length)` 的长度并给 `None` 指定返回值。试着修改 `None` 的返回值，更新个人副本中的预期值后运行。

给独立的 `None` 标注 `Option<&str>`，比较 E0282 反例与成功对照。接着删掉 `match preview(...)` 的 `None` 分支，观察 E0004，再补回并换用非法范围实际运行该分支。用自己的话分别解释“编译器怎样确定 `Option<T>` 里的 `T`”和“编译器怎样确认分支都覆盖了”。

第 6 章的枚举携带数据与模式匹配可以继续读 D3 的 [`Message`](../projects/book-d3-oop-patterns/oop-pattern-lab/src/lib.rs#L15)。对照 D4 的 [`Millimeters` 与 `Meters`](../projects/book-d4-advanced-features/advanced-lab/src/lib.rs#L254)，说明两个元组结构体如何把同为 `u32` 的值区分成不同单位类型。
