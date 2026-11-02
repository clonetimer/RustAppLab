# 进阶 B · 从可以运行到可以复用

概念阅读入口：在平台根目录运行 `cargo run --locked --offline -p labquest -- learn reading "my-taskpocket-pro"`，参见 [Rust Book 对照学习](BOOK_GUIDE.md)。

## B1 数据层（数据契约）

先阅读 `taskcore/src/lib.rs`，并用测试定义数据格式。`Task`、`Summary` 属于业务层，不应由 CLI 决定它们的含义。学员需解析损坏行、重复 ID、非法状态、中文和空数据文件。完成后讨论：什么该返回 `Result`，什么可以用普通值表达。

## B2 业务层（功能变更）

实现 `add`、`complete`，确保错误发生时已有数据不被改变。观察闭包/迭代器写法和手写循环各自的清晰度。设计一条自己想要的规则（例如标题去重），先补测试，再改业务代码。

## B3 工程交付（两程序复用）

同一份 `taskcore` 同时供 CLI 与报告程序使用。实际操作：add 任务、完成任务、新进程 list/stats、Markdown 报告。解释为何 `taskreport` 应依赖业务库而非 CLI。为新的命令增加回归测试，确认坏数据不会被悄悄覆盖。

## Cargo 工具实践

```powershell
cargo metadata --no-deps --format-version 1
cargo tree --workspace
cargo test --offline --workspace
cargo test --offline -p taskcore --features strict-titles
cargo test --locked --offline --workspace --doc
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

注意 feature 可能因依赖图而合并；独立运行某个包的行为不等于完整 Workspace feature 组合。

B3 平台检查先累计回归 B1/B2，再验证新进程 list 的完整 TSV、独立 Markdown 报告，以及默认和 strict-titles 两种标题规则。80/81 个中文字符边界会通过验证、解析和新增接口分别检查。

## B4（待真实依赖，不能伪称已完成）

下一阶段使用 serde / serde_json 实现 JSON 导出，练习 crates.io 依赖、Cargo.lock、derive、features 与 MSRV 核对。本轮交付仍为 B1–B3；B4 需补齐依赖锁定、实现和 Windows/Linux 验证后进入已交付范围。

## 质量范围

当前教学型文件保存未承诺断电原子性、多进程并发安全、符号链接竞争安全；真实备份应使用 SafeSync，不应仅靠这份 TaskPocket 数据文件。
