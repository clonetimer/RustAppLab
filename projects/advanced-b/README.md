# 进阶 B · 可复用业务库与 Serde JSON 导出

- `reference/`：基础三成员 Workspace：taskcore、TaskPocket CLI、Markdown 报告；独立的 `reference/json-export/` 使用真实 Serde。
- `learner/`：相同接口与可编译 TODO、固定可见测试；B1–B4 分步实现。
- [完整课程](ADVANCED_B.md)：编辑位置、显式准备、DTO/derive、三级提示、个人数据实验和只读边界。

基础 B1–B3 仍不依赖 crates.io。B4 独立 Workspace 使用 Edition 2021 / Rust 1.74 和固定 Cargo.lock；首次需要联网准备，不交付 vendor 缓存。

在平台根目录开始学员工程：

```powershell
cargo run --locked --offline -p labquest -- advanced-b start "my-taskpocket-pro"
cargo run --locked --offline -p labquest -- advanced-b prepare-json "my-taskpocket-pro"
cargo run --locked --offline -p labquest -- advanced-b check "my-taskpocket-pro" 4
```

旧工程先显式 `advanced-b add-json <目录>`；若扩展已存在则拒绝覆盖。新工程不需再添加。先完成前三阶段，B4 尚未实现时检查失败是正常反馈。

维护者直接验证参考软件（第一条明确联网，其余离线）：

```powershell
cargo fetch --locked --manifest-path projects/advanced-b/reference/json-export/Cargo.toml
cargo test --locked --offline --manifest-path projects/advanced-b/reference/Cargo.toml --workspace
cargo test --locked --offline --manifest-path projects/advanced-b/reference/json-export/Cargo.toml --all-targets
cargo test --locked --offline --manifest-path projects/advanced-b/reference/json-export/Cargo.toml --all-targets --all-features
cargo test --locked --offline --manifest-path projects/advanced-b/reference/json-export/Cargo.toml --doc
```

准备过程使用平台固定源码，不运行学员源码；完成准备不等于通过课程。JSON 导出不写回 TSV，不提供 JSON 导入。只读导航不会自动准备或升级工程。
