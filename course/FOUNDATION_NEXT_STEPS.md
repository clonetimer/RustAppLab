# 第二段实战：从文本工具到能保存、能展示、能交互的软件

前置条件：已完成 FirstCard、UnitLab、QuizDeck、TextScope 的基本练习。这里不要求熟悉 SQLite、Tokio、WASM，也不用先安装 Python。Windows 学员在源码根目录双击 `LEARN_NEXT_WINDOWS.cmd` 可继续旧工程；选择 P 查看多个工程的进度线索。

**这三款不是并行作业**：建议先 LedgerLite（真实数据）、再 BadgeForge（可视化产物）、最后 TermGarden（状态与交互）。每款三步，不必一次写完。每一步先预测，再只改一个 TODO，用 `learn check` 检查，最后用自己的数据运行实际 CLI。

## A. LedgerLite：让记录真正留在磁盘

**目的**：制作自己的简单开销账本，而不是再写一个把样例输入算出固定数字的函数。数据采用 TSV，金额用 `u64` 整数分，避免浮点舍入。它是教学工具，不是正式财务记账系统。

在 **平台根目录** 执行：

```powershell
cargo run --offline -p labquest -- studio start ledgerlite my-ledgerlite
cargo run --offline -p labquest -- learn lesson my-ledgerlite 1
cargo run --offline -p labquest -- learn check my-ledgerlite 1
```

第一步实现 `parse_cents`。先预测 `0.1`、`0.2` 的整数分为何是 10、20；再试 `1.234`、负数、整数溢出，应被拒绝而不是悄悄取整。**第二步**实现 `parse_entries`，把每行 `<日期> TAB <分类> TAB <金额>` 解析为 `Entry`。需要拒绝不存在的日期和缺列的行，并指出行号。**第三步**实现 `totals`，按分类累加，用 `checked_add` 防止溢出。逐步检查 1、2、3。

完成三步后，在 **自己的 my-ledgerlite 目录**执行：

```powershell
cargo run --offline -- add my-expenses.tsv 2026-10-30 餐饮 12.50
cargo run --offline -- add my-expenses.tsv 2026-10-30 交通 2.00
cargo run --offline -- summary my-expenses.tsv
```

预期应有餐饮 `12.50`、交通 `2.00`、总额 `14.50`（显示顺序依分类排序）。退出终端、重新运行 summary，结果应保持。把日期改成 `2023-02-29` 做异常实验，检查原始 TSV 是否保持不变。**提交**：自己的账本、两次正常运行、一条拒绝错误、简短说明。

思考：`Result` 的错误为什么比返回一个代表失败的金额更安全？

## B. BadgeForge：让自己的代码生成可打开的作品卡

**目的**：不靠截图，也不用设计软件，直接生成一张 SVG 矢量卡，浏览器就能打开。重点是安全地把用户文字放进 XML，再借助 trait 组合内容。

```powershell
cargo run --offline -p labquest -- studio start badgeforge my-badgeforge
cargo run --offline -p labquest -- learn lesson my-badgeforge 1
cargo run --offline -p labquest -- learn check my-badgeforge 1
```

第一步实现 `escape_xml`，预测 `A&B <Rust>` 必须显示为文本，不能变成标签。注意 `&` 的转义顺序，否则已经转义的文本会被重复转义。第二步实现 `Element for Badge::render`，并调用自己的第一步，而不是手写安全字符替换。第三步用泛型 `document<T: Element>`，确保不仅 Badge，其他实现 Element 的图形也能输出完整 SVG。

完成后，在自己的工程运行：

```powershell
cargo run --offline -- make "我的 Rust & CLI" "学习记录" my-first-badge.svg
```

然后打开 `my-first-badge.svg`，验证中文、& 等特殊字符显示正确。重复同一命令应拒绝覆盖文件，这是为了保护已经生成的作品。**提交**：SVG、自己的文字、一个特殊字符实验、说明为什么使用 trait。

思考：如果以后画的是圆形、柱形图，为什么不需要为每种形状重新实现 `document`？

## C. TermGarden：从一次执行升级为持续交互

**目的**：让任务列表根据每次事件改变状态，能显示、保存、再次读取，而不把解析、业务和界面都塞进 main.rs。

```powershell
cargo run --offline -p labquest -- studio start termgarden my-termgarden
cargo run --offline -p labquest -- learn lesson my-termgarden 1
cargo run --offline -p labquest -- learn check my-termgarden 1
```

第一步实现 `parse_command`：`add 写测试` 应成为 `Event::Add`，而 `next extra` 应报错。第二步实现 `apply_event`：添加、选中、完成/撤销、只看未完成；记住选中的是稳定任务 ID，不是可见列表的索引。第三步实现 `render`：读 `&Garden` 生成文本，不应该改变数据，也不在这里读取键盘或写文件。

完成后，可先在自己的项目 `samples` 建立 `my-session.txt`：

```text
add 写一份 Rust 单元测试
add 整理今天的学习笔记
p
x
q
```

再在自己的工程目录运行：

```powershell
cargo run --offline -- replay my-garden.tasks samples/my-session.txt
```

预期退出后保留 `.tasks` 文件，重新读取能看到两项任务，其中一项已完成；也应能展示 `done=1/2`。把脚本末尾加入未知命令做异常实验，输出目录使用全新路径，确认失败后没有部分保存。**提交**：自己的操作脚本、保存的任务文件、一次视图输出和关于状态转换的说明。

思考：为什么绘制界面只借用状态，就更容易从终端迁移到 GUI？

## 每一步卡住怎么办？

查看 `learn lesson <学员目录> <步数>` 给出本次要改的具体函数；`learn hint <目录> 1 <步数>` 先给概念提示，按需要再使用第 2/3 层；`learn check <目录> <步数>` 才会真正编译运行。Windows 的 `LEARN_NEXT_WINDOWS.cmd` 可以用菜单完成这些动作。将第一条错误留在学习日志里，不要一次把三个 TODO 全部替换为参考实现。

## 教师观察与最终迁移

不要因为三步测试通过就直接判定“独立掌握”。让学员用自己的数据完成交付，并现场解释：修改一个输入会改变什么？出错时状态或文件是否保持不变？他们参考了哪一级提示？

学完三款，进入 BatchMeter 体验并发，随后根据基础选择 TaskPocket/PageMint 的独立开发；现在**不要求**先掌握 Tokio、SQLite 或第三方 GUI 框架。学习记录可填写 `course/LEARNING_LOG.md`。


### 0.18.0-dev.5 · 第三段衔接

BatchMeter → MiniKV → NetFetch → ServiceDesk 已补齐 12 张微任务卡和 36 条分层提示，总覆盖为 10 款/30 张/90 条。完整路径见 [SYSTEMS_PATH.md](SYSTEMS_PATH.md)。新建工程有 TASK_CARDS.md，旧工程可直接使用新版 learn lesson/hint，无需覆盖。StoreDesk/AsyncBoard 的额外微任务卡尚未完成；成熟异步网络、桌面和 WASM 仍未完成。
