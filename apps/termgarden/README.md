# TermGarden · 让任务随着操作真正改变

**这次要做成什么：**在终端添加任务、移动选择、切换完成状态，退出后还能继续。你的成果是一份个人任务文件，以及能解释每次操作如何改变状态的程序。

**开始条件：**完成 [BadgeForge](../badgeforge/README.md)，能读懂结构体、枚举、借用与 trait。

本目录是已实现的参考软件；你的练习代码放在下面新建的 `my-termgarden/` 中。

## 1. 看目标，创建自己的工程

在 **RustAppLab 根目录**（能看到根 `Cargo.toml`）运行：

```powershell
cargo run --offline -p termgarden -- demo reference-garden
cargo run --offline -p labquest -- studio start termgarden my-termgarden
cargo run --offline -p labquest -- learn lesson my-termgarden 1
cargo run --offline -p labquest -- learn check my-termgarden 1
```

会生成 `reference-garden/garden.tasks`、`session.txt` 和 `SESSION_REPORT.txt`。打开报告观察“输入命令 → 看板变化”；演示目录必须是新的。

学员首次检查因 TODO 未实现而 FAIL 是正常的。先看第一条失败；若是编译器或链接器找不到，则应先修好环境。

打开 `my-termgarden/START.html` 或 `TASK_CARDS.html`；只编辑 **`my-termgarden/src/lib.rs`**，每次一个 TODO。命令行外壳 `src/main.rs` 已提供，先不改。目录已存在就继续使用；新建另一份练习时换名称。

## 2. 三个检查点

| 步骤与函数 | 要完成的行为 | 本次概念 |
|---|---|---|
| TODO(1) · `parse_command` | 把输入行转为 `Event`，未知命令明确报错 | `enum`、`match`、拥有型字符串与解析 |
| TODO(2) · `apply_event` | 更新任务、稳定 ID、光标和筛选；错误不留下半成品状态 | `&mut`、`Option`、先验证后修改 |
| TODO(3) · `render` | 只读状态并绘制看板，不在绘制时读取键盘或写文件 | 共享借用、纯函数、状态与显示分离 |

每步先写一组输入和预测，再修改、保存。第一步通过后，在 **平台根目录**继续：

```powershell
cargo run --offline -p labquest -- learn lesson my-termgarden 2
cargo run --offline -p labquest -- learn check my-termgarden 2
cargo run --offline -p labquest -- learn lesson my-termgarden 3
cargo run --offline -p labquest -- learn check my-termgarden 3
```

后一步会累计回归前面的步骤。提示用 `cargo run --offline -p labquest -- learn hint my-termgarden 1`；进度用 `cargo run --offline -p labquest -- learn next my-termgarden`。详细解释见 [termgarden 讲义](../../course/studio/termgarden.md)。

## 3. 用自己的输入完成一次作品

三步通过后，选一个尚不存在的 `my-garden.tasks`，把自己的两件待办录进去。

从 **平台根目录进入学员目录**执行：

```powershell
cd my-termgarden
cargo run --offline -- play my-garden.tasks
```

在提示符后逐行输入并按 Enter：

```text
add 写一条自己的测试
add 整理作品卡
p
x
q
```

完成 `x` 后应看到 `done=1/2 growth=sprout`，第一条任务标为完成。再执行 `cargo run --offline -- play my-garden.tasks`，两条任务和完成状态应仍在；输入 `q` 退出。第二轮换成自己的真实任务标题。

## 4. 交付与解释

在 **学员目录**运行 `cargo test --offline` 回归全部测试。保存任务文件和一次操作记录；也可把命令存成文件，用 `replay <任务文件> <命令文件>` 重放。把预测、错误和解释写入 `LEARNING_LOG.md`，再补一个自己的边界测试。

用自己的话回答：为什么选择项用任务 ID 而非列表下标？为什么 `apply_event` 需要 `&mut Garden`，`render` 只需要 `&Garden`？把绘制和输入分开后，测试方便在哪里？

测试通过只说明已检查的行为符合合同；个人输入、真实产物和解释能帮助你判断学到了什么。

标题最多 80 个字符，最多 1000 项任务。这是按行回车的单人、单进程工具；`play` 支持 `board/compact` 两种视图。不要同时写同一任务文件；同目录暂存替换不等于跨进程事务或断电持久化保证。

## 下一款：BatchMeter

接下来把读取一份状态文件扩展为读取多份文本，用线程和通道练习并发。在 **RustAppLab 根目录**运行（若仍在学员目录，先 `cd ..`）：

```powershell
cargo run --offline -p labquest -- studio start batchmeter my-batchmeter
cargo run --offline -p labquest -- learn lesson my-batchmeter 1
```

继续阅读 [BatchMeter README](../batchmeter/README.md)。
