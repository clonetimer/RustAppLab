# Book D6-A · 附录参考与实践

这是 Rust Book 2018 附录 A–G 的**可选参考实验**。代码部分聚焦关键字、运算符、可派生 trait 与 `Debug` / `Display`；工具、Edition 和工具链实践由配套讲义提供。

## 按问题来查，不必从头刷完

适合已经会运行 Cargo、写基本函数和结构体，正在查一个语法或工具问题的学员。FirstCard 初学者先按 [基础后续指南](../../course/AFTER_FIRSTCARD.md) 做作品；本目录不是必经的下一关。

- 独立 Rust 2021 Workspace，最低 Rust 1.74、零 registry 依赖、`publish = false`。
- 不属于根工作区，也不增加学员作品数量。
- `appendix-lab` 是**库包**；从测试和 rustdoc 观察行为，没有需要启动的 CLI。
- 完整参考表与练习见 [D6-A 讲义](../../course/BOOK_D6_APPENDICES.md)，工具 / Edition / Nightly 实操见 [D/E/G 实践](../../course/BOOK_D6_PRACTICAL.md)。

## 先运行与你的问题有关的测试

在 **源码根目录或学员包根目录**（能看到 `projects/`）运行：

```powershell
cargo test --locked --offline --manifest-path projects/book-d6-appendices/Cargo.toml raw_identifier_allows_a_keyword_as_a_function_name
cargo test --locked --offline --manifest-path projects/book-d6-appendices/Cargo.toml debug_and_display_are_distinct_contracts
cargo test --locked --offline --manifest-path projects/book-d6-appendices/Cargo.toml --workspace --doc
```

前两条应显示对应测试通过；第一条检查 `r#match` 这样的 raw identifier，第二条比较调试输出与面向用户的显示。文档测试同时包含可运行示例和应该编译失败的示例；`compile_fail` 通过表示非法程序按预期被拒绝。

## 读哪个例子、回答什么问题

实现与单元测试都在 `appendix-lab/src/lib.rs`：

| 入口 | 观察或解释 |
|---|---|
| `r#match` 与相关 rustdoc | 关键字如何作为已有接口名使用，哪些特殊名称仍不允许 |
| `precedence_example`、`range_total` | 运算符优先级与范围是否包含上界 |
| `apply_mask`、`Point` 的 `Add` 实现 | 位运算与逻辑运算、普通运算符与 trait 重载 |
| `Task`、`Priority` | 派生 Clone / Eq / Hash / Ord 等的约束 |
| `Debug` 与 `Display` 测试 | 调试表示和用户显示为什么是不同合同 |

这份库不能替代讲义的全部关键字、符号和版本差异表。看到一个词或测试通过，不等于已经掌握附录全部内容。

## 一次个人迁移练习

把整个实验复制到新的个人目录。在副本 `appendix-lab/src/lib.rs` 给 `Task` 增加一个字段，先判断哪些 derive 仍然成立，再调整构造代码和测试。尝试给含 `String` 的类型派生 Copy 时，说明编译器拒绝的原因，保留为编译失败示例或恢复后再跑完整检查。

在 **副本 Workspace 根目录**运行：

```powershell
cargo test --locked --offline --workspace --all-targets
cargo test --locked --offline --workspace --doc
```

完成后应能给一条表达式加括号说明优先级，给一个类型解释 derive 的条件，并写出它自己的 Display 输出。记录预测、首条诊断和最后结果，不只保存通过截图。

## 参考代码的完整验证

在 **同一个根目录**执行；fmt / clippy 需要本机对应组件：

```powershell
cargo fmt --manifest-path projects/book-d6-appendices/Cargo.toml --all -- --check
cargo clippy --locked --offline --manifest-path projects/book-d6-appendices/Cargo.toml --workspace --all-targets -- -D warnings
cargo test --locked --offline --manifest-path projects/book-d6-appendices/Cargo.toml --workspace --all-targets
cargo test --locked --offline --manifest-path projects/book-d6-appendices/Cargo.toml --workspace --doc
cargo test --locked --offline --manifest-path projects/book-d6-appendices/Cargo.toml --workspace --all-targets --release
```

## 下一步

回到自己的作品，为遇到的关键字或 trait 写一条具体解释；工具与 Edition 练习按讲义在个人副本执行，不升级整个根 Workspace。若缺口是变量、模块、数组、所有权或集合，按 [D6-B 基础参考实验](../book-d6-foundations/README.md) 选择对应单元。参考测试没有平台自动评分含义。
