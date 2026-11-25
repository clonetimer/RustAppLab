# D9-A · 现代 Rust 稳定特性：处理一批数据

[现代化路线](MODERN_RUST_ROADMAP.md) · [D8](BOOK_D8_EDITION_2024.md) · [版本索引](MODERN_STABLE_FEATURE_INDEX.md) · [实验室](../projects/modern-d9-stable/README.md)

完成 D8 后，从 projects/modern-d9-stable/learner 复制整个目录，在自己的副本修改 src/lib.rs。固定 Rust 1.100.0 / Edition 2024，无需第三方依赖；首次安装编译器需要联网。不是将旧 Rust 工程改为新版。正常学员不需要 Python。

## 九步任务：先预测，再实现，再验证

| 步骤 | 新能力 | 写什么 | 正常/错误输入与迁移 |
|---|---|---|---|
| 1 | GAT（1.65，前置补课） | LendingReader::next 返回借用自身的滑动窗口；不克隆整批数据 | 宽度 0、超过长度、重叠窗口、第二次借用前使用第一次借用；尝试宽度 3 |
| 2 | 精确捕获（1.82） | owned_minimum 返回拥有结果的迭代器，签名 use<> 不捕获输入生命周期 | 清空输入、短生命周期输入、空集合；换成最大值 |
| 3 | Trait Upcasting（1.86） | 将 &dyn Sensor 转为 &dyn Named；不创建替代对象 | 检查名字和指针；增加不同 Sensor 实现 |
| 4 | async fn in trait（1.75） | collect 逐项 await SampleSource，跳过缺失值，保序 | 交替有效和缺失 key、空输入；增加另一数据源 |
| 5 | async closure/AsyncFnMut（1.85） | visit 接收可借用捕获状态的异步闭包，逐项调用并 await | 输出顺序、空列表、闭包修改外部 Vec；将倍增改成计数 |
| 6 | match if-let guard（1.95） | positive 用 match guard 解析非零 u32，仍需兜底 arm | 负值、溢出、无值、合法边界；解析其他格式 |
| 7 | Never type（1.100） | infallible 用不可构造的错误类型提取结果，不需要 panic | 整数、拥有型 String、Infallible 别名；换成自己的领域类型 |
| 8 | DropGuard（1.100） | append_batch 在中间解析失败时恢复原 Vec 长度，成功 dismiss | 已有数据、首项失败、中途失败、溢出、空批次；改变旧缓冲区 |
| 9 | Allocator（1.100） | copy_into 用调用方的 allocator 分配 Vec，并保留全部值 | System/Global、空 Vec、最大值；传入另一合法分配器 |

每次在学员副本运行 cargo test --locked --offline --test contracts step1_，按步骤更换 1 到 9。初始 TODO 会编译，业务测试失败是起点。公开合同验证行为，不能强制你使用某个语法；应保留个人改动并解释所用概念。参考解答仅在完整 source 包中。

## 为什么要学习这些能力？

**借用与泛型。** GAT 允许关联类型依赖本次 self 借用的生命周期；旧 Iterator 的 Item 类型不能直接表达这类重复借用。第一次窗口仍被使用时，不能再次可变借用 reader。Edition 2024 默认更广泛捕获 RPIT 生命周期，use<> 可明确不捕获；它不能让真正借用的数据活得更久。Trait upcasting 只允许转为真实 supertrait，不能随意转换成不相关 Trait。

**异步。** async fn in trait 提供静态分发接口，但并不自动产生 dyn-compatible Trait 或 Send Future 保证。AsyncFnMut 可以让 async closure 的 Future 借用其捕获状态；普通返回 async block 的 FnMut 闭包不能随意将这种借用逃逸。此课测试只使用明确可立即完成的 Future，以单次 poll 验证异步语法和数据行为；不模拟网络，不假装一般 executor。D10 会专门处理 Pending、Waker、Pin、Stream、取消与真实运行时。

**模式与资源。** 带 if-let guard 的 match 不把 guard 当成穷尽性证明；必须保留 fallback。Never 表示不存在的值，不是“总会 panic”；Result<T, !> 可以直接用不可失败的模式提取。1.100 的 Infallible 为 ! 别名。DropGuard 是内存状态回滚工具，不等于数据库事务、断电恢复或 panic=abort 保证；dismiss 是提交，忘记 dismiss 会在 drop 时回滚。Allocator 支持 Vec/Box 的核心接口；其他集合和 allocator_ext 的 nightly 接口不纳入本课。学员只调用已有安全分配器，不要求编写 unsafe 内存管理。

## 真实反例

完整源包 reference 中有独立 probes/，每组 red.rs 是刻意错误的程序、green.rs 是修复版。不要把 red.rs 放进正式 crate 的 src/。进入 reference 后，可分别运行 rustc --edition=2024 --emit=metadata probes/gat/red.rs（预期失败）和 cargo test --locked --offline --all-targets。诊断目标包括 GAT 活跃借用、过度捕获、错误 Trait 转换、异步 Trait 的 dyn 限制、普通 FnMut 闭包逃逸、guard 不穷尽、构造 Never、Allocator 类型边界与忘记提交的实际行为。完整验收脚本按诊断和运行输出验证，不接受其他无关编译错误。

最后在个人工程运行 cargo test --locked --offline --all-targets，再运行 cargo run --locked --offline --example batch -- 7 0 3 和 cargo run --locked --offline --example batch -- 7 bad 3 。前者应显示 accepted=2; stored=[100, 7, 0, 3]，后者非零退出并显示 batch rejected; retained=[100]。保留自己的新输入和源码变化，填写 LEARNING_LOG.md。

维护者从源码根目录运行 python scripts/modern_d9_check.py --output d9-evidence （Python 3.11+）；检查在临时副本中进行，保存原始日志与 SHA-256。D9-A 不是 26 个版本所有 API 已完成的声明；余下版本增量、标准库/Cargo、RPITIT、unsafe 专项、网页工作台和真人记录仍待推进。

参考 [GAT](https://blog.rust-lang.org/releases/1.65.0) · [async trait](https://blog.rust-lang.org/releases/1.75.0) · [precise capture](https://blog.rust-lang.org/releases/1.82.0) · [async closures](https://blog.rust-lang.org/releases/1.85.0) · [upcasting](https://blog.rust-lang.org/releases/1.86.0) · [if-let guards](https://blog.rust-lang.org/releases/1.95.0) · [1.100 稳定特性](https://blog.rust-lang.org/releases/1.100.0) · [Allocator](https://doc.rust-lang.org/1.100.0/std/alloc/trait.Allocator.html)。
