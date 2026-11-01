# TermGarden · 把任务种成一座终端花园

## 这次要交付什么

这是一款能每天使用的小型任务管理器。你输入要完成的事情，在终端里选择任务、标记完成或撤销，退出后仍能读取自己的任务文件。完成数从 0 到 1 再到 3 时，花园从 seed → sprout → bloom。它不会替你判断学习效果，也不依靠挂机计时奖励。

**先修要求：**完成 QuizDeck，理解 `Vec`、枚举、`&mut` 和 `Result`；做过 BadgeForge 会更容易理解最后的两种界面。刚完成 FirstCard 的学员先去 UnitLab，不必跳到这里。

这次是**按行输入版**，每条命令按 Enter 执行，不需要方向键即时响应、鼠标或终端原始模式。这样的输入也能保存成文件，重复播放，便于把“看起来能用”变成可测试的行为。完整 raw-mode TUI 留作后续扩展，不把这一版冒充成它。

## 先体验一个真正的软件

在完整 RustAppLab 根目录，运行参考版：

```powershell
cargo run --offline -p termgarden -- play my-garden.tasks
```

依次输入下面几行，每行按 Enter：

```text
add 学会 Rust 的枚举
add 完成自己的词频工具
p
x
f
q
```

第二次添加后选中 #2；`p` 移到 #1；`x` 把 #1 标成完成；`f` 只显示未完成任务，光标移到 #2；`q` 退出。终端中的 `>` 标记当前项，`[x]` 表示已完成。再次运行相同的 `play` 命令，两项任务仍在；筛选和光标属于本次会话，不持久化。

不要只看花园图案：打开 `my-garden.tasks` 看三列数据，再回到程序确认磁盘记录确实决定显示的内容。不要把它当作可任意编辑的普通笔记；格式损坏时软件会拒绝加载，不会悄悄覆盖。

## 创建自己的工程

在 **RustAppLab 根目录**运行：

```powershell
cargo run --offline -p labquest -- studio start termgarden my-termgarden
cargo run --offline -p labquest -- studio check my-termgarden 1
```

首次 FAIL 正常：你还没有写命令解析。打开 `my-termgarden/START.html`，用编辑器打开 `my-termgarden/src/lib.rs`。别修改参考目录 `apps/termgarden`。

目录关系：`main.rs` 负责收键盘输入、输出提示和保存；`lib.rs` 管理业务状态，是本次三个练习所在；`storage.rs` 已提供文件格式与写入，先不修改。输入经过 `parse_command` 变成 `Event`，由 `apply_event` 修改 `Garden`，最后由 `render` 生成文本界面。

## 第 1 步：把一句输入变成一个事件

修改 `src/lib.rs` 的 **TODO(1)，`parse_command`**。

`Event::Next` 只有“下一项”这个含义，不需要额外数据；`Event::Add(String)` 既说明要添加，也带着新任务的标题。不要使用一组可能互相冲突的布尔值代表命令。

| 输入 | 应返回 | 原因 |
|---|---|---|
| `n` 或 `next` | `Ok(Event::Next)` | 同一操作支持缩写 |
| `p` / `prev` | `Ok(Event::Prev)` | 上一项 |
| `x` / `toggle` | `Ok(Event::Toggle)` | 完成或撤销 |
| `f` / `filter` | `Ok(Event::Filter)` | 切换过滤条件 |
| `q` / `quit` | `Ok(Event::Quit)` | 结束循环 |
| `add 写一个测试` | `Ok(Event::Add(...))` | 标题要保存为拥有数据的 String |
| `add`、空输入、未知命令 | `Err(...)` | 不能猜测用户意图 |

先用 `trim()` 去掉行首行尾空白，再区分带标题的命令和固定命令。`strip_prefix("add ")` 返回 `Option<&str>`；`if let Some(title)` 表示只有前缀匹配时才使用标题。调用已提供的 `valid_title` 拒绝空标题、控制字符和超过 80 字符的标题。

**所有权练习：**参数 `input: &str` 只是借用这一行，函数返回后输入缓冲可以被重用。事件需要保存标题，所以这里要创建 `String`，而不是把借用的 `&str` 塞进一个要求 `String` 的枚举。

在平台根目录检查第 1 步，或进入学员目录执行：

```powershell
cargo test --offline --test milestones step1_
```

看到 `2 passed; 0 failed` 后再做第二步。预测题：`next extra` 应当执行 next 还是报错？用你自己的测试说明理由。

