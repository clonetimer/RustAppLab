# LogLens · 08–11 关卡分级提示

建议先让对应 `#[ignore]` 挑战失败，再看提示。不要直接修改测试，让它通过后必须能用真实文件执行。

## 08 · `analyze --top N`

- **A**：`print_summary` 目前直接写死 `top_messages(5)`。
- **B**：要从 `cli::parse` 开始把参数传到 `Command::Analyze`，不能只增加打印逻辑。
- **C**：复用 `report` 中已有的 `--top` 参数校验思路；显示条数不应改变汇总的记录数量。

## 09 · `--contains`

- **A**：过滤应使用 `LogRecord.message`，不要把完整的 `Summary` 转成字符串再搜索。
- **B**：`analyze_file` 已逐行解析，可以让它接受过滤选项或引入一个简单 `AnalyzeOptions`。
- **C**：明确“已识别”统计是指过滤后匹配项，未识别行仍需要保留诊断信息。

## 10 · `--fail-on-regression`

- **A**：现有 `compare` 已经计算 `current.error_rate() - previous.error_rate()`。
- **B**：参数应通过 `cli::parse` 传入业务分支，再决定退出码。
- **C**：没有有效记录不能用 `0%` 伪装成可比较的数据。主动区分“不适用”和“错误率未升高”。

## 11 · `--title`

- **A**：从 `Command::Report` 增加一个 `Option<String>`；其他路径继续采用默认标题。
- **B**：不要通过报告成品字符串全文替换硬编码标题，应修改 `render` 的参数契约。
- **C**：传入标题时必须调用 `escape_md`，已有测试用例会尝试注入 HTML 标签。
