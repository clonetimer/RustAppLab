# Rust Book 31 项残余概念 · 定向补课与验收清单

来源：[2026-11-18 核心概念审计](BOOK_CONCEPT_AUDIT.md)。此处 **31 项只属于第 4–6、8–10 章的 118 个抽样概念群**，不是全书欠账，也不是学员掌握率。此清单是新增练习任务，不能直接改写历史审计为已完成。

## 学员通用操作

1. 找到对应专题源码（[所有权](BOOK_OWNERSHIP_PRACTICE.md)、[集合错误](BOOK_COLLECTIONS_ERRORS.md)、[泛型生命周期](BOOK_GENERICS_LIFETIMES.md)）。在独立副本中操作，不修改教师参考代码。
2. 对每项先写预测，再制造题目指定的成功或错误，保存命令、实际退出码与关键诊断。
3. 修改一处后重新测试，并写下规则和自己的反例。**只读讲义/通过参考测试均不算学员完成。**

## 31 项逐项练习

| 审计 ID | 知识点 | 预测、修改与观察任务 |
|---|---|---|
| `4.1-03` | String 与 &str | 建立字符串字面量、String::from 和 push_str 三组变量，比较拥有数据和借用数据的区别。 |
| `4.1-06` | Copy 与 Clone | 尝试复制 (u32,bool) 和包含 String 的元组；记录哪组需显式 clone。 |
| `4.1-07` | 函数参数与所有权 | 让函数分别接收 String 和 u32，调用后再使用原值，记录移动和复制差异。 |
| `4.2-07` | 多共享引用 | 同时创建两个活跃的 &String 再读取；随后尝试在其中插入 &mut 引用。 |
| `4.3-05` | 字面量是 &str | 标注 let text: &str = "..."；说明字面量与可增长 String 的区别。 |
| `5.1-02` | 字段与 mut | 先对不可变 struct 直接改字段，记录失败；将整体绑定改为 mut 后重复。 |
| `5.1-03` | 字段初始化简写 | 比较字段名与局部变量同名时的 title 和 title: title 写法。 |
| `5.1-06` | 类单元结构体 | 定义 struct Marker; 实现方法，并与 () 的类型及行为比较。 |
| `5.2-01` | 结构体设计理由 | 分别用松散变量、元组、具名字段表示一张任务卡；解释修改字段的可读性。 |
| `5.3-04` | 方法借用其他结构体 | 写 fn compare(&self, other: &Note)，调用后验证两个 Note 均未移动。 |
| `6.1-03` | 枚举方法 | 为 enum Status 定义 impl 方法，并区分 Status::new 与实例方法。 |
| `6.3-03` | if let 的非穷尽性 | 分别匹配 Some(3)、Some(4)、None；对照 match 如何要求覆盖所有分支。 |
| `book-8-1-vec-drop-elements` | Vec 与释放 | 在测试对象 Drop 中记录释放顺序，检查 Vec 离开作用域后元素也析构。 |
| `book-8-2-string-construction` | String 构造 | 对同一文本执行 String::new + push_str、String::from、to_string，并比较结果。 |
| `book-8-2-string-integer-index` | 字符串索引复杂度 | 用 Unicode 文本观察 bytes/chars，解释第 n 个字符为何不能依赖 O(1) 整数索引。 |
| `book-8-3-hashmap-construction` | HashMap 键值类型 | 对比 new/insert，故意把 String 键和数值键混用，记录类型错误。 |
| `book-8-3-hashmap-collect` | zip + collect | 使用 keys.zip(values).collect::<HashMap<_,_>>() 构造映射，并验证两个键；见 book_gap4.rs。 |
| `book-8-3-hashmap-ownership` | Map 插入所有权 | 分别插入 String 和 Copy 数值，检查插入后原绑定是否仍可用。 |
| `book-8-3-hashmap-hasher` | BuildHasher | 在默认 HashMap 与显式 BuildHasherDefault 下查询相同键；不假定哈希数值稳定。 |
| `book-9-1-panic-unwind-abort` | unwind 与 abort | 用 catch_unwind 观察 unwind；另在隔离进程比较 panic=abort，不能在测试进程内执行 abort。 |
| `book-9-1-panic-location` | panic 源码定位 | 触发一次显式 panic 和索引 panic，记录自己的源码文件和行号。 |
| `book-9-1-panic-backtrace` | RUST_BACKTRACE | 运行 panic_backtrace_probe，先禁用再开启 backtrace，对比调用栈和自身代码位置。 |
| `book-9-2-result-error-kind` | I/O 错误分类 | 对不存在文件区分 NotFound 和 PermissionDenied 等错误，不把所有失败吞成默认值。 |
| `book-9-2-result-unwrap-expect` | unwrap 与 expect | 分别在 Some/None 和 Ok/Err 上调用 unwrap/expect，解释诊断信息与 panic 风险。 |
| `book-9-3-panic-prototype-assumption` | 已知前提下的 unwrap | 写出断言支持的已知正确值，再说明为何普通外部输入仍应返回 Result。 |
| `book-9-3-validated-types` | 合法状态构造 | 将字段设为 private，只经 try_new 校验，证明调用者不能构造无效状态。 |
| `book-10-1-generic-monomorphization` | 单态化 | 分别以 u32、String 实例化泛型函数，描述编译期具体化与静态分发。 |
| `book-10-2-trait-coherence` | 孤儿规则 | 尝试给外部类型实现外部 trait，再改用本地 newtype 实现。 |
| `book-10-2-trait-copy-clone-borrow` | Ord/PartialOrd 与借用 | 比较整数全序和含 NaN 浮点部分有序，解释值克隆与引用返回的取舍。 |
| `book-10-2-trait-blanket-impl` | Blanket impl | 给满足 Display 的 T 统一实现本地 trait，再添加另一具体类型验证。 |
| `book-10-3-lifetime-method-impl` | 生命周期 impl | 分别写 impl<'a> Excerpt<'a> 与 impl Excerpt<'_>，解释生命周期绑定及方法额外借用。 |

