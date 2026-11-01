# Windows 学员 · 进阶 B（无需重新安装 Rust）

建议把新源码包解压到新目录，保留原 0.19.0-dev.1 进阶 A 工程。进入包含顶层 Cargo.toml 的 RustAppLab 文件夹。

1. 双击 `START_ADVANCED_B_WINDOWS.cmd`；默认创建全新 `my-taskpocket-pro` 练习目录。
2. 打开工程 `START_HERE.md`，编辑 `taskcore/src/lib.rs`；先实现 B1 的四项函数。
3. 在平台根目录运行 `cargo run --locked --offline -p labquest -- advanced-b check my-taskpocket-pro 1`。初始未实现会失败，这是刻意设置的教学步骤。
4. B1 通过后再运行阶段 2 和 3。完整参考软件在平台 `projects/advanced-b/reference`；不要将其整个复制为学员提交。
5. 最终切换到学员工程，运行 `cargo run --offline -p taskpocket-pro -- add tasks.tsv "完成进阶B"`；随后使用 `cargo run --offline -p taskpocket-report -- tasks.tsv` 查看报告。

`labquest advanced-b next <目录>` 只读反馈；`learn overview <父目录>` 可查看 B 工程进度。所有检查均为本地运行可信源码，不是隔离恶意代码的沙箱。

**支持边界**：当前 Linux x86_64 已测试，Windows x64 MSVC 的新入口需原生复验；Rust 1.74 最低版本尚未对本开发分支重新验证。不声明 macOS。
