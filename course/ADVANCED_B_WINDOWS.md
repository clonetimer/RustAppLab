# Windows 学员 · 进阶 B（无需重新安装 Rust）

把新版源码包解压到新目录，保留已有学员工程。进入包含顶层 Cargo.toml 的 RustAppLab 文件夹；中文和空格路径始终用引号。

1. 双击 `START_ADVANCED_B_WINDOWS.cmd` 创建全新练习，或使用旧工程路径。新工程已包含 json-export。
2. 编辑学员 `taskcore/src/lib.rs`，从 B1 开始；运行 `cargo run --locked --offline -p labquest -- advanced-b check "my-taskpocket-pro" 1`。
3. 完成 B2、B3 后分别检查阶段 2 和 3；累计检查会回归前面阶段。
4. **旧工程升级 B4**：在平台根目录执行 `cargo run --locked --offline -p labquest -- advanced-b add-json "my-taskpocket-pro"`。它只添加新目录，已有 json-export 则拒绝覆盖。
5. **显式联网准备**：执行 `cargo run --locked --offline -p labquest -- advanced-b prepare-json "my-taskpocket-pro"`。外层 Cargo 离线启动平台，子命令内部明确联网下载固定依赖；准备日志保存在学员 `.advanced-b-dependencies/`。不运行学员源码、不修改全局 Cargo 配置。
6. 编辑学员 `json-export/src/lib.rs` 与 `src/main.rs`，完成 DTO/Serialize derive、快照验证、排序和只读 CLI。按 [完整讲义](ADVANCED_B.md) 查看三级提示。
7. 执行 `cargo run --locked --offline -p labquest -- advanced-b check "my-taskpocket-pro" 4`。`all`、`learn check` 也检查到 B4；`3` 保留原有无 registry 缓存的检查。

在学员工程目录试用：

```powershell
cargo run --locked --offline -p taskpocket-pro -- add tasks.tsv "完成进阶B"
cargo run --locked --offline -p taskpocket-report -- tasks.tsv
cargo run --locked --offline --manifest-path json-export/Cargo.toml -- export-json tasks.tsv
```

导出输出到控制台；不要把输出重定向到输入 TSV。PowerShell 的文件重定向编码因版本而异，先在控制台检查结果，再按需要保存副本。数据库不存在会输出 `[]`，不会创建数据库。

`advanced-b next`、`learn lesson`、`learn reading` 只读；不会下载依赖、迁移旧工程或写成绩。旧 B3 报告不代表通过 B4，修改 JSON 源码后需要重新检查。缓存缺失时显式重跑准备；锁错误或 Cargo 缺失请按保留的实际诊断处理。

正式目标为 Windows x64 MSVC、Linux x86_64、Rust 1.74；实际平台结果以当前源码的 CI 记录为准。教学检查只运行自己的可信源码，不是恶意代码沙箱；真实异步网络课程 C 尚未交付。