## 四项原 gap 的可重现实验

原审计四项 `gap` 为：`book-8-3-hashmap-collect`、`book-8-3-hashmap-hasher`、`book-9-1-panic-unwind-abort` 和 `book-9-1-panic-backtrace`。

代码：[book_gap4.rs](../projects/book-d6-foundations/foundation-lab/src/book_gap4.rs)，以及 [panic_backtrace_probe.rs](../projects/book-d6-foundations/foundation-lab/examples/panic_backtrace_probe.rs)。

在仓库根目录执行：

```powershell
cargo test --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab --lib book_gap4
cargo test --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab --doc book_gap4
```

正向测试需要通过。故意触发 panic 的示例只能放在单独进程，**不在个人项目主进程里演示 abort**。

PowerShell 的回溯对比：

```powershell
$env:RUST_BACKTRACE = "0"
cargo run --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab --example panic_backtrace_probe
$env:RUST_BACKTRACE = "1"
cargo run --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab --example panic_backtrace_probe
Remove-Item Env:RUST_BACKTRACE
```

这两次执行**预期非零退出**；打开 backtrace 后应能定位自身示例的调用位置。栈内容、具体路径和行号以本机编译器为准，不能伪造固定日志。unwind 和 abort 区别见 [Rust 参考手册](https://doc.rust-lang.org/reference/panic.html)，切勿将中止进程作为普通错误处理手段。

## 验收边界

这轮建立了 31 项任务入口，并为四项原 `gap` 增加参考演练和可执行检查；其余 27 项需要逐条核对个人失败/修复产物、目标解释和独立复核，**暂不宣称全部完成正式教学签核**。不为了提高覆盖率删减目标或重复计数。读课、参考代码的测试通过、个人能力达标分开登记。

补充如何保留学员真实证据见 [学习效果观察协议](LEARNER_EVIDENCE_PROTOCOL.md)。
