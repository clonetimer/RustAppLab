# 06 · 把结果变成产品：离线界面、安全转义与测试

**本章中心**：业务逻辑和展示层分离，才能同时保留 CLI、HTML 与测试能力；用户可控文本必须经过正确的 HTML 转义。

## 观察

```sh
cargo run -- report filequest-playground -o lesson06.html --limit 20
```

用浏览器打开 `lesson06.html`。它是 Rust 进程生成的静态文档，筛选功能由内嵌的少量浏览器脚本提供，没有外部资源请求。

打开 `apps/filequest/src/report.rs`，关注三个函数：

- `render(&ScanSummary, limit)`：数据 -> HTML 文本；不做文件扫描。
- `escape_html(&str)`：将 `&`、`<`、`>`、引号等文本字符变为 HTML 实体；阻止文件名被解释为标签。
- `write_report(path, html, force)`：写文件，并处理覆盖行为。

## 思考一个安全问题

如果文件名叫 `<img onerror='x'>.rs`，直接拼接进 HTML，浏览器可能把它解释为标签。当前测试会在实际临时目录创建这种文件名，检查报告中的文本经过了转义。**文件名来自本机，并不意味着它天然适合放入 HTML。**

## 你的产品扩展

为 `report` 增加：

```sh
cargo run -- report filequest-playground -o lesson06-custom.html --title "我的项目"
```

并把标题用于报告主标题或副标题。标题是用户输入，不可以直接插入 HTML。需要修改 `cli.rs` 的命令契约、`lib.rs` 调用关系，以及 `report.rs` 的渲染 API；在渲染时继续使用 `escape_html`。保留现有默认标题。

**自动验收**：

```sh
cargo test --test challenges mission_06_custom_report_title_is_escaped -- --ignored --exact
cargo test
```

挑战传入 `<b>Archive</b>`，要求最终 HTML 中是可见文本，而不是可执行的 HTML 标签。

## 更完整的测试观念

测试分三层：

1. `src` 中的单元测试：例如字节格式、扩展名识别和转义。
2. `tests/acceptance.rs`：用真实目录文件验证扫描、忽略规则、覆盖安全和转义。
3. `tests/cli_smoke.rs`：像真正用户一样启动可执行程序，串联 `demo -> scan -> find -> report`。

**验证命令**：

```sh
cargo test
cargo fmt -- --check
cargo clippy -- -D warnings
```

格式化与 Clippy 还需要本地安装相应 rustup 组件，具体参见 `verification/QUALITY_GATE.md`。

**本章验收物**：**可以交付给其他人的离线报告，以及防止 HTML 注入的回归测试**。
