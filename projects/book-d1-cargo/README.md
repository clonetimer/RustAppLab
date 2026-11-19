# Book D1 · Cargo 可运行参考实验

这是 Rust Book 第 14 章的**可选补充实验**，用一个业务库和一个 Cargo 子命令认识 Workspace、profile、打包、安装和工具扩展。它不是新的学员评分工程，不增加主工作区的 17 个包。

## 什么时候用它

建议完成 [进阶 B 的 B1–B3](../../course/ADVANCED_B.md) 后，带着“我的程序怎样组织、构建和安装”的问题来做。刚完成 FirstCard 时，先按 [基础后续指南](../../course/AFTER_FIRSTCARD.md) 进入 UnitLab。

Edition 2021、最低 Rust 1.74、零外部 Rust 依赖，所有包 `publish = false`。本目录有自己的 `Cargo.toml` 和 `Cargo.lock`；以下 `--manifest-path` 选择这个独立 Workspace，而不是根平台。

## 先运行，观察 Cargo 到底调用了什么

在 **源码根目录或学员包根目录**（能看到 `projects/`）运行：

```powershell
cargo run --locked --offline --manifest-path projects/book-d1-cargo/cargo-applab-map/Cargo.toml -- profiles
cargo run --locked --offline --manifest-path projects/book-d1-cargo/cargo-applab-map/Cargo.toml -- workspace
cargo metadata --locked --offline --no-deps --format-version 1 --manifest-path projects/book-d1-cargo/Cargo.toml
```

前两条分别输出以 `14.1`、`14.3` 开头的说明；metadata 应显示 `book-card` 与 `cargo-applab-map` 对应的两个 package。主题还包括 `publish`、`install`、`extension`。这些主题输出教学说明，不会自动发布或安装软件。

把主题换成 `invalid` 应返回错误和非零退出码。用自己的话解释 `--` 前后参数分别由谁处理。

## 从哪个文件读起

| 文件 | 看什么 | 你应能解释的关系 |
|---|---|---|
| `Cargo.toml` | members、dev/release profile | 构建配置为什么放在 Workspace 根 |
| `book-card/Cargo.toml`、`src/lib.rs` | 库包与 `Topic` / `describe` | 公共接口怎样保存并复用主题逻辑 |
| `cargo-applab-map/Cargo.toml` | path 依赖与二进制名 | package、crate 和程序名的区别 |
| `cargo-applab-map/src/main.rs` | 参数解析 | 直接运行与 Cargo 外部子命令调用的差别 |
| `cargo-applab-map/tests/cli.rs` | 真实进程测试 | 怎样验证输出和失败退出 |

## 参考代码的检查

仍在 **同一个根目录**执行：

```powershell
cargo test --locked --offline --workspace --all-targets --manifest-path projects/book-d1-cargo/Cargo.toml
cargo test --locked --offline --workspace --doc --manifest-path projects/book-d1-cargo/Cargo.toml
```

两条分别检查普通目标与文档示例。测试通过证明参考实现符合当前合同，不会给你生成个人学习成绩。

## 一次个人迁移练习

把整个 `projects/book-d1-cargo` 复制到新的个人目录，保留清单和锁文件。在**副本**的 `book-card/src/lib.rs` 增加讲义要求的 `lint` 主题，解释 `cargo clippy` 的用途；在 CLI 测试中覆盖合法主题、未知主题、多余参数。

在 **副本的 Workspace 根目录**运行：

```powershell
cargo test --locked --offline --workspace --all-targets
cargo test --locked --offline --workspace --doc
cargo run --locked --offline -p cargo-applab-map -- lint
```

完成后应能指出：新增逻辑属于库，CLI 通过依赖使用它。再按 [D1 完整讲义](../../course/BOOK_D1_CARGO.md) 选做 profile 对比、`cargo package --list` 和本地安装；记录真实命令及产物，不凭构建速度推断普遍性能结论。

安装练习使用全新的专用 `cargo install --root` 目录，不覆盖日常 Cargo 工具；保持离线，不运行 `cargo publish`。发布准备和实际上传是不同事情。

## 学完后回到哪里

把本次依赖图、一个新增测试和一条错误命令记入个人学习日志，再回自己的 B 工程检查库与 CLI 分工。若疑问转为“数据由谁拥有、何时释放”，可继续 [D2 智能指针参考实验](../book-d2-smart-pointers/README.md)；其他概念按 [Book 对照指南](../../course/BOOK_GUIDE.md) 按需查找。
