# 08 · 故障侦探入职：第一次分析真实日志

**中心思想**：学习 Rust 的文件 I/O，不是为了熟悉 `BufReader` 这个名字，而是要回答“新版本上线之后，错误是否变多了”。

**本关真实成果**：在项目里留下一个可重复执行的日志统计命令，以及一份标注统计口径的基线数据。

## 1. 体验产品（无需修改源码）

在 RustAppLab 根目录执行：

```sh
cargo run -p loglens -- demo
cargo run -p loglens -- analyze loglens-playground/baseline.log
cargo run -p loglens -- analyze loglens-playground/candidate.log
```

观察：第一份日志共有 **8 条可识别记录，1 条 ERROR**；第二份有 **8 条可识别记录，3 条 ERROR，另有 1 条未识别文本**。如果默认目录已经存在，用 `cargo run -p loglens -- demo my-logs-2` 新建，不要直接覆盖真实文件。

**先预测**：两份文件各有 8 条有效记录。ERROR 所占比例应该分别为多少？为什么第二份的总行数是 9？

## 2. 源码地图

先读 `apps/loglens/src/main.rs`（7 行左右），再读 `analyzer.rs` 的 `analyze_file`：

```rust
let reader = BufReader::new(file);
for (index, item) in reader.lines().enumerate() {
    let line = item.map_err(|err| line_error(path, index + 1, &err))?;
    // 解析一行，更新汇总结果
}
```

- `File::open` 获取文件句柄；`BufReader` 缓冲读取，避免每读一行就单独进行一次系统调用。
- `reader.lines()` 按行迭代；每次消费一行，而不是 `read_to_string` 把完整文件先装进内存。
- `Result` 保存实际错误信息；`?` 可以把无法继续处理的 I/O 错误传到上层。
- 文件解析不保存全部日志行，但当前版本会保存不同消息的计数，极高基数日志仍可能使用较多内存。

## 3. 本关 RED → GREEN 任务

日志里消息种类很多时，用户想按需只看前 N 种高频消息。为 `analyze` 增加 `--top N`，复用 `Summary::top_messages(limit)`。

```sh
cargo test -p loglens --test challenges mission_08_analyze_top_count -- --ignored --exact
```

修改前应失败；修改后运行：

```sh
cargo run -p loglens -- analyze loglens-playground/candidate.log --top 1
cargo test -p loglens --test challenges mission_08_analyze_top_count -- --ignored --exact
cargo test --workspace
```

**验收**：高频消息只打印 1 条；总行数、级别分布、错误率不能因展示行数减少而变化。需校验 0、负数、非数字、过大数值的输入行为。

## 4. 案件记录

记录一次运行的路径、识别行数、错误率和未识别行数；结果必须来自程序，不能手填“预期值”冒充运行证据。

下一关：[09 · 异常与解析契约](09_parsing_and_errors.md)。
