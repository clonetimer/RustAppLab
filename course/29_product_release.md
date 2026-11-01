# 第 29 关 · 把 TaskPocket 交付为自己的软件

**中心任务**：用不同的真实数据验证同一份程序，而不是为教程示例写死输出。达成验收后，交付能独立运行的 Rust 可执行文件与使用说明。

## 一、先理解验收的意义

在 RustAppLab 根目录执行：

```sh
cargo run -p labquest -- app check my-taskpocket
```

`app check` 会先用 Cargo 离线构建 TaskPocket，再对同一个可执行程序执行九组真实 CLI 测试：空表、连续添加、跨进程读写、状态与幂等、多任务统计、UTF-8、错误后数据不变、损坏存储和多个数据库隔离。所有测试使用隔离的临时目录；不会覆盖你自己创建的 `todo.tsv`。

| 观察到的反馈 | 应该推敲什么 |
|---|---|
| 缺失数据库与空列表失败 | 是否把缺失路径视为零记录？是否在只读操作时创建了文件？ |
| 添加与持久化失败 | ID 是否每次从磁盘计算？是否写对了制表符与换行？ |
| 完成状态失败 | 是否只更新匹配 ID 的任务？重复 done 是否幂等？ |
| UTF-8/标题处理失败 | 是按字符处理，还是错误地用字节截断了中文？ |
| 损坏数据拒绝失败 | 是否先完整验证所有记录再进入修改阶段？ |
| 数据库隔离失败 | 是否真的使用命令行传入的路径，而非写死了固定目录？ |

评测器采用**可核对的行为合同**，不检查你是不是用了某种特定语法或算法；同样的行为可以有多种正确实现。

## 二、真实使用试验

在 `my-taskpocket/` 下独立运行你自己的工具：

```sh
cargo run -- add personal.tsv "完成 Rust 第29关"
cargo run -- add personal.tsv "使用真实数据检验 TaskPocket"
cargo run -- done personal.tsv 1
cargo run -- list personal.tsv
cargo run -- stats personal.tsv
```

再创建 `work.tsv`，保证它与 `personal.tsv` 数据完全隔离。退出终端、重新进入项目，确认数据仍在。手动打开文件解释每一行；故意输入不存在的任务 ID，比较出错前后的文件字节是否一致。

请在 `PROJECT_JOURNAL.md` 写清自己遇到的两个真实错误、相应修复及对应 Rust 概念。记录可比单纯的“完成”勾选更有价值。

## 三、独立打包

```sh
cargo fmt --all -- --check
cargo test --offline
cargo build --release --offline
```

程序位于 `my-taskpocket/target/release/`（Windows 则有 `.exe` 后缀）；带上源码、`Cargo.toml`、使用说明和你自己的测试记录作为作品交付。`APP_ASSESSMENT.md` 是自动生成的结果快照，但不代表代码无安全风险，不保证断电时原子写入，也不代表跨平台已经验证。

**通关后可做的自由扩展**：增加 `search` 命令、给任务增加截止日期、用临时文件与 rename 优化数据写入、编写你自己的单元测试。先写用户需求和验收标准，再考虑具体 Rust 技术，不强制选择某个框架。

## 四、交付判据

- 完整合同 9/9 项通过，成绩仅代表本地行为测试覆盖范围；
- Rust 源码和 Release 程序能够脱离 RustAppLab Workspace 独立使用；
- 至少一次真实任务管理操作，并有自己的开发日志；
- 能口头说明 `String` 所有权、可变借用、`Result` 和文件写入为何在这款软件中必需。

**学习成效还需实际学员独立完成后才能评价**。集成测试的通过只能说明教学工具行为符合预期，不代表已经有人在无提示条件下掌握了 Rust。


## 新版连续指导

[第五段：独立开发](INDEPENDENT_PATH.md) 提供只读→业务→完整合同三个累计行为检查点、设计记录、个人回归测试和发布清单。旧 app check 仍执行完整合同，不必重建已有项目。
