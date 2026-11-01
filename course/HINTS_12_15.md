# LocalHub 12–15 · 逐级提示卡

先阅读当前章节的输入、预期输出，再运行具体 `--ignored` 挑战；本文件不是现成答案。

## 12：真实 HTTP 响应

- 提示一：看看 `src/http.rs` 的 `Response::to_bytes`，头正文的分界是 `\r\n\r\n`。
- 提示二：Rust 的 `String::len()` 是字节长度（UTF-8），不同于显示字符数量。
- 提示三：真实 TCP 测试在 `tests/acceptance.rs` 的 `real_tcp_http_round_trip_and_graceful_limit`。

## 13：扩展名过滤

- 提示一：不要直接改变 `ScanSummary`，先考虑在 JSON 编码前用 `.iter().filter(...)` 得到符合条件的借用引用。
- 提示二：路由解析应区分 `?limit=...`、`?ext=...` 与未知参数；服务端不能由查询字符串决定扫描根路径。
- 提示三：过滤后的 `shown / truncated` 以及 `/api/files` 原有不带参数行为都需要测试。

## 14：日志级别统计

- 提示一：等级顺序已经被 `Level` 的 `Ord` 派生定义。
- 提示二：`Summary::count(level)` 可直接返回六种级别数量，不必重新解析原始日志。
- 提示三：新增字段时要保证原有 `records`、`errors`、`error_rate` 口径不变。

## 15：健康检查命名

- 提示一：当前 `/api/health` 是常量字符串，需要通过受控的启动状态带入名称。
- 提示二：顺着 `cli::Command::Serve` → `lib::run` → `http::handle_request` 修改，不要只改测试。
- 提示三：自定义名称要经过 `json_string` 转义；写一个包含引号的名称测试。启动后在真实 TCP 连接里读到名字，才算完成。
