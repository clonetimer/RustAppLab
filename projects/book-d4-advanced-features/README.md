# Book D4 · 高级特性参考实验

这是中文 Rust Book 2018 版第 19 章的**可选补充实验**，按主题对应新版英文 Book 的 Advanced Features。五个入口演示 unsafe、高级 trait、类型、函数与闭包、宏；它不是新建学员项目的入口。

## 什么时候需要它

已有所有权、借用、trait、泛型和闭包基础，并且能提出一个具体接口问题时来查。初学者先看 [FirstCard 后续指南](../../course/AFTER_FIRSTCARD.md)；对象设计可先补 [D3](../book-d3-oop-patterns/README.md)。

本实验独立于根 Workspace，Edition 2021、最低 Rust 1.74、零第三方依赖、`publish = false`。过程宏只使用稳定 `proc_macro` API，不依赖 syn / quote，也不需要 Nightly。详细安全契约和练习见 [D4 讲义](../../course/BOOK_D4_ADVANCED_FEATURES.md)。

## 一次选择一个主题

在 **源码根目录或学员包根目录**（能看到 `projects/`）运行：

```powershell
cargo run --locked --offline --manifest-path projects/book-d4-advanced-features/advanced-lab/Cargo.toml -- unsafe
cargo run --locked --offline --manifest-path projects/book-d4-advanced-features/advanced-lab/Cargo.toml -- traits
cargo run --locked --offline --manifest-path projects/book-d4-advanced-features/advanced-lab/Cargo.toml -- types
cargo run --locked --offline --manifest-path projects/book-d4-advanced-features/advanced-lab/Cargo.toml -- functions
cargo run --locked --offline --manifest-path projects/book-d4-advanced-features/advanced-lab/Cargo.toml -- macros
```

| 主题 | 输出线索 | 应能解释什么 |
|---|---|---|
| `unsafe` | `raw-sum=10`、两个互不重叠的切片 | 哪些不变量由安全封装保证？ |
| `traits` | `point=6,8`、`qualified=trait name` | 关联类型、运算符 trait、完全限定调用 |
| `types` | `meters=12`、`display-len=4` | newtype 与别名、动态大小类型的引用 |
| `functions` | `function-pointer=5`、`returned-closure=12` | 函数指针与闭包及其捕获的区别 |
| `macros` | `derive=Packet`、`attribute=telemetry`、`function-macro=3` | 声明宏与三类过程宏在编译期做什么 |

## 找到实现和证据

`advanced-lab/src/lib.rs` 包含 demo、公开接口、单元测试和 SAFETY 注释；`src/main.rs` 负责 CLI；`tests/cli.rs` 检查真实输出。过程宏的实现位于 `d4-proc-macros/src/lib.rs`，调用例子位于前一个包。

在 **同一个根目录**检查参考代码：

```powershell
cargo fmt --manifest-path projects/book-d4-advanced-features/Cargo.toml --all -- --check
cargo clippy --locked --offline --manifest-path projects/book-d4-advanced-features/Cargo.toml --workspace --all-targets -- -D warnings
cargo test --locked --offline --manifest-path projects/book-d4-advanced-features/Cargo.toml --workspace --all-targets
cargo test --locked --offline --manifest-path projects/book-d4-advanced-features/Cargo.toml --workspace --doc
```

fmt / clippy 需要本机对应工具组件；缺组件属于环境问题。测试通过是参考行为的证据，不会认证你掌握所有高级特性。

## 从一项个人实验开始

把整个实验复制到新的个人目录，先选 `functions`：在副本 `advanced-lab/src/lib.rs` 中，为返回的闭包增加一个不同捕获值和输入的测试，先写预测再运行。随后按完整讲义逐项补 trait、过程宏和安全抽象的个人练习，不必第一天同时修改五个主题。

在 **副本 Workspace 根目录**运行：

```powershell
cargo test --locked --offline --workspace --all-targets
cargo test --locked --offline --workspace --doc
```

你应能指出闭包保存了什么值、为什么返回类型这样写；学习 unsafe 部分时，还应能逐条解释 SAFETY 注释，并说明 `unsafe` 并不关闭普通借用检查。保留自己的测试和解释，比复制 demo 输出更能说明理解。

unsafe 示例仅使用满足契约的有效、有界输入；全局计数器 API 仅限单线程，测试不尝试触发未定义行为。修改实验也应保持这些边界。

## 下一步

回到自己的作品，只有接口确实需要时再应用这些特性。想理解闭包、通道与资源清理如何组合成完整服务，可继续 [D5 同步线程池 Web Server](../book-d5-webserver/README.md)；想查语法和工具则看 [D6 附录](../book-d6-appendices/README.md)。这些都是按需参考入口。
