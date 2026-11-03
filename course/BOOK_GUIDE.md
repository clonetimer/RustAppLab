## 2026-11-03 · C1 异步下载

新增独立 Tokio/Reqwest 回环 HTTP 下载工程、三阶段学员骨架、LabQuest 固定验收和离线 vendor 分发。基础 MSRV 继续1.74，C1单独使用1.85.0；C2 Axum 尚未交付。参见 [C1 讲义](ADVANCED_C1.md)；具体跨平台结果以当前提交的 CI 为准。下文带旧版本号的内容保留为历史路线记录。

# 用 Rust Book 理解自己的作品

先运行作品、完成当前任务，再针对遇到的问题阅读相关概念。Rust AppLab 继续以已有软件为练习载体；阅读帮助你解释设计，实际检查验证修改后的行为。

## 从当前工程出发

在 Rust AppLab 根目录执行：

```powershell
cargo run --locked --offline -p labquest -- learn reading
cargo run --locked --offline -p labquest -- learn reading "my-textscope"
cargo run --locked --offline -p labquest -- learn reading "my-taskpocket-pro"
```

第二个参数是**已有学员工程的路径**。命令识别工程标记，显示相关章节、一个思考实验以及返回检查的命令。中文和空格路径用引号包围。FirstCard、十二款 Studio、TaskPocket、PageMint、进阶 A/B 都有对应入口。

命令在本机读取工程类型，不执行学员程序、不保存成绩。参考链接由你主动打开；网络不可用时仍可查看导航和本地讲义。新生成 Studio 工程的 `TASK_CARDS.md` 和 `TASK_CARDS.html` 也附带相同参考段；已有工程可直接使用新版命令，无需重建。

## 版本怎么对应

