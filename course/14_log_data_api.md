# 第 14 关 · 日志观察：复用另一款产品的领域模型

**中心问题**：浏览器能不能展示日志风险，而不自己重新实现日志解析？

## 使用现有服务

```sh
cargo run -p localhub -- serve --root localhub-playground/files --log localhub-playground/logs/candidate.log
```

在另一个终端：

```sh
curl http://127.0.0.1:8765/api/logs
```

**核对**：演示日志识别到 8 条记录、3 条 ERROR/FATAL、1 条未识别，错误记录占比是 37.50%。不配置 `--log` 时应是 `{"enabled":false}`，不是 500 错误。注意记录级错误占比不是请求失败率。

## 看清职责分离

- `apps/loglens/src/parser.rs`：格式解析。
- `apps/loglens/src/analyzer.rs`：计算记录总数、级别数量及高频消息。
- `apps/localhub/src/data.rs`：借用已分析 `Summary`，构造 JSON。
- `apps/localhub/src/ui/app.js`：从 `/api/logs` 请求数据，显示条形分布与消息列表。

你会遇到 `Option<Summary>`、`if let`、数组迭代和 `write!`。要明确：LocalHub 不负责理解日志文本，它负责把另一个 crate 的结果**以可审查的数据契约**暴露出来。

## 新能力挑战：按等级统计

支持 `GET /api/logs?min_level=ERROR`。返回现有字段，并追加 `"filtered_count": N`，它等于等级大于或等于 ERROR 的记录数（ERROR + FATAL）。没有配置日志时仍返回 `enabled:false`。无效等级返回 400。不能悄悄改变原有 `records` 的含义。

```sh
cargo test -p localhub --test challenges mission_14_logs_min_level -- --ignored --exact
```

**验收**：演示数据的 `filtered_count` 为 3，关卡测试自建日志的 `filtered_count` 为 2；异常等级 400，已有 `/api/logs` 输出兼容。完成后进入 [`15_release_gate.md`](15_release_gate.md)。
