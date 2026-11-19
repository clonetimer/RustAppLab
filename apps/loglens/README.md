# LogLens — 本地日志分析器

Rust AppLab 的第二款独立应用。**当前是教学开发版源码，不等于已验证的 Stable 二进制。**

## 学习位置与本次成果

这是第 08–11 章的拓展参考应用，适合已做过文本解析与集合练习的学员。刚完成 FirstCard，先到 [UnitLab 衔接页](../../course/AFTER_FIRSTCARD.md)；这条拓展通过修改自己的仓库副本学习，不使用 `studio start loglens`。

| 观察点 | 源码位置 | 你要带走的能力 |
|---|---|---|
| 一行怎样成为记录 | `src/parser.rs` · `parse_line` | 区分 `Ok(Some)`、空行 `Ok(None)` 与解析错误 |
| 多行怎样汇总 | `src/analyzer.rs` · `analyze_file`、`Summary` | 逐行 I/O、集合、统计分母与 `Result` |
| 怎样交付可读证据 | `src/report.rs` · `render`、`write_report` | 借用统计结果、Markdown 转义、文件输出 |

完成时保存一份自己的日志、一次对比或报告和统计解释；运行参考命令只是体验，源码挑战是下一层任务。

## 真实用途

在部署前后比较日志，快速回答“新版本 ERROR/FATAL 的记录占比是否变化了”，并将调查证据保存为 Markdown。它不会读取服务器 API、不会上传日志，也不会试图自动推断真实故障原因。

```sh
# 从 RustAppLab 根目录执行
cargo run --offline -p loglens -- demo
cargo run --offline -p loglens -- analyze loglens-playground/baseline.log
cargo run --offline -p loglens -- analyze loglens-playground/candidate.log
cargo run --offline -p loglens -- compare loglens-playground/baseline.log loglens-playground/candidate.log
cargo run --offline -p loglens -- report loglens-playground/candidate.log -o my-log-report.md
```

**例子说明**：演示旧日志有 8 条有效记录与 1 条 ERROR；新日志有 8 条有效记录、3 条 ERROR、1 条无法识别的堆栈文本。错误占比从 12.5% 变为 37.5%，增加 25 个百分点；由于样本与业务量未严格对齐，这不等于服务实际故障率的变化。

## 亲手改变输入，再做一个源码挑战

在 **平台根目录**用编辑器保存 UTF-8 文件 `my-learning.log`：

```text
INFO 开始学习
ERROR 我的测试失败
这不是支持的日志格式
```

先预测，再在 **平台根目录**运行：

```powershell
cargo run --offline -p loglens -- analyze my-learning.log
cargo run --offline -p loglens -- analyze my-learning.log --strict
cargo run --offline -p loglens -- report my-learning.log -o my-learning-report.md
```

普通分析应有 2 条识别记录、1 条 ERROR、1 条未识别，错误占比 50%；严格模式应在第 3 行报错。解释为什么未识别行不进入分母，保存报告与自己的预测。

源码从 [第 08 章](../../course/08_first_log.md) 开始：在自己的仓库副本修改 `apps/loglens/src/cli.rs` 和 `src/lib.rs`，为 analyze 增加 `--top N`。**当前只有 report 已支持 --top；analyze 的这个选项是待实现挑战。** 先运行：

```powershell
cargo test --offline -p loglens --test challenges mission_08_analyze_top_count -- --ignored --exact
cargo test --offline -p loglens
```

第一条修改前应失败，修改后应通过；第二条回归已有功能。挑战默认忽略，不能用普通测试全绿替代自己的实现与解释。

## 输入契约

当前识别 UTF-8 单行文本：

```text
INFO service started
[WARN] disk is slow
2026-10-28T10:20:30Z ERROR timeout reading cache
warning: deprecated API
```

不支持跨行堆栈语义、JSON Lines、任意日志框架格式或本地时区推算。默认非严格模式会跳过不支持的非空行并显示未识别数量；`--strict` 第一个非法行即退出。时间窗口仅对识别到的 UTC `YYYY-MM-DDTHH:MM:SSZ` 时间戳求最小/最大值。

**统计口径**：记录级错误率 = `(ERROR + FATAL) / 已识别日志记录数`。未识别行不进入分母；不是请求失败率、CPU/内存故障率或事件因果分析。

## 质量与风险边界

- 文件输入逐行读取，不把原始文件一次性载入内存；但不同日志消息的聚合映射没有容量上限，高基数日志仍可能消耗大量内存。
- 日志中的控制字符在终端展示时转成替换字符，Markdown 表格单元格对 `|`、尖括号等进行转义。
- 写报告默认拒绝覆盖，`--force` 只允许普通文件，并拒绝将报告写回输入日志路径；**当前强制覆盖不保证原子更新**，操作系统级并发修改仍存在竞争条件。
- 日志可能含路径、口令或用户数据。工具本身不上传数据，但导出的报告需要在分享前人工审查。
- 本版本实现不依赖第三方 crate，可在离线 Rust 环境构建；平台 CI 仍需真实执行结果。

教学入口：[`course/08_first_log.md`](../../course/08_first_log.md) 至 11 章。`missions` 命令可查看任务路径。

## 下一段拓展：LocalHub

接下来把文件与日志结果提供给浏览器，观察 crate 复用、JSON 和本机 HTTP。在 **平台根目录**运行：

```powershell
cargo run --offline -p localhub -- demo my-localhub-demo
```

按 [LocalHub README](../localhub/README.md) 启动服务，或回到 [零基础主线](../../course/AFTER_FIRSTCARD.md) 继续当前作品。
