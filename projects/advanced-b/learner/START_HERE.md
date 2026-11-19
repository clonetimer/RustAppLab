# 进阶 B：TaskPocket Pro · 从单文件到可复用软件

## 你要做成什么

把同一份任务数据交给两个程序：TaskPocket CLI 管理任务，`taskreport` 输出 Markdown；二者共用 `taskcore`。最后在独立 `json-export` 包里用 Serde 只读导出 JSON。完成后应能解释业务库、命令行适配和输出格式各自负责什么。

建议先做过 TaskPocket，能用 `Result` 处理文件错误。这里不是 FirstCard 后的第一款练习。完整平台的 `course/AFTER_FIRSTCARD.md` 会引导零基础继续 UnitLab。

**先确认目录**：本文件所在的是你的学员工程，日常改这里的源码。含 `apps/` 的 RustAppLab 目录是平台根目录，`-p labquest` 的命令在那里执行。不要修改平台 `projects/advanced-b/reference` 来完成自己的任务。

## 先完成 B1，再逐步推进

在 **学员工程目录**运行第一项，看到 TODO 失败后再实现：

```powershell
cargo test --locked --offline -p taskcore --test b1
```

| 检查点 | 编辑位置与任务 | 完成后的证据 |
|---|---|---|
| B1：纯逻辑 | `taskcore/src/lib.rs` 的 `validate_title`、`parse_tsv`、`to_tsv`、`summary` | 能处理中文、空数据，拒绝重复 ID 和非法状态 |
| B2：业务变更 | 同文件的 `add`、`complete` | 新增、完成任务；编号溢出或非法输入不改变原数据 |
| B3：两程序复用 | 同文件的 `load`、`save`、`markdown`；阅读 CLI / report | 新进程读回任务，并用独立程序输出一致的报告 |
| B4：JSON 适配 | `json-export/src/lib.rs`、`json-export/src/main.rs` | DTO / Serialize 输出按 id 排序、精确且只读的 JSON |

每次先预测一个正常输入和一个失败输入，只实现当前阶段。通过后在 **学员工程目录**依次检查：

```powershell
cargo test --locked --offline -p taskcore --test b2
cargo test --locked --offline -p taskpocket-pro --test b3
cargo test --locked --offline -p taskpocket-report --test report_cli
cargo test --locked --offline -p taskcore --test feature --no-default-features
cargo test --locked --offline -p taskcore --test feature --no-default-features --features strict-titles
```

回到 **平台根目录**做固定验收。下面的 `my-taskpocket-pro` 替换为你的工程路径，按进度将最后的 `1` 改为 `2`、`3`：

```powershell
cargo run --locked --offline -p labquest -- advanced-b check "my-taskpocket-pro" 1
cargo run --locked --offline -p labquest -- advanced-b next "my-taskpocket-pro"
```

B3 会累计回归 B1 / B2、真实 CLI、独立报告、默认 / strict-titles 两种配置。B1、B2 不要求先实现 B3；本地测试通过与平台固定验收是两份不同证据。

## B3 之后：真的管理自己的任务

在 **学员工程目录**，用未使用的文件名运行：

```powershell
cargo run --locked --offline -p taskpocket-pro -- add my-tasks.tsv "写一条自己的回归测试"
cargo run --locked --offline -p taskpocket-pro -- done my-tasks.tsv 1
cargo run --locked --offline -p taskpocket-pro -- list my-tasks.tsv
cargo run --locked --offline -p taskpocket-pro -- stats my-tasks.tsv
cargo run --locked --offline -p taskpocket-report -- my-tasks.tsv
```

新文件第一次新增得到 `added=1`，完成后 `total=1 open=0 done=1`。报告打印到终端，不自动保存文件。试完成不存在的 ID，并核对 TSV 没有改变。再修改 `taskcli/src/main.rs` 增加一项自己的小功能（如 search），写一条回归测试；这属于个人迁移成果。

## B4：先准备依赖，再写 JSON

B1–B3 只用本地依赖；B4 使用 Edition 2021 / Rust 1.74 与固定锁文件。新工程已含扩展；旧工程缺少 `json-export` 时，从 **平台根目录**显式运行 `advanced-b add-json <工程路径>`，已有扩展不覆盖。

在 **平台根目录**运行：

```powershell
cargo run --locked --offline -p labquest -- advanced-b prepare-json "my-taskpocket-pro"
# 完成两个 JSON 源文件后再检查
cargo run --locked --offline -p labquest -- advanced-b check "my-taskpocket-pro" 4
```

prepare-json 明确联网，外层 `--offline` 只限制平台启动。准备只运行平台固定参考源码，不运行本工程源码；日志在 `.advanced-b-dependencies/`。准备成功不等于课程通过。`all` / `learn check` 包含 B4，显式 `3` 可继续无 registry 依赖检查。

B4 三级提示：① 画 TSV → Task → DTO → JSON 数据流，标明出错时不能写文件；② 复用标题校验、用集合查重复 ID，排序 DTO 而非输入；③ 用 Serialize derive 和 `serde_json::to_string`，全部成功后 CLI 才输出 JSON 与换行。禁止静默 trim 快照标题，strict-titles 沿用 80 字符规则。

准备并实现后，在 **学员工程目录**运行：

```powershell
cargo test --locked --offline --manifest-path json-export/Cargo.toml --all-targets
cargo test --locked --offline --manifest-path json-export/Cargo.toml --all-targets --all-features
cargo test --locked --offline --manifest-path json-export/Cargo.toml --doc
cargo run --locked --offline --manifest-path json-export/Cargo.toml -- export-json my-tasks.tsv
```

用虚构任务验证中文、引号、反斜杠、`u64::MAX`、乱序 ID、80/81 字符，并比较导出前后 TSV 字节。导出不提供导入，不创建缺失数据库；缺失数据库只输出 `[]`。旧 B3 报告不冒充 B4，源码变化后必须重新检查。

## 最后的设计自查与下一步

在自己的 `LEARNING_LOG.md` 写出实际输入、输出、一次失败和原因，并回答：

1. 为什么 CLI 不直接解析 TSV，`taskreport` 不依赖 `taskcli`？
2. 数据库坏了，为什么 add 应报错而不是覆盖？
3. path 依赖怎样共享业务规则，`strict-titles` 怎样改变规则？
4. JSON 的 DTO 为什么放在适配包，id 为什么一直保持 u64？
5. 为一个公开函数补 `///` 示例，用 `cargo test --locked --offline -p taskcore --doc` 验证，再用 `cargo tree --workspace` 解释依赖。

完整平台创建工程时还会附带 `ADVANCED_B.md`，其中有完整合同与分级提示。四阶段检查通过后，提交个人任务数据、新功能、测试和设计说明。回平台可读 `course/BOOK_D1_CARGO.md` 补 Cargo；具备同步 NetFetch / AsyncBoard 基础后，按 `course/ADVANCED_C1.md` 进入真实异步下载。

当前 `save()` 是教学型写入，不保证多进程并发安全或断电一致性。平台自动测试只用于自己的可信代码，不是安全沙箱。
