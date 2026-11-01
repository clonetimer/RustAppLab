# 第 26 关 · 同一份 Rust 代码能应对不同输入吗？

**中心思想**：一个程序在当前样例上成功，不等于它实现了需求。我们用同一份真实编译运行的 Rust 源码，在不同 `timeout_ms` 下验证通用行为；不修改用户正在调查的日志和配置文件。

## 一、发现原先容易蒙混过关的方式

从项目根目录创建**全新**的案例（已有案例请换名称）：

```sh
cargo run -p labquest -- start transfer-case
cargo run -p labquest -- practice
```

先思考：如果你将 `project/config/service.conf` 的 `timeout_ms` 设置为 800，直接在 `project/src/main.rs` 中写 `println!("SUMMARY requests=12 errors=0");`，程序是否会编译成功？是否在该阈值下得到正确统计？如果阈值改成 150 呢？

在这个**刚创建的练习案例**中可以尝试写死 `errors=0`，并将配置改为 800，运行：

```sh
cargo run -p labquest -- test-app transfer-case
```

**预期**：真实 Rust 编译可以成功，当前配置输出也可能正确，但后续迁移场景会失败，因此**不会生成新的 `PROGRAM_PROOF.txt`**。这不是格式错误，而是没有满足“统计结果必须依赖输入”的需求。

## 二、先预测，后运行

无需运行学生程序，也无需修改配置，即可先做边界预测：

```sh
cargo run -p labquest -- predict 150 11
cargo run -p labquest -- predict 400 6
cargo run -p labquest -- predict 750 0
```

工具会核对 12 个固定延迟数据，并解释 `latency_ms > timeout_ms` 的严格边界。注意恰好等于 400ms 或 750ms 的请求不是 ERROR。你也可以提交错误预测，看看反馈如何帮助定位 `>` 和 `>=` 的混淆。**预测不写入磁盘，不计入六步通关分数。**

## 三、实现真正的动态逻辑

打开 `transfer-case/project/src/main.rs`。在现有的 `for` 循环后添加 Rust 代码，使摘要的 `errors` 来自计算。可以用普通循环累加，也可以尝试 `LATENCIES.iter().filter(...).count()`。

你只需要改一个独立的小型 Rust 应用，不需要理解 LabQuest 源码。`test-app` 对 **0.8.0 新案例**的验收为：

| 运行方式 | 程序应输出 | 学习点 |
|---|---|---|
| 读取当前配置 | 12 条日志 + 动态 SUMMARY | 文件读取和错误传播 |
| `--timeout-ms 150` | `errors=11` | 迭代器与条件判断 |
| `--timeout-ms 400` | `errors=6` | 400ms 相等边界 |
| `--timeout-ms 750` | `errors=0` | 750ms 相等边界 |

这四次运行均针对相同 Rust 源码：先用 Cargo 真实编译一次，再顺序运行相同二进制程序四次；**测试不会修改你的 `project/config/service.conf`，也不会更新 `logs/current.log`**。全部通过后才保存 `logs/program.log` 和 `deliverables/PROGRAM_PROOF.txt`。新案例的证据头为 `LabQuest-Program-Proof-v2`。

完成代码后再次运行 `test-app`，然后用 `status` 查看第 5 步。由于改动了源码，FileQuest 地图和 SafeSync 快照必须与最新 `project/` 对齐，参见[第 24 关](24_labquest_case.md)。

## 四、旧案例兼容

0.7.0 创建的案例标识为 `Case-v1`，仍使用原有的**单配置验收**，旧证明也可以继续检查；0.8.0 新建的是 `Case-v2`，执行上述四种输入验收。我们没有悄悄改写旧练习目录，也没有宣称旧证明已经完成了迁移测试。想学习本关，请新建案例，而不是手工修改 `.labquest-case` 标识。

## 五、学到什么才算完成

你应当能够不用复制固定答案，解释 `iter()` 如何借用数组、`filter()` 如何保留满足条件的元素、`count()` 为什么无需手工计数，以及为什么等于阈值时必须区分 `>` 和 `>=`。

**边界**：该验收能发现常见的写死输出和比较符错误，但不是对任意输入的形式化正确性证明，也不是执行未知代码的安全沙箱。`test-app` 只用于你自己创建并检查过的练习案例。