本次参考的 [rust.bootcss.com](https://rust.bootcss.com/) 首页声明采用 **Rust 1.37.0、Edition 2018**。它适合辅助理解所有权、借用、错误处理和测试组织。Rust AppLab 的现有工程采用 **Edition 2021，最低 Rust 1.74.0**，实际设置见各工程的 `Cargo.toml`。

[当前官方 Rust Book](https://doc.rust-lang.org/book/) 使用 Edition 2024 的示例基线。阅读时保留自己工程的版本设置，遇到不同 API 或工具选项时查阅官方说明。旧中文站与当前官方书的后半部分章号不同，例如异步编程参考官方的 [Async/Await](https://doc.rust-lang.org/book/ch17-00-async-await.html) 章节。按主题查找比按旧章号跳转更可靠。

安装了 rust-docs 组件时可使用 `rustup doc --book` 阅读本机工具链对应的离线书籍。

## 概念与现有作品对照

| 主题 | 在现有作品中观察 | 阅读后做什么 |
|---|---|---|
| 变量、函数、枚举、match | FirstCard、UnitLab | 换两组输入，解释表达式如何产生返回值，再运行对应任务检查 |
| 所有权、借用、切片 | QuizDeck、TextScope | 说明返回值拥有数据还是借用原文，比较移动和借用的使用范围 |
| Result、错误传播 | LedgerLite、MiniKV | 给出合法与损坏输入，检查错误如何到达 CLI，以及失败后原数据是否保持完整 |
| 模块、trait、泛型 | BadgeForge、TaskPocket、PageMint | 解释业务库和程序入口的职责，为新规则写一个公开接口测试 |
| 闭包与迭代器 | TextScope、TermGarden | 比较循环与迭代器的输出；说明闭包读取、修改或取得了哪些数据 |
| 智能指针、内部可变性 | 进阶 A | 画出强/弱引用关系，解释借用守卫何时释放，以及跨线程为何传拥有型快照 |
| 线程、消息传递、共享状态 | BatchMeter、MiniKV | 检查最终数量和状态，允许线程输出顺序变化；解释谁拥有共享状态 |
| 测试、Workspace、feature、derive | 进阶 B | 跑默认/strict-titles，用库、CLI、报告和 doctest 验证接口；在独立 B4 适配包中用 DTO/Serialize derive 导出 JSON |
| I/O、网络、等待 | NetFetch、ServiceDesk、AsyncBoard | 分清同步阻塞、线程并行和 Future 等待；解释当前应用实际使用哪种机制 |
| 事务与回滚 | StoreDesk | 故意让批次第二项失败，检查库存和流水是否一起回滚，再解释 Rust 错误传播与数据库事务的分工 |

对照阅读时记录三项：**我预测的行为、实际得到的结果、我用 Rust 规则解释的原因**。可以先查中文概念，再用官方资料核对 API，最后回到自己的代码验证。

## 把测试组织落实到进阶 B

[中文书的测试组织](https://rust.bootcss.com/ch11-03-test-organization.html) 和 [官方对应章节](https://doc.rust-lang.org/book/ch11-03-test-organization.html) 区分单元测试与从公开接口出发的集成测试。进阶 B 用 `taskcore` 提供业务接口，由 CLI 和报告程序复用；测试既覆盖解析与状态变更，也实际运行独立进程验证输出。

在**自己的进阶 B 工程目录**执行下列命令。骨架尚未实现时出现失败是待完成任务的反馈。两条 `--test feature` 命令适用于新版平台生成、包含 `taskcore/tests/feature.rs` 的工程；旧工程可直接使用下方新版平台的累计检查，平台会提供固定 feature 合同，无需向学员工程补文件或重建工程：

```powershell
cargo test --locked --offline --workspace --all-targets
cargo test --locked --offline -p taskcore --no-default-features --test feature
cargo test --locked --offline -p taskcore --no-default-features --features strict-titles --test feature
cargo test --locked --offline --workspace --doc
```

`--all-targets` 与文档测试有各自的选择规则，因此两条命令分别执行，见 [Cargo test 官方说明](https://doc.rust-lang.org/cargo/commands/cargo-test.html)。公开函数的 `///` 示例可以作为 doctest，检查文档中的用法仍能编译运行；维护者的 `taskcore::parse_tsv` 已提供一个实例。

平台检查从早期步骤累计执行：在 Rust AppLab 根目录运行 `cargo run --locked --offline -p labquest -- advanced-b check "my-taskpocket-pro" 3`，会回归 B1、B2、CLI、独立报告以及默认/严格标题配置。单独运行某个测试方便定位；累计检查用于确认整体仍然满足合同。修改用户目录中的 tests 不会替换平台固定合同。

## 维护者复测入口

```powershell
python scripts/check_error_labs.py
cargo fetch --locked --manifest-path projects/advanced-b/reference/json-export/Cargo.toml
python scripts/release_check.py --output ../qualification-current
```

fetch 是维护者显式联网准备；后续完整门禁不自行联网。Rust 1.74 必须用实际 1.74 工具链单独准备与运行，缓存不从较新工具链结果推断。输出目录须为新目录。完整工程检查包括主工作区、进阶 B 独立参考工作区、feature 组合及文档测试。`verify.sh` 和 `verify.ps1` 提供不依赖 Python 的 Cargo 检查；Windows 完整工程记录继续使用 `VERIFY_RELEASE_WINDOWS.cmd`。普通学员只需使用 Cargo 和学习入口。

B4 在独立 `json-export/` 中复用 `taskcore`，练习 [Serde derive](https://serde.rs/derive.html) 与 [Cargo resolver](https://doc.rust-lang.org/cargo/reference/resolver.html)。旧 Book 可帮助理解 trait、模块和测试；第三方 API 以这些官方资料和固定 Cargo.lock 为准。在平台根目录显式 `advanced-b prepare-json <工程>` 后，再 `advanced-b check <工程> 4`；旧工程先 `advanced-b add-json <工程>`。`3` 保持原合同，`all`/`learn check` 包括 B4；准备不会执行学员源码，只读导航不会准备依赖。完整步骤和个人数据实验见 [B4 讲义](ADVANCED_B.md)。真实异步网络课程 C 继续按 [课程路线](CURRICULUM.md) 推进。
