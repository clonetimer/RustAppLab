# Rust Book 31 项残余概念 · 可运行教学链

本课只处理 [2026-11-18 审计](BOOK_CONCEPT_AUDIT.md)中已确认未完整的 **31 个稳定目标**，范围仍为第 4–6、8–10 章的 **118 个抽样概念群**。原始审计保留不动；新增逐项证据见 [2026-11-24 补课记录](../verification/concept-repair-20261124.json)。它表示教材提供了讲解、可运行对照和迁移任务，不表示学习者已经掌握，也不表示整本 Book 全覆盖。

## 开始练习

在个人副本中打开 [concept_repairs.rs](../projects/book-d6-foundations/foundation-lab/src/concept_repairs.rs)，用下面每节的审计 ID 查找同名源码标题。每节先写预测，运行正例，再单独尝试反例；修复后换一组自己的输入。普通代码块由 rustdoc 实际执行；compile_fail 通过表示该片段确实被编译器拒绝。额外的集成测试还核对具体错误编号，避免把无关语法错误误算成目标规则的证据。

从源码仓库根目录执行（使用个人副本时替换 manifest 路径）：

```powershell
cargo test --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml --doc concept_repairs
cargo test --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml --test concept_repairs -- --nocapture
```

第二条会显示真实诊断观察。panic/backtrace 示例的单独运行预期可能失败；集成检查只有观察到约定的失败及正确修复对照才通过。源文件包含可复制的小程序；想直接看编译器完整诊断时，把一个反例放进个人副本的 examples/own_probe.rs 的 `fn main() { ... }`，再执行：

```powershell
cargo check --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml --example own_probe
```

每次只保留一个反例。完成后移走或修复 own_probe.rs，避免故意失败的个人实验影响全部 targets 的正式检查。不要把编辑教师参考工程当成自己的作业，也不要把本课参考检查的绿色结果记录为个人理解成绩。

## 逐项补课

### 4.1-03 · 字符串字面值与可增长String、String::from和push_str

- **讲解**：字符串字面量借用程序中的不可变 UTF-8 数据；String::from 创建拥有的可增长缓冲区。mut 只改变绑定允许的操作，不会把 &str 变成 String。
- **先预测**：把字面量转成 String 后给 String 追加 " Book"，原字面量会变吗？给 &str 加上 mut 能调用 push_str 吗？
- **运行成功对照**：运行本节正例：字面量仍为 Rust，拥有的值为 Rust Book，两个断言都通过。
- **运行反例／观察失败**：对 &str 调用 push_str，实际编译得到 E0599；这个方法属于可增长 String，mut 不能补上它。
- **修复**：用 String::from(literal) 建立可变拥有值，再调用 push_str；保留原 &str 做对照。
- **迁移与解释**：换成“学习🦀”并追加中文，解释拥有数据、共享引用与可变绑定各自控制什么。

