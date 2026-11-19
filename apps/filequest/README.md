# FileQuest · 认识一个目录，生成自己的文件地图

FileQuest 扫描本地目录、按相对路径查找文件，并生成离线 HTML 报告。它是已经实现的参考工具，也承载第 00–07 章的源码拓展练习。

**学习位置：**刚完成 FirstCard 时，先按 [衔接页](../../course/AFTER_FIRSTCARD.md) 学 UnitLab。等你能辨认函数、输入和输出，再按兴趣回来做这条拓展；这里不会创建三个 TODO 的 Studio 工程。

## 先用自己的文件观察结果

在 **RustAppLab 根目录**运行，演示目录必须是新的：

```powershell
cargo run --offline -p filequest -- demo my-filequest-demo
cargo run --offline -p filequest -- scan my-filequest-demo
cargo run --offline -p filequest -- find my-filequest-demo src
```

浏览器打开 `my-filequest-demo/report.html`。报告来自生成时的快照；之后修改目录，旧 HTML 不会自动更新。

用编辑器在 `my-filequest-demo/docs/my-goal.md` 写三行自己的学习目标，预测文件数会如何变化，再运行：

```powershell
cargo run --offline -p filequest -- scan my-filequest-demo
cargo run --offline -p filequest -- find my-filequest-demo my-goal
cargo run --offline -p filequest -- report my-filequest-demo -o my-filequest-report.html
```

相比上一遍扫描应多一个文件；查找结果包含 `docs/my-goal.md`，新 HTML 能看到它。保存这个报告、运行命令和修改前后的观察，这就是第一份可核对的成果。

## 从使用工具到修改源码

完整教学从 [第 00 章](../../course/00_first_five_minutes.md) 开始。先在自己的仓库副本中练习，修改位置是下面列出的 `apps/filequest/src/`，不是生成的演示目录。

| 学习阶段 | 阅读或修改位置 | 完成后应能说明 |
|---|---|---|
| 00–02 · 命令与目录 | `src/main.rs`、`src/cli.rs`、`scanner::scan` | 参数怎样变成命令；Path 与字符串的用途 |
| 03–05 · 错误与数据 | `src/scanner.rs` 的 `ScanSummary`、`find` | 文件访问失败如何记录；结构体、集合、借用如何协作 |
| 06–07 · 报告与交付 | `src/report.rs`、`tests/` | 用户文件名怎样转义；报告如何验证和重复生成 |

第一次源码挑战是第 01 章的 `version` 命令；它尚待学员实现。在 **平台根目录**显式运行这一项，观察失败，再按 [第 01 章](../../course/01_cli_contract.md) 修改：

```powershell
cargo run --offline -p filequest -- missions 01
cargo test --offline -p filequest --test challenges mission_01_version_command -- --ignored --exact
cargo test --offline -p filequest
```

挑战默认忽略，所以普通测试全绿不等于已经写好 `version`、扩展名过滤或自定义报告标题。每次只做当前章的一项，保留自己的正常和错误输入。

## 已有命令与边界

| 命令 | 当前用途 |
|---|---|
| `demo [新目录]` | 创建练习文件与一份 report.html |
| `scan <目录> [--all]` | 文件数、体积、扩展名分布 |
| `find <目录> <关键词> [--all]` | 按相对路径查找，终端最多展示 100 项 |
| `report <目录> [-o 文件.html] [--limit N] [--force] [--all]` | 生成 HTML；默认最多展示 400 个文件 |
| `missions [00-07]`、`help` | 查看章节或命令帮助 |

扫描与查找只读；默认跳过 `.git`、`target`、`node_modules` 等构建目录，并跳过符号链接。报告默认拒绝覆盖；只对自己选择的小目录实验。报告与终端结果可能暴露文件路径，分享前先检查内容。

复盘时回答：为什么报告行数可能小于扫描总文件数？为什么增加文件后必须重新生成报告？为什么访问异常不应该伪装成“目录里没有文件”？

## 下一段拓展：LogLens

第 08–11 章把“目录中的文件”换成“日志中的记录”，练习解析、统计口径和错误处理。在 **平台根目录**运行：

```powershell
cargo run --offline -p loglens -- demo my-loglens-demo
cargo run --offline -p loglens -- analyze my-loglens-demo/candidate.log
```

继续阅读 [LogLens README](../loglens/README.md)；零基础主线仍按 [学习衔接页](../../course/AFTER_FIRSTCARD.md) 续学。
