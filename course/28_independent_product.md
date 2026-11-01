# 第 28 关 · 从空白工程开发 TaskPocket

**中心任务**：离开 CASE 01 的预置源码和固定输入，独立开发一款能用在现实生活中的离线待办管理器。你写的 TaskPocket 将拥有自己的 Cargo 项目和数据文件，能在其他文件夹单独运行。

## 先领需求，不领答案

请在 `RustAppLab/` 根目录执行：

```sh
cargo run -p labquest -- app init my-taskpocket
```

它会创建一个完整但尚未实现功能的独立工程，包含 `Cargo.toml`、`src/main.rs`、`CONTRACT.md`、`PROJECT_JOURNAL.md`。若目录已存在会直接拒绝，不覆盖你的文件。**首要动作是认真阅读 `my-taskpocket/CONTRACT.md`**，弄清每条命令的输入、输出、磁盘格式和异常结果。

```sh
cd my-taskpocket
cargo run -- help
cargo run -- list todo.tsv   # 初始版本预期失败：尚未实现
```

失败是正常的：本轮不提供已完成的业务实现，学员要亲手编写。项目独立设置了 `[workspace]`，不会把你自己的程序编进原有五个应用。课程维护者的参考实现藏在测试维护资料里；建议完成本关之前不要阅读，以免替代独立思考。

## 四个可观察里程碑

| 里程碑 | 用户可见的结果 | 需要用到的 Rust 知识 |
|---|---|---|
| M1 空列表 | 找不到数据文件时，`list` 不输出，`stats` 输出三个零 | `args`、`match`、`Result`、`Path` |
| M2 添加任务 | `add todo.tsv "研究 Cargo"` 后文件出现一条 TSV 记录 | `String`、`Vec`、文件读取与写入 |
| M3 完成任务 | 重启 CLI 后 `done todo.tsv 1` 更新状态，`stats` 反映变化 | 可变借用、迭代、状态转换 |
| M4 异常不损坏数据 | 空标题、错误 ID、损坏记录返回非零退出码，原文件不变 | `?`、`Option`、`Result`、验证顺序 |

先只完成 M1；每实现一项，立刻用真实文件测试。把失败原因写入 `PROJECT_JOURNAL.md`，不要等到全部写完才调试。

## 第一次自动验收

返回 RustAppLab 根目录运行：

```sh
cargo run -p labquest -- app check my-taskpocket
```

验收会真实编译并执行你编写的程序，输出当前的通过/失败项，并在你的项目中生成 `APP_ASSESSMENT.md`。**第一次没有通过所有测试是正常的。** 先修复最早一项失败，再重复运行。

卡住时依次使用 `cargo run -p labquest -- app hint 1`、`... hint 2`、`... hint 3`。提示不会直接把完整解答粘贴进项目。验收器不是沙箱，只能用于你信任的自建代码，尤其不要用它运行从陌生网页下载的 Cargo 项目。

## 本关成果

必须产出一份能独立编译的 Rust 源码工程，能实际添加任务，并解释为什么它采用文件持久化而不是只保存进程内变量。**不要求第一天就获得 9/9**；学会逐步调试比快速抄出完整代码更重要。

下一步：[第 29 关 · 多场景验收与作品交付](29_product_release.md)。


## 新版连续指导

[第五段：独立开发](INDEPENDENT_PATH.md) 提供只读→业务→完整合同三个累计行为检查点、设计记录、个人回归测试和发布清单。旧 app check 仍执行完整合同，不必重建已有项目。
