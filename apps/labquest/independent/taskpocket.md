# TaskPocket：把自己的待办做成可以交付的软件

## 先看终点，不先写一大段代码

你要交付的是一个离线任务管理器。它管理的任务属于使用者，不是编译器里的固定数组。今天添加的学习任务，明天重启后还在；把一条任务标为完成，不应改掉另一条。最终交付源码、你写的测试、一个不含隐私的真实任务文件和一页使用说明。

这不是 Studio 的填函数作业。`src/main.rs` 只提供入口和错误退出，你可以自行拆成 `lib.rs`、`storage.rs` 等模块；检查器看命令行为和磁盘内容，不要求某个内部函数名。还看不懂 `Vec`、`Result`、文件读取时，先回到 QuizDeck、LedgerLite。不会因为等待其他十二款作品都完成才允许开始。

`CONTRACT.md` 是固定需求；本页解释怎样逐步落实它。不要为过测试修改合同或报告。初始程序可以编译，但业务未实现，第一次检查失败是正常起点。

## 两个目录先分清

| 当前位置 | 应执行什么 |
|---|---|
| 学员工程，有自己的 Cargo.toml | `cargo run --offline -- list samples/read-only.tsv`、`cargo test --offline` |
| 完整 RustAppLab 平台根目录 | `cargo run --offline -p labquest -- learn check "学员工程完整路径" 1` |

Windows 用户可双击平台的 `LEARN_NEXT_WINDOWS.cmd`，输入本工程完整路径。L 看当前任务，E 编辑，C 检查，S 指定检查点，G 打开本页。已有旧版工程不需要重建，平台 `INDEPENDENT_PATH.html` 与 CLI 也提供指导，不会覆盖旧代码。

## 开始前：你自己做三项决定

先在 `PROJECT_JOURNAL.md` 写下：你要管理哪类真实任务；一条 Task 有哪些字段和 Rust 类型；输入→读取→验证→业务处理→写入→显示的顺序。字段类型不要求统一，但要解释 ID 为什么不是行号、状态为什么不能只有一句打印文字。

把工作拆成可以证明的小块，而不是一开始写“完成所有命令”。建议画出三条边界：CLI 负责参数；业务负责状态；存储负责字节。先在一个文件写通，遇到重复逻辑再拆模块。每改一段先 `cargo check --offline`；类型错误时先修第一条 error，不同时重写整个项目。

## 检查点 1：只读最小功能

目标：只有 `list` 和 `stats`。现在不要求实现 `add`、`done`。先预测：不存在的文件应该算空表，还是立即创建？已有记录乱序时，显示顺序按什么？`DONE` 怎样影响统计？把答案写下来再运行。

本工程附带 `samples/read-only.tsv`，没有表头，两条记录的 ID 是 8、3。不要改它来适配你的输出。实现后在学员目录运行：

```powershell
cargo run --offline -- list samples/read-only.tsv
cargo run --offline -- stats samples/read-only.tsv
cargo run --offline -- stats samples/not-created.tsv
```

应先按 ID 输出 `3 OPEN 写自己的测试` 和 `8 DONE 复盘`（列之间是制表符），第二条命令是 `total=2 open=1 done=1`；缺失文件统计为全零，而且不应创建这个文件。`list` 的空表 stdout 是空文本，不是“没有任务”。调试信息临时放 stderr，避免破坏 stdout 协议。

实现时想清楚：读取失败不一定都是文件不存在。把权限错误当空表，后续保存可能覆盖本来存在的数据。先分清 `NotFound`，其他失败通过 `Result` 返回。`Vec<Task>` 可以拥有字符串；不必为了省一行 `.to_owned()` 把临时文件内容的借用带到整个程序中。

在平台执行检查点 1。它会验证缺失文件与已有数据的读取、排序、统计、不改写；尚未实现的修改命令不会阻止你通过这一点。通过后不是整款产品完成。

## 检查点 2：把任务完整保存下来

目标：`add`、`done` 加入已有只读功能，数据跨进程保留。先预测下面五次操作的 ID、状态和总数，然后在学员目录使用一个新的练习路径：

