# 01 · 让命令行听懂你：enum 与 match

**本章中心**：学习语言分支不是为了背语法，而是为了为真实软件定义一个可维护的命令契约。

## 问题与观察

FileQuest 能理解 `scan`、`find` 和 `report`，但运行 `cargo run -- version` 会报“未知命令”。你需要为它增加用户常见的版本查询命令。

先执行下面的目标测试：

```sh
cargo test --test challenges mission_01_version_command -- --ignored --exact
```

它当前**应失败**；测试在 `apps/filequest/tests/challenges.rs` 中，不修改测试本身。

## 阅读路径

1. `apps/filequest/src/cli.rs` 的 `Command` 枚举：每个变体表达一个合法操作，变体内部的字段是一条命令需要的参数。
2. `cli::parse`：把外部 `OsString` 参数解析成有类型的操作，这里重点关注 `match action`。
3. `apps/filequest/src/lib.rs` 的 `run`：接收已解析的命令并执行，应用的各层互不混淆。

`enum` 是**一组互斥业务状态**，而不只是数字常量；`match` 强制你思考不同状态如何处理。Rust 编译器通常会提醒你没有覆盖的枚举分支。

## 本章修改任务

在 `Command` 中增加 `Version` 变体，在 `parse` 中识别 `version` 和可选的 `--version`，最后在 `run` 中输出 `FileQuest 0.1.0`。

完成标准：

```sh
cargo run -- version
cargo test --test challenges mission_01_version_command -- --ignored --exact
cargo test
```

第一条命令应输出明确版本；后两条应通过。尤其注意，在修改 `enum` 后，`run` 的匹配必须有相应分支。

## 追问和支线

- 若 `--version` 是全局命令，它应在哪一层识别？
- 如果把 `Command` 换成 `String`，运行时会失去哪些类型检查？
- 试试 `cargo run -- unknown`；为什么错误要写入 stderr 而不是 stdout？

本章验收物：**为真实 CLI 增加了一个有测试的命令**。下一章学习文件系统。
