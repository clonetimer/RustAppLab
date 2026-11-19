# 进阶 B · 让一份业务库服务多个程序

本阶段把 TaskPocket 的任务管理拆成可复用的 `taskcore`、命令行程序和 Markdown 报告程序，再增加只读 JSON 导出。你要交付的是自己的多包工程、任务数据、回归测试和设计说明。

## 什么时候来学

建议先完成 [TaskPocket 独立作品](../../course/INDEPENDENT_PATH.md)，能解释 `Result`、结构体、文件读写和测试。刚完成 FirstCard 时，请先按 [FirstCard 后续指南](../../course/AFTER_FIRSTCARD.md) 做 UnitLab。

- `learner/` 是可编译的 TODO 骨架和可见测试；通过平台创建自己的工程后再编辑。
- `reference/` 是完整参考实现，用于比较行为，不计为你的作品。
- B1–B3 使用本地 path 依赖；B4 的 `json-export/` 是独立 Workspace，使用真实 Serde。
- Edition 2021 / Rust 1.74；保留随工程提供的 `Cargo.lock`。B4 首次需要联网准备，源码目录不附 vendor 缓存。

## 第一次开始：先做 B1

在 **RustAppLab 平台根目录**（含 `apps/` 的目录）运行；目标目录必须不存在：

```powershell
cargo run --locked --offline -p labquest -- advanced-b start "my-taskpocket-pro"
cargo run --locked --offline -p labquest -- advanced-b lesson
cargo run --locked --offline -p labquest -- advanced-b check "my-taskpocket-pro" 1
```

初次检查因 TODO 未完成而失败是正常起点。打开生成的 `my-taskpocket-pro/START_HERE.md`，先改自己工程的 `taskcore/src/lib.rs`，再重跑同一检查。

| 检查点 | 实现位置与任务 | 通过后能做什么 |
|---|---|---|
| B1 | `taskcore/src/lib.rs`：校验标题、解析/输出 TSV、汇总 | 将文本变成任务，拒绝坏数据 |
| B2 | 同文件：`add`、`complete` | 更改业务状态，失败时保留原数据 |
| B3 | 同文件：`load`、`save`、`markdown`；阅读两个 CLI | 同一业务库供任务程序和报告程序复用 |
| B4 | `json-export/src/lib.rs`、`src/main.rs` | 用 DTO / Serialize 导出精确、有序、只读的 JSON |

依次把 `check` 最后的阶段改为 `2`、`3`；每个阶段会累计回归前序合同。查看下一步用：

```powershell
cargo run --locked --offline -p labquest -- advanced-b next "my-taskpocket-pro"
```

## B3 通过后，用自己的任务验证

在 **自己的 `my-taskpocket-pro` 工程目录**运行。选一份未使用的任务文件名：

```powershell
cargo run --locked --offline -p taskpocket-pro -- add my-tasks.tsv "解释业务库为什么可以复用"
cargo run --locked --offline -p taskpocket-pro -- done my-tasks.tsv 1
cargo run --locked --offline -p taskpocket-pro -- list my-tasks.tsv
cargo run --locked --offline -p taskpocket-pro -- stats my-tasks.tsv
cargo run --locked --offline -p taskpocket-report -- my-tasks.tsv
```

新文件第一次添加应得到 `added=1`，完成后汇总应为 `total=1 open=0 done=1`。最后一条在终端输出 Markdown；它不会自动创建报告文件。换一个新进程再读，数据仍应存在。

## B4：准备环境以后再实现

回到 **平台根目录**，明确联网准备固定依赖：

```powershell
cargo run --locked --offline -p labquest -- advanced-b prepare-json "my-taskpocket-pro"
# 实现 JSON 的两个 TODO 后再检查
cargo run --locked --offline -p labquest -- advanced-b check "my-taskpocket-pro" 4
```

外层 `--offline` 只约束平台启动；`prepare-json` 内部会联网 fetch，并验证平台固定参考源码，不运行学员源码。日志在学员工程的 `.advanced-b-dependencies/`，准备成功不等于课程通过。

新工程已有 JSON 骨架。旧工程缺少扩展时才运行 `advanced-b add-json <工程路径>`，已有扩展拒绝覆盖。`4`、`all` 和 `learn check` 包含 B4；暂不准备依赖时显式检查 `3`。

B4 通过后，在 **学员工程目录**运行：

```powershell
cargo run --locked --offline --manifest-path json-export/Cargo.toml -- export-json my-tasks.tsv
```

应得到含 `id`、`done`、`title` 的 JSON 数组；导出前后的 TSV 字节不变。它不提供 JSON 导入，也不创建缺失的数据库。

## 怎样知道自己学会了

用中文、引号、乱序 ID 和不存在的任务做个人实验；为失败不改数据增加一条测试。说明 `taskreport` 为什么依赖 `taskcore`，以及 DTO 为什么留在 JSON 适配包。再给自己的 CLI 增加一个小功能及回归测试，证明你能复用公开接口。

完整规则、strict-titles 的 80/81 字符边界、三级提示见 [B 讲义](ADVANCED_B.md)；Windows 操作见 [B4 指南](../../course/ADVANCED_B_WINDOWS.md)。源码修改后重新检查，旧报告不能证明新代码通过。

完成个人交付后，可用 [D1 Cargo 参考实验](../book-d1-cargo/README.md) 补 Workspace 和安装知识；已有同步 NetFetch 与 AsyncBoard 基础时，再进入 [C1 异步下载](../advanced-c1/README.md)。

## 参考实现回归与边界

维护者在 **平台根目录**执行；第一条联网，其余离线：

```powershell
cargo fetch --locked --manifest-path projects/advanced-b/reference/json-export/Cargo.toml
cargo test --locked --offline --manifest-path projects/advanced-b/reference/Cargo.toml --workspace
cargo test --locked --offline --manifest-path projects/advanced-b/reference/json-export/Cargo.toml --all-targets
cargo test --locked --offline --manifest-path projects/advanced-b/reference/json-export/Cargo.toml --all-targets --all-features
cargo test --locked --offline --manifest-path projects/advanced-b/reference/json-export/Cargo.toml --doc
```

教学型 `save` 不保证断电一致性或多进程写入安全。平台检查会执行自己的可信 Rust 代码，不是恶意程序沙箱；只读导航不会自动联网准备或升级工程。