## 第 2 步：让状态变化，而不是打印一句“成功”

修改 **TODO(2)，`apply_event`**。它收到 `&mut Garden`，所以能改变已有花园；`render` 只收到 `&Garden`，不能随意改变任务。返回 `Ok(true)` 表示继续运行，`Ok(false)` 表示退出，`Err` 表示这次操作无效。

先只实现 Add，检查任务是否进入 `tasks`、ID 是否递增、光标是否选中新任务。再实现 Next/Prev 的首尾循环，之后实现 Toggle、Filter，最后处理 Quit。不要一次写完六个分支才测试。

光标保存的是稳定的任务 ID，不是“当前显示第几行”。比如 #1 完成后被筛掉，#2 仍是 #2，不能因为它现在位于第一行就把它误认成 #1。已提供 `visible_ids` 返回当前可见 ID；改变过滤条件或完成状态后，需要把失效的选中项移到第一个可见项，或者在空列表时设为 `None`。

**一条错误不能改变数据。**添加前先验证标题、数量和 `checked_add`；空列表 Toggle 应报错，而不是下标越界。`checked_add` 不是复杂技巧，它把整数溢出变成能处理的 `Option`。

```powershell
cargo test --offline --test milestones step2_
```

再从平台运行 `studio check my-termgarden 2`：它会同时复查第 1 步。自己重现“完成当前任务 → 仅未完成 → 全部完成 → 没有可见任务”的状态，不要只用两项未完成任务测试。

## 第 3 步：同一份状态，两种界面

修改 **TODO(3)，`render`**。它应返回一个 `String`，不是直接 `println!`，也不应该在函数内部读取键盘或文件。这样测试可以比较完整结果，主程序可以自由选择显示到终端还是保存为报告。

固定第一行 `=== TermGarden ===`；第二行包含 `done=完成数/总数 growth=阶段 view=all或open`。各任务用 `> [ ] #编号 标题` 表示选中未完成项，用 `[x]` 表示完成项；没有可见项时必须含 `(empty)`。阶段规则：0 项完成为 seed，1–2 项为 sprout，3 项起为 bloom。

调用 `visible_ids`，遍历可见任务，把每一行追加到输出。渲染不应改变光标、完成数和任务标题；对同一个状态重复渲染应得到相同文本。

现在查看文件末尾已提供的 `Renderer` 接口。`BoardView` 调用你实现的完整界面；`CompactView` 只显示摘要。`renderer` 返回 `Box<dyn Renderer>`，因为使用者在运行时选择界面，而不是像 BadgeForge 的泛型那样提前确定具体类型。暂时不用自己写虚函数表；你需要理解“拥有一个符合接口的对象”，并能够新增自己的实现。

```powershell
cargo test --offline
cargo run --offline -- play garden.tasks
cargo run --offline -- play garden.tasks compact
```

这些命令在**学员工程目录**运行。平台的 `studio check my-termgarden 3` 则必须在 RustAppLab 根目录运行。完整三步为 8 项合同测试；最终两项真实运行你的 CLI，并检查保存结果以及错误脚本不产生半份文件。

## 亲手交付，而不是只取得 PASS

用自己的 5 项任务运行两次，保存 `garden.tasks`，至少完成一项，再退出重新打开确认。把一段操作写到 `samples/mine.txt`，用 `cargo run --offline -- replay another.tasks samples/mine.txt` 重放；不要用现有个人文件做破坏实验。

扩展任务任选一项：增加 `rename 新标题` 事件，或增加 `MinimalView` 作为第三种 Renderer。先说明要改变哪层、写一个测试，再实现功能。不要删除原来的测试。

最后执行 `cargo build --offline --release`，保存自己的任务数据与 `LEARNING_LOG.md`。Windows 程序在 `target/release/termgarden.exe`。在记录里解释：为什么解析返回拥有数据的 String？为什么光标用 ID？为什么 render 不直接读键盘？

## 常见卡点与边界

“移动后借用”通常来自一边迭代借用任务、一边 push/修改列表；先算出 ID 或位置，结束只读借用，再获取可变引用。`Option` 不是可以直接作为下标使用的整数；先处理 `None`。修改后完全没变化时，检查是否编辑了学员目录而非参考目录，并确认 Ctrl+S 保存。

这是单人、单进程任务工具。不要同时打开两个进程写同一文件；同目录暂存后替换不等于跨进程事务或断电持久化保证。它不是完整即时键盘 TUI，不覆盖终端 raw mode、鼠标、焦点或无障碍测试。
