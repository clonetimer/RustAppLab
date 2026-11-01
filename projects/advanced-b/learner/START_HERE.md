# 进阶 B：TaskPocket Pro · 从单文件到可复用软件

## 软件最终形态
独立 Cargo Workspace 包含 `taskcore` 业务库、`taskcli` 命令行与 `taskreport` Markdown 报告程序。运行同一数据文件，两个程序使用相同的业务规则。

## 三个检查点

- **B1（纯逻辑）**：编辑 `taskcore/src/lib.rs`，实现 `validate_title`、`parse_tsv`、`to_tsv`、`summary`。预测重复 ID/非法状态/中文标题的处理。
- **B2（业务变更）**：实现 `add` 和 `complete`，处理编号溢出、非法标题、不存在的 ID；失败时内存数据保持不变。
- **B3（软件集成）**：实现 `load`、`save`、`markdown`，然后在新目录用 CLI 加任务/完成任务，用报告程序导出。修改 `taskcli/src/main.rs` 让它支持至少一条自定义新功能，如 `search`。

## 手动检查
在工程根目录依次执行：

```powershell
cargo test --locked --offline -p taskcore --test b1
cargo test --locked --offline -p taskcore --test b2
cargo test --locked --offline -p taskpocket-pro --test b3
```

前一阶段未通过时，不要跳到下一阶段。`B3` 的 CLI 测试需要真实磁盘和进程，不只检查函数输出。

## 设计审查
1. 为什么 CLI 不直接解析 TSV？
2. `taskreport` 为何不应该依赖 `taskcli`？
3. 如果数据库文件原本是坏数据，`add` 应该覆写还是报错？
4. 共享业务逻辑使用 path 依赖与复制粘贴相比，有何不同？
5. `strict-titles` 是 Cargo feature；切换后哪些语义会变化？
6. 尝试给 `taskcore` 增加独立文档测试，并用 `cargo tree` 查看依赖。

注意：当前 `save()` 是教学型写入，不保证多进程并发安全或断电一致性。平台自动测试不是安全沙箱。
