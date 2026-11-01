# 00 · 第一次使用：先把软件玩起来

> 如果你完全不会 Rust，先做 [第 0 阶段：完成第一张任务卡](BEGINNER_WINDOWS_GUIDE.md)，只改一行代码并获得第一次成功，再回来做本章。

**核心问题**：如果你从没写过 Rust，怎样在十五分钟内辨认一个 Rust 软件的输入、处理和输出？

**产品故事**：你接手了一个不熟悉的项目，想知道里面有多少 Rust 源码、日志和 Markdown 文档，以及哪些文件最大。

## 观察

在项目根目录执行：

```sh
cargo run -- help
cargo run -- demo
cargo run -- scan filequest-playground
cargo run -- find filequest-playground src
```

浏览器打开 `filequest-playground/report.html`，在筛选框输入 `rs`。这是第一次交付物：**可读、可搜索、可以交给其他人的 HTML 报告**。

## 读一点源码

只打开 `apps/filequest/src/main.rs`。它先收集命令行参数，然后调用 `filequest::run`，最后将错误变成非零退出码。

```rust
let arguments: Vec<_> = std::env::args_os().skip(1).collect();
```

理解三个概念即可：`Vec` 是存放若干参数的集合；`args_os` 允许路径包含非 UTF-8 字节；`skip(1)` 忽略可执行文件自身的名字。为什么不直接在 `main` 里写所有业务？因为参数解析和业务逻辑要便于测试与复用。

## 修改与验收

不修改 Rust 源码，先在演示目录下创建 `docs/new-notes.md`，写入三行文字。再运行：

```sh
cargo run -- scan filequest-playground
cargo run -- find filequest-playground new-notes
cargo run -- report filequest-playground -o my-first-report.html
```

**验收**：扫描的文件数比修改前多 1；能找到新文件；新报告能在本地浏览器打开，并显示该文件。

**额外挑战**：尝试 `cargo run -- scan ./apps/filequest`，比较扫描仓库中的真实 Rust 源码与演示文件。为什么有 `target` 时默认不会让统计被大量编译产物淹没？

完成后进入 [01 命令契约](01_cli_contract.md)。