```powershell
cargo run --offline -- add samples/my-week.tsv "阅读借用章节"
cargo run --offline -- add samples/my-week.tsv "写自己的测试"
cargo run --offline -- done samples/my-week.tsv 1
cargo run --offline -- done samples/my-week.tsv 1
cargo run --offline -- stats samples/my-week.tsv
```

预期依次是 `added=1`、`added=2`、两次 `done=1`、`total=2 open=1 done=1`。这里每条命令都是新进程，不能靠全局变量维持结果。关闭终端，再运行 list；用文本编辑器打开 TSV，检查实际状态。将另一个文件作为数据库，应该有自己的 ID，不受前一个文件影响。

设计难点不是调用 `fs::write`，而是什么时刻可以写。先读取与验证旧数据，验证参数，修改内存中的目标项，最后保存。新增 ID 取最大 ID 加一，不是 `len()+1`；考虑 `u64` 极限时返回错误而不是发生溢出。完成状态应幂等：重复 done 不新增记录，不把状态反转。

检查点 2 累计覆盖正常生命周期、动态统计、中文及 trim、两个文件隔离。遇到失败先用报告里的输入重现；只修一个原因，再回归已完成的功能。不要用固定 stdout 掩盖磁盘内容错误。

## 检查点 3：保护原有数据，执行完整合同

先复制练习数据，再试空白标题、制表符、零或不存在的 ID、重复 ID、损坏状态、未知命令。对每个失败，写下三件事：退出码是否非零、stderr 是否有解释、原文件字节是否完全保留。只看“报错了”不足以判断正确。

本阶段不增加新的 CRUD 命令，而是补齐已有行为的失败路径。先读取全部旧数据并验证；不要边读一行边重写原文件。对任意真实 I/O 失败都保证恢复、原子写入、多进程锁和断电一致性是后续可靠性专题，本合同不声称已经解决这些问题。

```powershell
# 在平台根目录；把路径替换为自己的实际工程目录
cargo run --offline -p labquest -- learn check "D:\RustLearning\my-taskpocket" 3
cargo run --offline -p labquest -- app check taskpocket "D:\RustLearning\my-taskpocket"
```

两条命令使用同一套完整合同探针。后者保留旧接口，不是更简单的捷径。新报告在 `.app-reports`；第三点还写 `APP_ASSESSMENT.md`。每次真实构建并运行，不从历史报告兑换 PASS。

## 你必须自己补一项测试

默认 `cargo test --offline` 仅检查“缺少参数应失败”，并跳过未写好的个人回归测试。这证明测试入口可用，**不证明产品正确**。

打开 `tests/self_check.rs`，把 `my_regression` 的 `todo!` 换成自己的 Arrange–Act–Assert：在临时目录准备已有数据，启动程序传入一个非法 ID，断言退出失败且文件字节不变。可用 `Command`、`std::fs`，具体组织方式由你决定。不要测试自己的真实待办。

先保留 `#[ignore]`，显式运行这项测试；实现好后删除 ignore，让常规测试包含它：

```powershell
cargo test --offline --test self_check my_regression -- --ignored
cargo test --offline
```

完整参考实现也不会自动完成你自己的测试作业。平台合同检查不会运行你的测试；你要单独保留它的运行记录。详细方法见 `TEST_PLAN.md`。

## 交付，不止于绿色结果

使用自己的非敏感任务，至少覆盖新增、完成、重启、第二个数据文件。按 `RELEASE_CHECKLIST.md` 写使用说明、编译 Release，并在不同工作目录用完整路径运行你的程序。Windows 可执行文件通常在 `target\release\taskpocket.exe`，Linux 无 `.exe`；以 Cargo 实际 target 配置为准。

保存源码、测试、示例 TSV、命令输出、`PROJECT_JOURNAL.md`，不要只交 `target` 缓存。记录使用过的提示；完成的是真实行为，不是“完全没看过帮助”的表演。下一款 PageMint 会改变数据流与输出形式，检验能否复用方法而不是复制任务列表代码。
