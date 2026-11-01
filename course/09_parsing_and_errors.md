# 09 · 一条奇怪的日志：解析契约与错误处理

**中心思想**：日志不是任何字符串都能随意猜测的“自然语言”。应该先定义已支持的格式，再决定不支持的输入如何影响结果可信度。

**本关真实成果**：一个可过滤日志消息的分析能力，并明确“筛选后统计”的范围。

## 1. 观察与实验

支持的基本形式：

```text
INFO service started
2026-10-28T10:20:30Z ERROR database unavailable
[WARN] request delayed
warning: unused variable
```

未知级别、缺少消息正文、无效时间戳、非 UTF-8 文本分别有不同的处理路径。体验容错与严格模式：

```sh
cargo run -p loglens -- analyze loglens-playground/candidate.log
cargo run -p loglens -- analyze loglens-playground/candidate.log --strict
```

默认模式允许“8 条记录 + 1 条未识别”；`--strict` 必须在第 9 行中断并给出异常位置。**这里没有任何自动修复输入的承诺**。

## 2. 读源码

打开 `parser.rs` 中的 `Level::parse` 和 `parse_line`；再看 `analyzer.rs` 中的 `match parse_line(&line)`。

理解三种状态：`Ok(Some(record))` 是一条可识别记录；`Ok(None)` 是空白；`Err(reason)` 是不能可靠识别的文本。这样比把所有无效行当成 `INFO` 更可信。

如果你想添加日期格式，请明确时区、无效日期和跨时区统计语义；当前只接受 UTC `YYYY-MM-DDTHH:MM:SSZ`，并不承诺通用时间解析。

## 3. 本关 RED → GREEN 任务

团队要从多种日志中抽出 `cache` 相关消息。给 `analyze` 添加 `--contains <关键词>`，对**已经解析出的消息正文**做大小写不敏感的包含过滤。示例：

```sh
cargo run -p loglens -- analyze loglens-playground/candidate.log --contains timeout
```

要求：只针对匹配的有效日志计算 `已识别`、各级别数量、错误率和高频消息。未识别行的原始数量仍应显式报告，不得因为过滤而伪称输入完整；空关键词、没有匹配、UTF-8 参数异常都需要明确行为。

```sh
cargo test -p loglens --test challenges mission_09_filter_message -- --ignored --exact
cargo test --workspace
```

**验收**：独立测试包含 INFO / ERROR 混合输入，指定 `cache` 后只统计 1 条匹配记录。不可把只改输出文本冒充真正过滤。

下一关：[10 · 版本前后比较](10_compare_and_metrics.md)。
