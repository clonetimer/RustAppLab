# 第 13 关 · 文件筛选：从 Rust 结构体到 JSON

**中心问题**：当前 FileQuest 已能统计目录，但浏览器只需显示指定类别文件时，应该由谁负责筛选？

## 体验现有功能

先运行第 12 关的 LocalHub 服务，然后执行：

```sh
curl http://127.0.0.1:8765/api/files?limit=1
curl http://127.0.0.1:8765/api/files?limit=200
curl -i http://127.0.0.1:8765/api/files?limit=9999
```

**预期**：前两条响应具有相同的总体 `stats`，但 `shown`、`files` 长度不同；第三条是 400。文件数量与返回条数不同：`files` 是扫描结果，`shown` 是该次接口返回的数量。

## 需要理解的 Rust 知识

阅读 `apps/localhub/src/data.rs` 的 `Snapshot::load`、`files_json`、`json_string`。在 Workspace 中，`localhub` 通过 `path` 依赖复用 `filequest::scanner::scan` 的结果，而不是复制一套扫描器。理解 `&self` 借用 `ScanSummary`，为什么返回 JSON 时不应移动 `self.files`。

`json_string` 是教学用最小编码器，必须覆盖双引号、反斜杠和控制字符。前端 `apps/localhub/src/ui/app.js` 使用 `textContent`，这与 Rust 输出合规 JSON 形成两道不同的边界。

## 新能力挑战：支持 `?ext=rs`

设计：

- `GET /api/files?ext=rs` 只返回扩展名为 `rs` 的文件（但总体目录统计仍基于完整扫描）。
- `ext` 不区分 ASCII 大小写；参数值为空、未知参数或同时包含冲突参数时返回 400。
- 不允许用户通过 query 提交根目录。选定扫描目录只能来自 `serve --root`。
- `shown`、`truncated` 必须真实反映过滤后的输出，不得用未过滤结果冒充。

先主动运行失败挑战，再编辑 `src/http.rs` 和 `src/data.rs`：

```sh
cargo test -p localhub --test challenges mission_13_extension_filter -- --ignored --exact
```

**验收**：测试从失败变为通过；随后运行 `cargo test --workspace --all-targets`，此前 `/api/files?limit=N` 行为不能被破坏。若感到困难，参考 [`HINTS_12_15.md`](HINTS_12_15.md)。

下一关：[`14_log_data_api.md`](14_log_data_api.md)。
