# 11 · 交出证据，而不是一张截图：Markdown 报告

**中心思想**：软件结果只有在可核对、可分享且不会无意覆盖数据时，才具有交付价值。

**本关真实成果**：一份可以提交到团队 Issue / PR 的离线日志报告。

## 1. 体验产品

```sh
cargo run -p loglens -- report loglens-playground/candidate.log -o candidate-analysis.md --top 5
```

用文本编辑器打开 `candidate-analysis.md`。文件包括输入路径、记录总数、识别/跳过数量、级别分布、常见消息、可用 UTC 时间范围和未识别行样例。

再运行相同命令应拒绝覆盖；显式 `--force` 才会替换**普通文件**。符号链接与目录仍不允许覆盖。

## 2. 源码与安全讨论

读取 `report.rs` 中的 `render`、`escape_md` 和 `write_report`：

- 报告渲染不负责日志解析，只使用 `Summary`；
- 日志正文可能包含 `<script>`、`|` 等文本，生成 Markdown 表格时需要转义；
- 报告默认禁止覆盖，但当前强制写入并非事务式的原子替换，断电等情况下仍可能留下不完整报告。

## 3. 本关 RED → GREEN 任务

添加 `report --title <标题>`，并将用户标题显示在报告主标题。复用 `escape_md`，不能使用户输入变成 Markdown 中的原始 HTML。

```sh
cargo test -p loglens --test challenges mission_11_report_custom_title -- --ignored --exact
cargo test --workspace
```

**验收**：输入 `<b>build</b>`，报告里必须出现 `&lt;b&gt;build&lt;/b&gt;`，不能出现原样的 `<b>build</b>` 标签。

## 4. 交付检查

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
cargo build --workspace --release
```

上传或提交报告前检查文件路径和日志内容，里面可能含真实系统信息、用户数据或凭证。本程序只做本地处理，不等于报告天然可公开。

走完 00–11 的主线，你已完成两款用途不同、可分别打包的 CLI 软件；接下来才考虑 HTTP/异步应用，而不是把两个现有工具强行揉成巨型单体。
