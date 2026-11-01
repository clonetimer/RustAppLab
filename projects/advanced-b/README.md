# 进阶 B · 可复用业务库与 Cargo Workspace

- `reference/`：可实际使用的参考实现，三个 Workspace member：`taskcore`、`taskpocket-pro`、`taskpocket-report`。
- `learner/`：与参考实现相同的接口，核心函数留给学员实现；提供固定合同测试，初始按设计失败。
- `README.md`：课程进度与离线限制。
- `ADVANCED_B.md`：完整教学路线和未完成的第三方依赖专题。

本阶段重点训练 **Cargo Workspace、path dependency、库/CLI 分层、feature、可独立测试的接口**。当前环境无法从 crates.io 获取 Serde，因此成熟第三方依赖课程不计入已完成项，后续与 Rust 1.74 MSRV 一起验证。

直接运行参考实现：

```powershell
cd projects/advanced-b/reference
cargo test --offline --workspace
cargo run --offline -p taskpocket-pro -- add tasks.tsv "完成Rust进阶B"
cargo run --offline -p taskpocket-pro -- stats tasks.tsv
cargo run --offline -p taskpocket-report -- tasks.tsv
```

开始学员工程：从 `learner` 复制到你自己的新目录，按照 START_HERE.md 做 B1/B2/B3。也可使用 `labquest advanced-b start <新目录>` 生成工程并以 `advanced-b check` 验收。
