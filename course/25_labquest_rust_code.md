# 第 25 关 · 不靠复制报告：亲手完成一个 Rust 程序功能

**中心思想**：真正的编程练习必须在源代码里实现新行为，并证明二进制程序的输出符合需求。本关只修改一处逻辑，理解迭代、借用、类型、格式化和 `Result`。

## 一、已有程序在做什么？

打开 `labquest-playground/project/src/main.rs`。不要先阅读 LabQuest 自身的数百行源码；这是你这次需要负责的独立 Rust 应用。

程序从 `config/service.conf` 读取 `timeout_ms`，遍历 12 个固定的 `LATENCIES`，对每个请求输出 INFO 或 ERROR。初始版本能执行，却**没有事件统计摘要**。

先运行（可在配置修复前或后）：

```sh
cargo run -p labquest -- test-app labquest-playground
```

**预期失败**：Cargo 可能编译成功，但工具会提示缺少第 13 行 `SUMMARY`。失败证明“能编译”并不等于“需求已实现”。

## 二、需求：新增动态摘要，不允许只填固定答案

在 `src/main.rs` 中现有 `for` 循环后、`Ok(())` 之前添加计算和打印逻辑。

完整约定：

```text
SUMMARY requests=12 errors=N
```

`N` 必须根据 `LATENCIES` 和配置的 `timeout` 算出来，而不是直接写死为 `0`。当 `timeout_ms=150` 时，`N=11`；当 `timeout_ms=800` 时，`N=0`。0.8.0 新案例的自动验收会运行当前配置，以及 150ms、400ms 和 750ms 三个迁移阈值；旧版 0.7.0 案例仍保持单配置兼容。

推荐学习路径：

1. 先观察原来 `if *latency > timeout` 如何判断错误。
2. 想办法把 `true` 的次数相加，可以使用 `for` 和可变变量，也可以使用 `.iter().filter(...).count()`。
3. 用 `println!` 的格式化输出打印结果，注意一行中间的空格、大小写和换行。
4. 运行 `test-app`；若报编译错误，阅读 Cargo 输出中的文件名、行号和类型错误；若报输出不符，检查格式约定。

## 三、三个关键 Rust 概念

| 所见代码 | 你要理解的事实 | 自查问题 |
|---|---|---|
| `LATENCIES.iter()` | 迭代时借用数据，而非把数组整体移走 | 为什么迭代元素带 `&`？ |
| `filter(...)`、`count()` | 通过组合迭代器计算符合条件的项数 | 为什么无需手工维护 `errors += 1`？ |
| `fn main() -> Result<(), Box<dyn Error>>` | 文件读取或解析失败可通过 `?` 向上返回 | 配置写成 `timeout_ms=abc` 时会怎样？ |

这里先解决实际需求，尚不需要学习 async、生命周期参数或宏设计。

## 四、用不修改配置的方式验证多种参数

先做预测，不要为了测试而反复修改已有案例配置：

```sh
cargo run -p labquest -- practice
cargo run -p labquest -- predict 150 11
cargo run -p labquest -- predict 400 6
cargo run -p labquest -- predict 750 0
cargo run -p labquest -- test-app labquest-playground
```

`test-app` 对新案例执行实际 Cargo 运行并检查多种阈值，全部通过后才记录代码证据。它不会更改当前 `service.conf` 或 `logs/current.log`。如果你已经因其他工作修改了源码或配置，仍需要更新独立的 FileQuest/LogLens 报告后才能完成整个案例。详细解释见[第 26 关](26_transfer_verification.md)。

你也可以在修改后运行 `cargo run -p labquest -- hint 5 1`、`hint 5 2`；仅当确实卡住，再使用 `hint 5 3` 查看关键表达式。

## 五、为什么状态板会让已通过的任务退回？

`test-app` 记录了**运行时的 stdout** 和当时 `project/` 的内容摘要。后续改动源码、配置或者 Cargo 文件后，原来的代码运行证据就不再代表最新版本，LabQuest 会要求重新执行 `test-app`。

这不是密码学上的签名、沙箱或不可伪造证明，只是通过文件摘要发现普通内容变化的教学工具。它把一个真实软件开发原则具体化：**提交的二进制行为、源码状态和报告必须相互对应**。

### 验收标准

- 独立小项目真实 Cargo 构建、运行成功。
- 标准输出包含 12 条原有模拟请求日志及新增的动态 SUMMARY。
- 两种不同配置得到不同、正确的错误数。
- `labquest status` 的第 5 步变为 OK，修改源码后旧代码证据失效。
- 完成这一关后返回 [`第 24 关`](24_labquest_case.md) 的最终备份和作品交付步骤。