执行证据：`concept_repairs.rs` 的 `4.1-03` 段。原书：[对应小节](https://rust.bootcss.com/ch04-01-what-is-ownership.html)。

### 4.1-06 · Copy与Clone区别、标量/全Copy元组可复制、含String不可Copy

- **讲解**：全体字段都是 Copy 的元组可以隐式复制；含 String 的元组不能 Copy。赋值和按值传参都遵守同一规则，clone 是显式取得独立拥有值。
- **先预测**：(u32,bool) 与 (String,bool) 赋值或传参后，原元组分别还能读哪些字段？clone 后修改标题会影响原值吗？
- **运行成功对照**：正例在赋值、函数调用后继续使用标量元组；对 String 元组克隆并修改后，两份标题分别为 Rust 与 Rust!。
- **运行反例／观察失败**：移动含 String 的元组后读取 original.0，编译器给出 E0382。
- **修复**：需要两份拥有值时显式 clone；只需读取时优先借用，不能为了消除报错把所有地方都克隆。
- **迁移与解释**：在元组中再加一个 u64，然后把 bool 换成另一个 String，先预测 Copy 能力是否变化。

执行证据：`concept_repairs.rs` 的 `4.1-06` 段。原书：[对应小节](https://rust.bootcss.com/ch04-01-what-is-ownership.html)。

### 4.1-07 · 函数传参可移动或复制，返回值把所有权交回调用者

- **讲解**：普通函数按值接收 String 会移动，按值接收 u32 会复制。函数返回 String 可把所有权交回，调用者需接住返回值；旧绑定不会自动恢复。
- **先预测**：两个函数分别接收 String 和 u32，调用后打印原变量会怎样？返回 String 后是否要重新绑定？
- **运行成功对照**：正例调用 number 后 score 仍为 7；title 接收并返回 String，重新绑定后得到 Rust!。
- **运行反例／观察失败**：consume(title) 后再打印 title，得到 E0382。
- **修复**：若需继续拥有同一值，返回它并 let title = title_fn(title)；若函数只需读，改为 &str 参数。
- **迁移与解释**：写一个按值接收 (String,u32) 并返回它的函数；解释“把值返回”与“把引用返回”的调用者责任。

执行证据：`concept_repairs.rs` 的 `4.1-07` 段。原书：[对应小节](https://rust.bootcss.com/ch04-01-what-is-ownership.html)。

### 4.2-07 · 多个共享借用可共存，限制是为了阻止冲突读写

- **讲解**：多个同时活跃的共享引用允许共存，因为它们都只读。可变引用要求独占；仍会使用的共享引用与写入重叠时必须拒绝，避免冲突读写。
- **先预测**：两个 &String 一起读取是否成功？把 &mut 放在它们最后一次使用之前与之后，有何区别？
- **运行成功对照**：正例同时使用两个共享引用得到总长度 8；其最后一次使用之后才建立可变引用，追加 ! 成功。
- **运行反例／观察失败**：先建立 &mut 并修改，再打印两个共享引用，得到 E0502。
- **修复**：把共享读取放到修改之前，使借用的实际使用范围不重叠；不要用猜测内存是否重新分配作为放宽理由。
- **迁移与解释**：把两个共享引用交给不同只读函数，并改变最后一次使用的位置，记录哪个版本被拒绝。

执行证据：`concept_repairs.rs` 的 `4.2-07` 段。原书：[对应小节](https://rust.bootcss.com/ch04-02-references-and-borrowing.html)。

### 4.3-05 · 字符串字面值本身是&str且不可变

- **讲解**：字符串字面量的类型是 &'static str，底层数据在程序存活期间有效且不能经共享引用修改。mut 绑定能改指向，不等于字符串字节可写。
- **先预测**：let mut literal: &str 能否重新指向另一个字面量？能否调用需要 &mut str 的 make_ascii_uppercase？
- **运行成功对照**：正例把绑定由 Rust 改为 Book，再转换为 String 后变成 BOOK；字面量本身仍为 Book。
- **运行反例／观察失败**：直接对 &str 调用 make_ascii_uppercase，得到 E0596。
- **修复**：重新绑定引用可以改变它指向的文本；需要修改内容时创建 String 并通过可变借用操作。
- **迁移与解释**：分别解释 &'static str 与 String 的所有者；不要把返回局部 String 的引用标成 static 来延长数据寿命。

执行证据：`concept_repairs.rs` 的 `4.3-05` 段。原书：[对应小节](https://rust.bootcss.com/ch04-03-slices.html)。

### 5.1-02 · 修改字段需要可变的整个实例

- **讲解**：直接赋值给普通结构体字段要求整个实例绑定为 mut；Rust 没有在普通字段声明前单独加 mut 的机制。内部可变性是另一个机制。
- **先预测**：ReadingNote 的 minutes 字段公开，是否就能经不可变 note 赋值？
- **运行成功对照**：正例用 let mut note，minutes 从 20 改为 30，title 保持 Rust。
- **运行反例／观察失败**：把 mut 去掉后执行 note.minutes = 30，得到 E0594。
- **修复**：让确实需要改变的实例绑定为 mut，保留只读使用场景的不可变绑定。
- **迁移与解释**：再修改 completed 字段，并解释字段的 pub 与绑定的 mut 分别决定什么。

执行证据：`concept_repairs.rs` 的 `5.1-02` 段。原书：[对应小节](https://rust.bootcss.com/ch05-01-defining-structs.html)。

### 5.1-03 · 字段初始化简写

- **讲解**：初始化简写 title 等价于 title: title，前提是同名局部变量存在；局部名不同就要显式写 title: heading。两种拼写不改变移动规则。
- **先预测**：把 title 和 minutes 展开成长写法，值或所有权行为会改变吗？局部变量改名 heading 后还能保留 title 简写吗？
- **运行成功对照**：正例分别构造简写与展开版本，断言 title、minutes 相等。
- **运行反例／观察失败**：只有 heading 变量却写 ReadingNote { title, ... }，得到 E0425。
- **修复**：改为 title: heading，或把局部变量命名为 title 后使用简写。
- **迁移与解释**：在自己的构造函数中展开全部简写，再恢复简写；说明编译后行为为何应一致。

执行证据：`concept_repairs.rs` 的 `5.1-03` 段。原书：[对应小节](https://rust.bootcss.com/ch05-01-defining-structs.html)。

### 5.1-06 · 类单元结构体无字段，可承载行为且不是()

- **讲解**：struct Marker; 声明无字段的类单元结构体，仍是独立的命名类型，可以实现方法或 trait；它不是单元类型 ()。同为零大小不表示类型相同。
- **先预测**：Marker 和 () 的大小可以相同吗？把 () 赋给 Marker 会通过吗？
- **运行成功对照**：正例构造 Marker 并调用 label 得到 ready，分别观察两个值的大小为 0。
- **运行反例／观察失败**：let marker: Marker = (); 得到 E0308。
- **修复**：以 Marker 构造该类型；以 () 表示没有有意义返回值，按接口需要选择。
- **迁移与解释**：给 Marker 再实现一个只读方法，再声明另一个无字段结构体，预测两种类型能否互相赋值。

执行证据：`concept_repairs.rs` 的 `5.1-06` 段。原书：[对应小节](https://rust.bootcss.com/ch05-01-defining-structs.html)。

### 5.2-01 · 从独立变量/元组到具名字段，表达相关数据的设计理由

- **讲解**：松散参数与元组依赖读者记忆位置含义；具名字段把相关数据归组，并在调用处表达含义。字段名提升可读性，但不会自动阻止把同类型的值填错字段。
- **先预测**：total=10、done=2 的三种接口都应输出什么？交换元组中的两个 u32 会编译失败还是产生含义错误？
- **运行成功对照**：正例的松散参数、元组、Progress 三版都输出 2/10。
- **运行反例／观察失败**：故意传 tuple((2,10))，程序仍编译，却与具名模型输出不同；assert_ne 定向验证了这个语义反例。
- **修复**：按 total、done 的含义恢复元组次序，并用 progress.total、progress.done 显式构造参数。
- **迁移与解释**：为任务卡添加 deadline，比较修改三版接口的调用处；说明字段名解决哪些阅读问题、哪些值域问题仍需校验。

执行证据：`concept_repairs.rs` 的 `5.2-01` 段。原书：[对应小节](https://rust.bootcss.com/ch05-02-example-structs.html)。

### 5.3-04 · 方法额外参数可借用另一结构体

- **讲解**：方法的额外参数可以借用另一个结构体。takes_longer_than(&self, other: &ReadingNote) 同时借用两个非 Copy 值，使调用者保留双方所有权。
- **先预测**：把 other: &ReadingNote 改为 other: ReadingNote，调用后 second 是否仍可用？
- **运行成功对照**：正例比较 30 与 20 分钟，随后继续读取两个笔记的 minutes 和 title。
- **运行反例／观察失败**：反例使用按值 other: Note，调用后读取 second.minutes 得到 E0382。
- **修复**：把额外参数改为 &Note，并在调用处传 &second；只读比较无需拿走对象。
- **迁移与解释**：写一个借用两个笔记标题的比较方法，验证调用后仍能分别修改两个笔记。

执行证据：`concept_repairs.rs` 的 `5.3-04` 段。原书：[对应小节](https://rust.bootcss.com/ch05-03-method-syntax.html)。

### 6.1-03 · 枚举可用impl定义有接收者的方法

- **讲解**：枚举可通过 impl 定义带接收者的方法，方法能根据变体及载荷返回结果。new() 不带 self，是通过类型调用的关联函数。
- **先预测**：Draft 与 Reviewed { minutes:20 } 调用 minutes() 各得到什么？draft.new() 是否是合法构造方式？
- **运行成功对照**：正例断言 Draft 得到 None，Reviewed 得到 Some(20)，并验证类型限定的方法写法等价。
- **运行反例／观察失败**：把构造函数写成 draft.new()，得到 E0599。
- **修复**：用 LessonState::new() 构造实例，再用实例.minutes() 调用接收者方法。
- **迁移与解释**：新增一个需要载荷的变体，在方法内部完整处理它，并解释关联函数为何不依赖某个现有实例。

执行证据：`concept_repairs.rs` 的 `6.1-03` 段。原书：[对应小节](https://rust.bootcss.com/ch06-01-defining-an-enum.html)。

### 6.3-03 · if let简洁性的代价是放弃穷尽检查，字面模式只选一个值

- **讲解**：if let Some(3) 只选择值恰为 3 的那个模式，简洁的代价是不强制穷尽处理。新增枚举变体时，if let 的其他情况可静默落入 else，完整 match 必须补分支。
- **先预测**：Some(3)、Some(4)、None 分别能匹配吗？加上 Archived 后哪种写法要求修改？
- **运行成功对照**：正例得到 [true,false,false]；包含 Archived 的 if let 仍可编译，修复后的完整 match 返回 archived。
- **运行反例／观察失败**：match 只覆盖 Draft、Done 却新增 Archived 时，得到 E0004。
- **修复**：需要逐一审查所有情况时用完整 match 并补分支；只关心一种模式时用 if let，并明确接受其余情况被忽略。
- **迁移与解释**：把模式从 Some(3) 改为 Some(value)，预测 Some(4) 的变化，再新增一个枚举变体比较两种控制流。

执行证据：`concept_repairs.rs` 的 `6.3-03` 段。原书：[对应小节](https://rust.bootcss.com/ch06-03-if-let.html)。

### book-8-1-vec-drop-elements · Vec 离开作用域同时释放所拥有的元素

- **讲解**：Vec 拥有其元素；离开作用域时，会析构仍保存在其中的元素并释放自身分配。pop 返回的元素已经转移给调用者，不随原 Vec 一起析构；不依赖元素析构顺序。
- **先预测**：三个带 Drop 计数器的元素先 pop 一个，Vec 离开作用域后计数是 2 还是 3？
- **运行成功对照**：正例先观察 0，Vec 离开作用域后为 2，最后 drop 取出的元素后为 3。
- **运行反例／观察失败**：“pop 已经销毁元素”的错误假设被 assert_ne(count,3) 否定；计数器说明移出与销毁是两个动作。
- **修复**：跟踪元素的当前所有者；对移出的值在其自身生命周期结束或显式 drop 时再判断析构。
- **迁移与解释**：先 clear Vec 再离开作用域，预测每个观察点的计数；不要把本次观察到的元素顺序写成 API 保证。

执行证据：`concept_repairs.rs` 的 `book-8-1-vec-drop-elements` 段。原书：[对应小节](https://rust.bootcss.com/ch08-01-vectors.html)。

### book-8-2-string-construction · String::new/from 与 to_string 的创建方式

- **讲解**：String::new() 创建空串；String::from(text) 与 text.to_string() 对这段 &str 创建内容相同的拥有字符串。new 后追加同一文本也得到相同内容，不保证分配过程相同。
- **先预测**：new 是否接收初始文本？三种构造最终内容能否相同，开始时是否都非空？
- **运行成功对照**：正例先断言 new 为空，追加“中 Rust”后与 from、to_string 相等，也比较了空文本情况。
- **运行反例／观察失败**：String::new("Rust") 得到 E0061，因为 new 没有这个参数。
- **修复**：空串后续累积时用 new；已有文本需要拥有副本时用 from 或 to_string。
- **迁移与解释**：把文本换为空串和 emoji；解释 to_string 的一般 trait 能力与本例 &str 转换之间的关系。

执行证据：`concept_repairs.rs` 的 `book-8-2-string-construction` 段。原书：[对应小节](https://rust.bootcss.com/ch08-02-strings.html)。

### book-8-2-string-integer-index · 整数索引不支持及 UTF-8/O(1) 原因

- **讲解**：UTF-8 每个 Unicode scalar 的字节数不同，整数索引还存在“字节、scalar、显示字符”的歧义。直接索引通常期望 O(1)，从头找第 n 个 scalar 需扫描解码；chars 不等于字形簇。
- **先预测**：A中🦀 的字节数与 chars 数各多少？chars().nth(2) 前会遍历几个 scalar？
- **运行成功对照**：正例得到 8 字节、3 scalar，inspect 记录 nth(2) 经过 3 个 scalar；合法字节范围返回中。
- **运行反例／观察失败**：String[1] 得到 E0277；get(1..2) 则安全返回 None，暴露非法 UTF-8 边界。
- **修复**：按需求明确选 as_bytes、chars 或边界有效的范围；处理外部字节下标时用 get 检查。
- **迁移与解释**：换成组合附加符文本，说明 chars 计数为何仍未必等于用户看到的字数；不要用本例推出字符串随机字符访问是 O(1)。

执行证据：`concept_repairs.rs` 的 `book-8-2-string-integer-index` 段。原书：[对应小节](https://rust.bootcss.com/ch08-02-strings.html)。

### book-8-3-hashmap-construction · HashMap<K,V>、new/insert 与同质键值

- **讲解**：一个 HashMap<K,V> 的所有键共享 K，所有值共享 V，键和值彼此可以是不同类型。显式注解或插入约束确定类型后，后续不能混入另一种键或值。
- **先预测**：HashMap<String,u32> 能否插入数字键或字符串分数？重复键会新增一项还是替换？
- **运行成功对照**：正例 new/insert 后读出 Ada 的分数，第二次 insert 返回旧值 Some(7)，查询得到 8。
- **运行反例／观察失败**：两个独立反例分别混入数值键和字符串值，都得到 E0308。
- **修复**：转换成约定的 String/u32，或在确实需要多种数据形状时设计一个明确的枚举类型。
- **迁移与解释**：去掉类型注解，让第一条 insert 决定 K/V，再交换插入次序并解释诊断位置为何改变。

执行证据：`concept_repairs.rs` 的 `book-8-3-hashmap-construction` 段。原书：[对应小节](https://rust.bootcss.com/ch08-03-hash-maps.html)。

### book-8-3-hashmap-collect · zip 后 collect 为 HashMap 与类型注解

- **讲解**：zip 产生键值对迭代器，collect 依据目标类型构造容器；HashMap<_,_> 为无法推断的目标提供信息。zip 在较短一边结束，不能假设两边所有元素都进入结果。
- **先预测**：不写目标类型时编译器怎么知道要 HashMap？两键一值时 map 有几项？
- **运行成功对照**：正例验证 Ada→3、Lin→5，并检查两键一值时只有一项且 Lin 查询为 None。
- **运行反例／观察失败**：省略 collect 的目的类型后调用 get，得到 E0282。
- **修复**：写 let map: HashMap<_,_> = ...collect()，或使用 collect::<HashMap<_,_>>()。
- **迁移与解释**：让键重复，预测哪个值保留；业务要求键值数量一致时，在 zip 之前增加长度校验并解释原因。

执行证据：`concept_repairs.rs` 的 `book-8-3-hashmap-collect` 段。原书：[对应小节](https://rust.bootcss.com/ch08-03-hash-maps.html)。

### book-8-3-hashmap-ownership · Copy、String move、引用键值的有效期

- **讲解**：insert 按值接收 K/V：Copy 标量复制后原绑定仍可用，String 移动后旧绑定失效。引用键值只借用数据，其拥有者必须覆盖 map 的使用期。
- **先预测**：分别插入 u32、String 和 &str 后再读原值，哪种会报错？引用的 String 提前离开作用域会怎样？
- **运行成功对照**：正例保留 Copy 键值，显式 clone String 后保留原名称，并在有效作用域中查询借用的键和值。
- **运行反例／观察失败**：直接移动 String 后使用它得到 E0382；让借用键的拥有者先销毁再用 map 得到 E0597。
- **修复**：需要拥有两份时 clone；只读且生命周期合适时借用；让借用 map 的使用结束早于引用数据的销毁。
- **迁移与解释**：把 String 放到 value 一侧而不是 key 一侧，重做预测，证明所有权规则对 K/V 都适用。

执行证据：`concept_repairs.rs` 的 `book-8-3-hashmap-ownership` 段。原书：[对应小节](https://rust.bootcss.com/ch08-03-hash-maps.html)。

### book-8-3-hashmap-hasher · 默认哈希的取舍与 BuildHasher 扩展入口

- **讲解**：默认 RandomState 用随机种子帮助抵抗恶意碰撞攻击；特定哈希策略可能在某些负载上取速度而舍弃部分防护。BuildHasher 是创建 Hasher 的扩展入口；本课不测量速度，也不承诺哈希值或遍历顺序跨版本稳定。
- **先预测**：更换构建器后同键查询语义应变吗？能把任意 struct 当构建器吗？相同查询结果能否证明另一策略更快、更安全？
- **运行成功对照**：正例使用默认 map 与 BuildHasherDefault<DefaultHasher>，比较已存在键与缺失键的查询结果。
- **运行反例／观察失败**：把未实现 BuildHasher 的 NotABuilder 用于插入，得到 E0599；类型存在不等于满足所需 trait。
- **修复**：使用满足 BuildHasher 的构建器；本课程涉及外部输入时保留默认策略。演示用 BuildHasherDefault 缺少默认随机种子，不作为优化建议。
- **迁移与解释**：解释若要评估自选策略，还需怎样的输入分布、威胁前提和基准；禁止用固定哈希数值或遍历次序写跨版本测试。

执行证据：`concept_repairs.rs` 的 `book-8-3-hashmap-hasher` 段。原书：[对应小节](https://rust.bootcss.com/ch08-03-hash-maps.html)。

### book-9-1-panic-unwind-abort · panic 的栈展开与 abort 配置区别

- **讲解**：unwind 展开 Rust 栈帧并调用沿途活跃对象的 Drop，可由合适边界捕获；abort 直接终止进程，不运行这些析构，catch_unwind 无法拦住。Cargo 的 profile panic 配置是构建策略。
- **先预测**：同一探针分别以 panic=unwind、panic=abort 编译后，哪个能到达 caught unwind？哪个会创建析构哨兵文件？
- **运行成功对照**：集成测试真实编译两个独立程序：unwind 子进程输出 caught unwind，退出成功，哨兵文件内容为 guard dropped。
- **运行反例／观察失败**：abort 子进程真实非零/信号退出，仍能观察进入 panic 的输出，但无哨兵文件，也无 caught unwind；不能在测试主进程里演示 abort。
- **修复**：需要处理正常失败时返回 Result；需要讨论 unwind 时只在相应构建策略的独立进程内观察。不能把 catch_unwind 当成 abort 恢复方案。
- **迁移与解释**：在个人副本的 release profile 写 panic = "abort"，解释它只影响相应构建配置，以及为何本测试用 rustc -C 显式对比。

执行证据：`concept_repairs.rs` 的 `book-9-1-panic-unwind-abort` 段，以及集成测试 `unwind_runs_destructors_and_abort_does_not`。原书：[对应小节](https://rust.bootcss.com/ch09-01-unrecoverable-errors-with-panic.html)。

### book-9-1-panic-location · 主动 panic、索引触发 panic 与源位置定位

- **讲解**：主动 panic 和越界索引都会报告触发位置；应读取消息旁的 file:line:column，再打开自己的表达式。库帧不必然是错误输入的来源，要追溯调用者。
- **先预测**：同一示例的 explicit 与 index 模式，错误消息和源码行应该相同吗？
- **运行成功对照**：集成测试运行两个真实子进程，均捕获自身 panic_backtrace_probe.rs 的位置；正例改为 get 后合法输入 Some(7)、越界 None。
- **运行反例／观察失败**：explicit 模式报告主动消息；index 模式报告 index out of bounds，两者都预期非零退出。
- **修复**：根据真实行号定位触发表达式；外部下标改为 get 并处理 None，程序内部不变量则追查产生错误值的地方。
- **迁移与解释**：在个人副本中给示例插入空行，再读取新行号；不要把当前机器的固定行号当成判定规则。

执行证据：`concept_repairs.rs` 的 `book-9-1-panic-location` 段，以及集成测试 `explicit_and_index_panics_both_locate_own_source`。原书：[对应小节](https://rust.bootcss.com/ch09-01-unrecoverable-errors-with-panic.html)。

### book-9-1-panic-backtrace · RUST_BACKTRACE 与调用链定位

- **讲解**：RUST_BACKTRACE=1 请求输出调用链，=0 关闭它；用新的子进程分别观察，避免进程内环境缓存影响。沿栈从 panic 向外找最先相关的自身函数，再追到调用者。
- **先预测**：开启回溯是否改变程序失败事实？probe_inner、probe_middle、probe_outer 应如何出现在调用链？
- **运行成功对照**：集成测试两次实际运行：=0 无 stack backtrace 段；=1 含三个自身函数及源码位置，保留实际文本供阅读。
- **运行反例／观察失败**：两次示例均预期非零退出；回溯只增加诊断信息，不修复导致 panic 的输入。
- **修复**：按调用链找到引入错误假设的调用处并改输入检查/Result；本节正例 checked_step 对溢出返回 Err。
- **迁移与解释**：再加一层自己命名的调用函数并重跑，指出新帧；对比只有消息和完整调用链时能回答哪些不同问题。

执行证据：`concept_repairs.rs` 的 `book-9-1-panic-backtrace` 段，以及集成测试 `backtrace_switch_exposes_the_own_source_call_chain`。原书：[对应小节](https://rust.bootcss.com/ch09-01-unrecoverable-errors-with-panic.html)。

### book-9-2-result-error-kind · 根据 io::ErrorKind 区分缺失与其它失败

- **讲解**：恢复策略取决于业务：本例“可选笔记不存在”可使用空文本，PermissionDenied 等其他错误必须保留。必须文件的 NotFound 也可能要直接返回错误，不能套用统一默认值。
- **先预测**：不存在文件与权限不足都属于 Err，能否都用空串替代？怎样验证没有吞掉其他错误？
- **运行成功对照**：集成测试真实读取不存在文件并确认 NotFound，恢复为空；创建文件后读到 Rust。
- **运行反例／观察失败**：PermissionDenied 值经 optional_note 仍是 Err，kind 保持原值；此分支用确定性错误值测试，不声称操作系统真实拒绝了权限。
- **修复**：match error.kind() 只在 NotFound 分支应用已声明的恢复策略，其余返回原错误。
- **迁移与解释**：把业务改成“必须存在的配置”，重新决定 NotFound 分支；再测试 InvalidData，不允许吞成成功。

执行证据：`concept_repairs.rs` 的 `book-9-2-result-error-kind` 段，以及集成测试 `missing_file_recovery_does_not_swallow_other_io_errors`。原书：[对应小节](https://rust.bootcss.com/ch09-02-recoverable-errors-with-result.html)。

### book-9-2-result-unwrap-expect · unwrap/expect 的 panic 语义与诊断信息

- **讲解**：unwrap/expect 在 Some、Ok 时取出值；在 None、Err 时 panic。expect 增加调用者对前提的说明，不会把失败变成 Result，也不会自动恢复。
- **先预测**：在四种容器状态上分别调用两者会发生什么？expect 是否保留原 Err 的诊断并增加自定消息？
- **运行成功对照**：正例验证 Some/Ok 的 unwrap、expect 都取出正确值；以局部 catch_unwind 观察实验性 panic。
- **运行反例／观察失败**：None/Err 的四种失败均实际触发 panic，断言 unwrap 信息包含状态/错误，expect 信息同时含自定前提及原错误。
- **修复**：生产输入可能失败时用 match、? 或其他明确返回 Result 的分支；保留 expect 只用于有证据支持的前提。
- **迁移与解释**：换一个自定义错误文本，比较两种诊断；解释为什么把 unwrap 改成 expect 并没有消除 panic 风险。

执行证据：`concept_repairs.rs` 的 `book-9-2-result-unwrap-expect` 段。原书：[对应小节](https://rust.bootcss.com/ch09-02-recoverable-errors-with-result.html)。

### book-9-3-panic-prototype-assumption · 示例/测试/原型与已知正确值中的 unwrap/expect

- **讲解**：测试夹具、经过核对的固定字面量可用 expect 表达明确前提；原型可暂时让未实现的错误处理暴露出来。用户输入没有同样保证，应返回 Result 并验证失败分支。
- **先预测**："42" 解析为 u32 的依据是什么？若相同代码接收外部文本，原来的 expect 是否仍有依据？
- **运行成功对照**：正例的已审固定字面量解析为 42，普通解析函数对 43 返回 Ok。
- **运行反例／观察失败**：外部样式的 forty-two 返回 Err；强行用“原型假设”expect 则实际 panic，说明消息不能使假设成真。
- **修复**：把暂用 expect 的外部输入路径改为 Result，保留测试中对成功和失败的断言及前提说明。
- **迁移与解释**：判断固定 IP 字面量、用户提交 IP、测试夹具、线上配置四种情境各能否使用 expect，并为每个判断给出可检验的依据。

执行证据：`concept_repairs.rs` 的 `book-9-3-panic-prototype-assumption` 段。原书：[对应小节](https://rust.bootcss.com/ch09-03-to-panic-or-not-to-panic.html)。

### book-9-3-validated-types · 私有字段与受控构造保证有效状态

- **讲解**：私有字段和受控构造让安全的外部调用者不能绕过值域约束。ValidatedScore 只接受 0..=100，公开只读方法保持不变量；输入验证被放到 API 边界。
- **先预测**：-1、0、100、101 各能否构造？外部代码能否直接写 value:255 绕开 try_new？
- **运行成功对照**：正例接受边界 0/100 并读出值，构造 -1/101 都返回 Err。
- **运行反例／观察失败**：外部结构体字面量直接写私有 value 得到 E0451；失败值不能以该公开构造路径变为成功对象。
- **修复**：调用 try_new 并处理 Result，让消费端接收 ValidatedScore；新增任何可变方法也要维护范围。
- **迁移与解释**：设计 1..=5 的 Rating 类型，并解释增加一个不检查的 set 方法为何会破坏这份合同。

执行证据：`concept_repairs.rs` 的 `book-9-3-validated-types` 段。原书：[对应小节](https://rust.bootcss.com/ch09-03-to-panic-or-not-to-panic.html)。

### book-10-1-generic-monomorphization · 单态化与静态分发的基本机制

- **讲解**：编译器针对实际使用的具体类型实例化泛型操作；本例 u32、String 的调用在编译期确定对应类型。静态分发不等于任何泛型代码都更快或更小，多种实例可能增加代码体积。
- **先预测**：identity::<u32> 与 identity::<String> 的参数/返回类型分别是什么？能否把 String 版本当 fn(u32)->u32 使用？
- **运行成功对照**：正例直接调用 u32、String、bool 三种实例，断言返回各自类型和值。
- **运行反例／观察失败**：把 String 实例赋给 fn(u32)->u32，得到 E0308，显示具体化后的签名并不通用。
- **修复**：按需要实例化正确的 T，让编译期类型与调用者合同一致；动态分发需另有相应接口，不能由此自动获得。
- **迁移与解释**：添加一个自定义结构体实例，说明多出的类型实例为何不代表必然的可测加速；区分类型证据与基准证据。

执行证据：`concept_repairs.rs` 的 `book-10-1-generic-monomorphization` 段。原书：[对应小节](https://rust.bootcss.com/ch10-01-syntax.html)。

### book-10-2-trait-coherence · trait 或类型须为本地、孤儿规则与 newtype

- **讲解**：本例的非泛型 impl 至少需要本地 trait 或本地类型。Display 与 Vec 都来自外部，直接组合违反孤儿规则；本地 newtype 提供独立类型，type 别名没有这个作用。一般泛型 impl 还有覆盖/顺序限制。
- **先预测**：给 Vec<String> 直接实现 Display 会怎样？包装成 Labels(Vec<String>) 后为何不同？
- **运行成功对照**：正例为本地 Labels 实现 Display，实际格式化得到 [Rust, Book]。
- **运行反例／观察失败**：直接 impl Display for Vec<String> 得到 E0117。
- **修复**：用本地 newtype 承载行为，并显式决定要暴露哪些内部操作；不要把别名误认为新类型。
- **迁移与解释**：再写一个本地 trait 并为 Vec<String> 实现，比较它与“外部 trait＋外部类型”组合的差别。

执行证据：`concept_repairs.rs` 的 `book-10-2-trait-coherence` 段。原书：[对应小节](https://rust.bootcss.com/ch10-02-traits.html)。

### book-10-2-trait-copy-clone-borrow · 泛型取最大值中的 PartialOrd/Copy/Clone 与借用替代

- **讲解**：PartialOrd 可以返回不可比较的 None，例如 NaN；原 largest 系列选择更强的 Ord 全序约束。本节 partial 版本遇不可比较扫描就返回 None。Copy/Clone 决定能否复制/克隆拥有结果，借用返回则引用原元素。
- **先预测**：f64 的 NaN 与 2.0 能否比较？String 能用于 Copy、Clone、借用三种最大值函数中的哪些？
- **运行成功对照**：正例验证整数 Copy、String Clone 与引用地址来源，验证浮点正常值最大项和空输入。
- **运行反例／观察失败**：NaN 使 partial_cmp 和本节扫描返回 None；把 f64 交给 Ord 版本得到 E0277。
- **修复**：需要部分序时明确不可比较值策略；需要全序时选择满足 Ord 的模型。只读结果可借用，确需拥有结果再讨论复制/克隆成本。
- **迁移与解释**：把 NaN 放在首项、中间、唯一项各运行一次；设计带姓名的非 Copy 记录，解释克隆与返回引用的成本和寿命。

执行证据：`concept_repairs.rs` 的 `book-10-2-trait-copy-clone-borrow` 段。原书：[对应小节](https://rust.bootcss.com/ch10-02-traits.html)。

### book-10-2-trait-blanket-impl · 满足 bound 的类型统一获得 trait 实现

- **讲解**：impl<T: Display> Tagged for T 为每个满足 bound 的 T 提供同一实现，包括后来定义的新类型；它不是列举若干类型。再为已经覆盖的具体类型添加同 trait 实现会重叠。
- **先预测**：u32、String 与新定义但实现 Display 的 Ticket 是否自动获得 tag？还需单独 impl Tagged 吗？
- **运行成功对照**：正例验证数字、String 和后来定义的 Ticket 都得到预期的带括号文本。
- **运行反例／观察失败**：泛型 blanket impl 后再 impl Tagged for u32，得到 E0119。
- **修复**：保留一个不冲突的实现策略；若行为要区分，重新设计 trait/bound 或本地包装，而非堆叠重叠 impl。
- **迁移与解释**：新增不实现 Display 的类型，预测为何暂时没有 Tagged；实现 Display 后复测并解释能力从哪里获得。

执行证据：`concept_repairs.rs` 的 `book-10-2-trait-blanket-impl` 段。原书：[对应小节](https://rust.bootcss.com/ch10-02-traits.html)。

### book-10-3-lifetime-method-impl · 带生命周期结构的 impl 与方法独立借用

- **讲解**：impl<'a> Excerpt<'a> 的前一个参数声明名字、后一个把它用于类型；impl Excerpt<'_> 在无需显式使用该名字时匿名处理。两种写法不延长字段引用。方法的 &self 借用与临时 prefix 借用是独立关系。
- **先预测**：临时 prefix 销毁后两个方法结果还能用吗？若 Excerpt 实例先销毁，按 self 省略规则返回的引用还能继续用吗？
- **运行成功对照**：正例对原有占位 impl 的 without_prefix 与命名 impl 的 named_without_prefix 调用，临时 prefix 结束后结果仍相同且指向同一原文位置。
- **运行反例／观察失败**：让 excerpt 先离开作用域再使用 named_without_prefix 的结果，得到 E0597，即使原始 String 仍活着也不能忽视方法签名的关系。
- **修复**：让接收者覆盖结果的使用范围；若 API 确实返回字段的完整存活期，需要显式设计返回 &'a str，不能以注解假造更长寿命。
- **迁移与解释**：展开两个方法的 self/prefix 输入借用参数，分别写出输出跟随哪一个；对比能否把 prefix 的生命周期绑定到返回值。

执行证据：`concept_repairs.rs` 的 `book-10-3-lifetime-method-impl` 段。原书：[对应小节](https://rust.bootcss.com/ch10-03-lifetime-syntax.html)。

## panic 与回溯：亲自观察命令

下面两条均运行独立子进程；两次都预期非零退出，开启回溯不会修复程序。PowerShell：

```powershell
$env:RUST_BACKTRACE = "0"
cargo run --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml --example panic_backtrace_probe -- explicit
$env:RUST_BACKTRACE = "1"
cargo run --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml --example panic_backtrace_probe -- explicit
Remove-Item Env:RUST_BACKTRACE
```

Bash：

```bash
RUST_BACKTRACE=0 cargo run --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml --example panic_backtrace_probe -- explicit
RUST_BACKTRACE=1 cargo run --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml --example panic_backtrace_probe -- explicit
```

把最后的 explicit 改为 index，观察越界表达式的位置。不要抄固定行号：在自己的源码中找 probe_inner、probe_middle、probe_outer 三帧，并说明每层把什么输入传给下一层。

unwind/abort 对照使用集成测试 `unwind_runs_destructors_and_abort_does_not`：它在唯一临时目录中以 `rustc -C panic=unwind` 与 `-C panic=abort` 编译同一个 [panic_strategy_probe.rs](../projects/book-d6-foundations/foundation-lab/examples/panic_strategy_probe.rs)，观察实际退出状态及 Drop 哨兵文件。此方式不改变正式工作区的 profile，也不让 abort 终止测试宿主。若在个人副本研究 Cargo 配置，`panic = "abort"` 应置于适当的 `[profile.release]` 下；不要把运行时环境变量与编译策略混淆。

## 参考与验收边界

Hasher 的取舍核对 [HashMap 标准库说明](https://doc.rust-lang.org/std/collections/struct.HashMap.html)与 [BuildHasherDefault](https://doc.rust-lang.org/std/hash/struct.BuildHasherDefault.html)；panic 策略核对 [Rust Reference](https://doc.rust-lang.org/reference/panic.html)与[析构规则](https://doc.rust-lang.org/reference/destructors.html)；字符串索引的复杂度核对[官方 Book 字符串一节](https://doc.rust-lang.org/book/ch08-02-strings.html)。实验保留 Rust 1.74 / Edition 2021 基线，源码未采用只在较新版本可用的 API。

“31 项教学链已提供”和“学员独立完成 31 项”是两种记录。个人记录仍需预测、自己的修改、命令/退出码/关键诊断及迁移解释，按 [学习效果观察协议](LEARNER_EVIDENCE_PROTOCOL.md)登记。没有真实学员材料时，个人掌握度保持“未评估”。最终平台、浏览器和 CI 验收只引用各自实际运行的结果。
