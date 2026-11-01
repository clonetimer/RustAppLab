# 02 · 深入文件森林：PathBuf、所有权与遍历

**本章中心**：以文件扫描为情境理解 Rust 如何管理路径数据、遍历目录并防止无意越界。

## 观察

先分别运行：

```sh
cargo run -- scan .
cargo run -- scan . --all
```

在 workspace 根目录运行时，`target/` 中可能包含很多编译产物：第一条默认跳过，第二条会纳入。真实大项目的 `--all` 可能耗时更长；两条命令都不会跟随符号链接。

打开 `apps/filequest/src/scanner.rs`，只看 `scan` 函数：

```rust
let mut pending = vec![root.clone()];
while let Some(dir) = pending.pop() {
    // 读取目录项、识别文件类型、按需入栈
}
```

这里用栈做非递归深度优先遍历。`PathBuf` 持有路径；`&Path` 是借用形式。只读扫描拿到 `fs::metadata` 长度后，存入 `FileRecord`，而不是把文件内容都读进内存。

## 你要增加的产品功能

用户有时只想看 Rust 文件，因此请为 `scan` 增加 `--ext rs`：

```sh
cargo run -- scan filequest-playground --ext rs
```

预期：显示统计仅针对 `.rs` 文件；扩展名不区分大小写。你可以先在 `cli.rs` 中扩展 `Command::Scan` 的字段和参数解析，再在 `lib.rs` 将扫描结果中的 `files` 过滤到目标扩展名。其他命令不受影响。

**注意**：这次改变的是统计口径，必须让“文件总数、总体积、类型分布、大文件”一致地反映过滤后的集合。不能只在输出时改一处数字。

## 自动验收

```sh
cargo test --test challenges mission_02_scan_extension_filter -- --ignored --exact
cargo test
```

测试使用独立临时目录，其中有两个 `.rs` 文件和一个 `.md` 文件；任务完成后结果应是 2 个。

## 思考

- 如果使用 `String` 而非 `PathBuf`，Windows 的路径处理会怎样？
- 为什么默认忽略 `target`、`.git` 等目录，而 `--all` 明确请求才纳入？
- 文件扫描为什么跳过符号链接？这能避免哪类循环或越界？

本章验收物：**真实目录扫描新增了一个有用的过滤选项**。
