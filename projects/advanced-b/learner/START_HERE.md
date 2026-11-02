# 进阶 B：TaskPocket Pro · 从单文件到可复用软件

## 软件最终形态
独立 Cargo Workspace 包含 `taskcore` 业务库、`taskcli` 命令行与 `taskreport` Markdown 报告程序。运行同一数据文件，两个程序使用相同的业务规则。

## 四个检查点

- **B1（纯逻辑）**：编辑 `taskcore/src/lib.rs`，实现 `validate_title`、`parse_tsv`、`to_tsv`、`summary`。预测重复 ID/非法状态/中文标题的处理。
- **B2（业务变更）**：实现 `add` 和 `complete`，处理编号溢出、非法标题、不存在的 ID；失败时内存数据保持不变。
- **B3（软件集成）**：实现 `load`、`save`、`markdown`，然后在新目录用 CLI 加任务/完成任务，用报告程序导出。修改 `taskcli/src/main.rs` 让它支持至少一条自定义新功能，如 `search`。

- **B4（第三方依赖）**：显式准备后，编辑 `json-export/src/lib.rs`、`json-export/src/main.rs`，用 DTO 和 `#[derive(serde::Serialize)]` 实现只读 JSON 导出；按 id 排序，精确保留 u64/bool/字符串，复用 taskcore 校验。

从平台根目录对本工程路径运行 `advanced-b prepare-json <目录>`（明确联网），然后 `advanced-b check <目录> 4`。新工程已含扩展；旧工程先 `advanced-b add-json <目录>`，已有扩展不覆盖。`all`/`learn check` 包括 B4，显式 `3` 保留原无 registry 依赖检查。准备只运行平台固定参考源码，不运行本工程源码；日志在 `.advanced-b-dependencies/`，准备成功不等于课程通过。

B4 三级提示：① 先画 TSV→Task→DTO→JSON 数据流，确认错误不写文件；② 复用 validate_title、集合检查 id，排序 DTO 而非输入；③ 用 Serialize derive 和 serde_json::to_string，CLI 完成所有校验后再输出 JSON 与换行。拒绝 silent trim 标题，strict-titles 沿用 80 字符规则。

## 手动检查
在工程根目录依次执行：

```powershell
cargo test --locked --offline -p taskcore --test b1
cargo test --locked --offline -p taskcore --test b2
cargo test --locked --offline -p taskpocket-pro --test b3
cargo test --locked --offline -p taskpocket-report --test report_cli
cargo test --locked --offline -p taskcore --test feature --no-default-features
cargo test --locked --offline -p taskcore --test feature --no-default-features --features strict-titles
```

前一阶段未通过时，不要跳到下一阶段。`B3` 的 CLI 测试需要真实磁盘和进程，核对新进程的 `list` 输出和 `stats`。平台检查 B3 会累计重跑 B1、B2，再执行 CLI、独立报告程序，以及默认/`strict-titles` 两种配置的固定合同；任何一项失败都不能获得 B3 通过记录。B1、B2 不要求先完成 B3 功能。

B4 准备后在本工程目录运行：

```powershell
cargo test --locked --offline --manifest-path json-export/Cargo.toml --all-targets
cargo test --locked --offline --manifest-path json-export/Cargo.toml --all-targets --all-features
cargo test --locked --offline --manifest-path json-export/Cargo.toml --doc
cargo run --locked --offline --manifest-path json-export/Cargo.toml -- export-json tasks.tsv
```

用虚构任务测试中文、引号、反斜杠、u64::MAX、乱序 id 和 80/81 字符；比较导出前后 TSV 字节。导出不提供导入，不创建缺失数据库。旧 B3 报告不冒充 B4；修改基础或 JSON 源码后重新检查，最新一次只检查 3 时导航仍提示 B4 待检查。

## 设计审查
1. 为什么 CLI 不直接解析 TSV？
2. `taskreport` 为何不应该依赖 `taskcli`？
3. 如果数据库文件原本是坏数据，`add` 应该覆写还是报错？
4. 共享业务逻辑使用 path 依赖与复制粘贴相比，有何不同？
5. `strict-titles` 是 Cargo feature；切换后哪些语义会变化？
6. 尝试给 `taskcore` 的公开函数增加 `///` 文档示例，在其中断言真实行为，用 `cargo test --locked --offline -p taskcore --doc` 运行；参考实现的 `parse_tsv` 提供了示例。再用 `cargo tree` 查看依赖。

注意：当前 `save()` 是教学型写入，不保证多进程并发安全或断电一致性。平台自动测试不是安全沙箱。
