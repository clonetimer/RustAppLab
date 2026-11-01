# 进阶 A：把 TermGarden 改成可共享、可撤销的工作区

**目标不是重新学一遍语法。** 你已完成基础作品，现在接到一个真实需求变更：两个编辑入口要看同一份任务；只读视图不应阻止关闭；批量改名失败不能留下半批修改；耗时报告要在后台生成，而界面还能继续编辑。最终交付自己的任务文件、两种报告、回归测试及设计说明。

本单元只扩展已有 TermGarden，不增加新的领域软件。原 `termgarden play` 的自动保存行为不变；进阶 `workbench play` 为防止意外改写，只有显式 `save` 才写回。新实验在新目录进行，不修改你已经通过的基础工程。

## 先体验，再决定数据如何被拥有

先修：完成 TermGarden、BatchMeter；理解 `Result`、`&T`、`&mut T`、`trait` 和基本线程。基础较弱的同学不需要跳过旧课程。没有固定学习天数。

在**平台根目录**（有17个包的那个 Cargo.toml）运行：

```powershell
cargo run --locked --offline -p termgarden --example workbench -- demo my-advanced-demo
cargo run --locked --offline -p termgarden --example ownership_lab
cargo run --locked --offline -p labquest -- advanced start my-workbench
```

打开 `my-advanced-demo/SESSION.txt`，会看到批量改名、撤销、无效修改拒绝、关闭后视图失效。`reports/board.txt` 和 `summary.txt` 来自两个真实工作线程，`snapshot.tasks` 是同一版本的数据。后台生成报告不是异步 I/O，也不宣称更快。

接着打开 **my-workbench/START.html**，只编辑 **my-workbench/src/lib.rs**。这是完整接口开发任务，不再是三处一行填空；骨架给出了对外契约和旧业务基础设施，没有提供核心进阶实现。你可划分 `src/` 子模块，但保持公开接口。当前单元固定使用本地 termgarden 依赖；第三方依赖工程留到 B 单元。

对照三个设计，不先决定“智能指针越多越好”：

| 方案 | 数据由谁拥有 | 适用情况 | 代价 |
|---|---|---|---|
| 单一 `Garden` + 稳定 ID | 控制器持有整个任务集合 | 当前只有一个事件循环和统一入口 | 最简单；用 `&mut Garden` 编辑即可 |
| `Rc<RefCell<Session>>` | 多个编辑句柄共同拥有会话 | 同线程多个入口需要独立持有句柄 | 运行时借用检查；必须控制借用作用域 |
| 不可变拥有型 `Snapshot` | 一份独立的数据副本 | 后台生成报表，接受捕获时版本 | 复制数据有成本，结果不是实时视图 |

本实验有意同时使用后二者，**并不把原有 ID 模型替换成每条任务一个 Rc**。任务仍保存在单个 Vec 内，以 ID 选择。实际产品只有一个控制器时，第一种方案往往足够。

## A1 所有权、共享视图与借用接口

### 先给生命周期画边界

`Workspace` 句柄共同拥有 `Session`；`Session` 拥有 `Garden`。`View` 的文档引用只观察，它持有 `Weak`，不拥有文档。自定义 Renderer 也不应暗中保存一个 Workspace 强句柄，否则会重新延长文档寿命。`Snapshot` 拥有一份克隆的 Garden。先在 DESIGN_REVIEW.md 画出这四者的箭头，并标注强/弱/普通借用。

需要完成 `Workspace::new/borrow/snapshot/view`、`View::draw`、`Snapshot::select` 和 `render_with`。`validate` 是提供的业务校验；错误词汇已提供，不要用 panic 代替可恢复的错误。

**预测一：** `let b = a.clone()` 后，修改 b 应不应该影响 a？此处 clone 的是句柄，两个编辑入口必须指向同一份 Session。若复制了底层 Garden，就成了两个互不相干的文档。

**预测二：** 丢弃 a，但 b 仍然存在，视图能否读取？再丢弃 b 后呢？`Weak::upgrade` 的 `Some/None` 正好表达“文档还在/已关闭”，无需保留一个永久有效的裸指针。

**预测三：** 为什么 `borrow` 返回 `Ref<'_, Garden>`，而 `snapshot` 返回拥有数据的 `Snapshot`？前者必须保持运行时借用守卫；不能先取得临时 `Ref`，返回其内部 `&Garden`，然后丢掉守卫。后者花一次复制成本换取独立生命周期。

`Snapshot::select` 返回的是当前快照中任务的引用，不是克隆任务。签名省略的生命周期把结果绑定到 `&self`。请把它手写展开一次，再解释为什么不能返回临时快照里的引用。这里不是“到处加 `'static`”。

`render_with<R: Renderer + ?Sized>` 同时接受具体 `BoardView` 和 `&dyn Renderer`。前者可静态分发，后者由 trait 对象动态分发；`?Sized` 使泛型参数允许动态大小的 trait 对象。不要声称“使用泛型就一定比动态分发快”，本实验没有测这个性能差异。

在平台根目录检查：

```powershell
cargo run --locked --offline -p labquest -- advanced check my-workbench 1
```

只实现 A1 即可通过 A1，A2/A3 的未实现函数不应阻挡本阶段。也可进入学员目录执行 `cargo test --offline --test a1`。

### 提示 1

先回答每一条关系是否应该延长数据生命。读视图选择 Weak，不是因为 Weak“更快”，而是关闭文档之后视图必须能够自然失效。当前数据结构本身没有环；额外的 `ownership_lab` 专门演示强引用环为什么会泄漏。

### 提示 2

使用 `try_borrow` 将冲突转换成 `EditError::Busy`。用 `Ref::map` 从 Session 的守卫投影到 Garden；在调用用户提供的 Renderer 之前先形成拥有型快照并结束借用作用域，避免渲染代码重入时持有额外借用。

### 提示 3

用 `ptr::eq` 比较两个句柄借到的 Garden 地址；先丢一个句柄，再丢最后一个；让空任务和中文任务都渲染。查询至少测试“全部匹配、无匹配、只未完成”。不要通过复制结果来掩盖生命周期设计问题。

### 交付与复盘

提交所有权图、一段自己的选择器、至少一条新测试。说明为什么只读 View 不应该保存 `Rc`，以及何时你会不用 `Rc/RefCell` 而回到 ID + 单一控制器。行为通过不能代替这份设计解释。

## A2 用闭包实现整批编辑、版本冲突与撤销

现在产品需求是：“对所有未完成任务编号改名；任意一条标题无效，整个批次不生效；用户可以撤销”。需要实现 `edit`、`rename_where` 和 `undo`。

三个闭包接口分工：`Fn(&Task)->bool` 是可重复查询的选择策略；`FnMut(&Task)->String` 允许编号器修改捕获的计数；`FnOnce(&mut Garden)->Result<T,String>` 允许一次编辑消耗拥有型输入。它们不是三种互不相交的闭包类别：实现 Fn 的闭包也可用于要求 FnMut/FnOnce 的接口。

**关键预测：** `move` 闭包是否必然只能调用一次？不是。`move` 决定捕获方式；闭包体是否移出捕获值决定调用能力。比较“每次读取一个已捕获 String 的长度”和“把该 String 返回给调用者”。不要把“按值捕获”和“调用时消耗”混为一谈。

编辑流程不是在 live Garden 上直接修改：

1. 用短作用域检查可编辑并记录基础版本、原始数据，随后释放所有 RefCell 守卫。
2. 在草稿上调用一次用户闭包，并校验整个草稿。
3. 再次取得可变借用，检查 live 版本是否仍等于基础版本。发生重入编辑时，拒绝覆盖其结果。
4. 只有实际变化才记录撤销并提交；一次批量动作算一条撤销。撤销最多保留32条；撤销恢复数据但版本继续增加。

为什么释放守卫后还要检查版本？虽然是单线程，用户回调仍然可能通过另一个 Workspace 句柄执行编辑。那不是线程数据竞争，但同样可能形成“旧草稿覆盖新结果”。这时返回 Conflict，已发生的内部编辑保留。

借用冲突、非法草稿、闭包返回错误都不得创建半成品或伪造撤销记录。**草稿隔离仅保护本会话的数据，不回滚闭包中的文件写入、日志打印或外部计数器。** 不能将此功能称为数据库事务或断电持久化。

在平台根目录检查：

```powershell
cargo run --locked --offline -p labquest -- advanced check my-workbench 2
```

A1会先重新运行。尝试第一条生成有效标题、第二条生成换行字符，检查两条任务和版本都未改变；再运行正确版本并撤销。

### 提示 1

先让最简单的 `edit(|_| Ok(()))` 成功且不增加版本；再做真实改名。错误发生后同时比较任务内容、版本号和是否能够撤销，不能只看 Result。

### 提示 2

取出快照、运行闭包、提交是三个不同借用作用域。提交时仍须比较版本。`rename_where` 可以复用 `edit`，不要维护第二套不一致的历史处理。FnMut 的参数绑定需要可变。

### 提示 3

用一个没有 Clone 的输入类型验证 FnOnce；保留读取守卫后编辑，应返回 Busy 且回调不执行。让回调经另一个句柄提交一次修改，外层应 Conflict，而内部修改仍在。测试33次以上修改时的历史上限。

### 交付与复盘

添加“只改含有指定关键词的未完成任务”策略，不修改库内部筛选逻辑。写一个捕获计数器的命名器。说明无效批次的外部计数器为什么仍可能变化，为什么撤销版本不能回退。列出草稿复制成本；本课程上限1000条任务，不宣称适用于巨大数据集。

## A3 快照跨线程、Send/Sync 与可替换报告

最后需要实现 `spawn_report`。它接受 `Arc<Snapshot>` 和一个渲染器，返回 `JoinHandle<String>`。两个后台报告共享同一份不可变快照；用户可继续编辑 live Workspace。报告必须带捕获的版本号，不得伪装为实时结果。

`Rc` 的引用计数不是跨线程设计；`RefCell` 的借用状态也不是线程同步机制。将 Rc 换成 Arc 不会自动使 `RefCell<T>` 成为可共享的同步对象。本实验明确不把 live Workspace 送进线程；跨过去的是拥有数据的不可变 Snapshot。

为什么 renderer 约束是 `Renderer + Send + 'static`，不是一律增加 Sync？渲染器只被移动给一个工作线程，不同时被多个线程借用。快照则被多个线程经 Arc 共享，因此内部数据需要满足相应的 Send/Sync 条件。契约测试专门提供一个 Send 但不是 Sync 的渲染器，避免无理由加上更强约束。

`'static` 在这里表示交给可能独立运行的线程的数据不含短命借用，不等于这个对象必须一直存活到程序退出。线程退出后拥有型 String、Vec 等仍然正常销毁。

先运行平台中的编译失败对照，再完成学员函数：

```powershell
cargo test --locked --offline -p termgarden --doc
cargo run --locked --offline -p termgarden --example ownership_lab
cargo run --locked --offline -p labquest -- advanced check my-workbench 3
```

文档中的五组 `compile_fail` 必须编译失败，包括借用越界、Rc跨线程、Fn误用、FnOnce重复调用、Arc<RefCell>跨线程。它们显示为测试通过，表示编译器按预期拒绝错误代码。两组正例必须正常运行。不能拿只有编译失败实验通过来宣布业务功能完成。

### 提示 1

先解释发送的是哪份数据，再使用 thread::spawn。传递完整快照可以减少共享可变状态的需求，Mutex不是所有后台任务的默认答案。

### 提示 2

将快照和渲染器 move 到线程闭包；复用 render_with；由调用者 join 和处理线程 panic。不要复制句柄后又偷偷从live工作区读数据。

### 提示 3

用 Barrier，而非“睡几毫秒”，固定“报告线程等待、主线程改名、解除等待”的顺序。报告必须仍包含旧标题和旧版本；新快照包含新标题。这是确定性功能测试，不是速度基准。

### 交付与复盘

在**学员工程目录**运行：

```powershell
cargo test --offline
cargo run --offline -- demo my-result
cargo run --offline -- play my-tasks.tasks
```

play模式中依次输入：

```text
add 整理自己的课程笔记
add 给工作区增加新测试
prefix 本周：
undo
report my-reports
save
q
```

`report` 将快照提交给两个工作线程后立即返回，继续编辑不会改变该报告。`reports`收取已完成报告，`wait`等待所有报告并写文件；退出或输入结束也会收取，最多保留4份待收取作业。文件写入在收取时完成，不是持续运行的服务。

再次 play 同一个任务文件确认持久化。报告目录必须全新，已有目录会拒绝覆盖。报告导出不是原子目录事务，写入异常可能留下不完整的新目录；只能用自己的实验数据。此版无数据库锁，只支持单写者；撤销历史不持久化、不提供redo，最多32条；会话重启后版本从0开始，不能跨会话当作全局身份。

说明具体类型渲染器与 `Box<dyn Renderer>` 的差别，解释为何一个Cell计数器渲染器可以被移动到线程而不要求Sync。若希望多个线程共同修改同一模型，需重新设计锁与不变量，不能只改类型名。

## 阶段完成与新需求考核

三阶段均通过之后，选择自己的任务数据，编写至少一条**平台没有提供的新测试**，实现“给指定关键词的任务加前缀，保留已完成任务”的策略，输出两个不同视图。填写 DESIGN_REVIEW.md，解释一个你拒绝采用的架构方案。

平台累计检查在新临时工程中复制 `src/**/*.rs`，使用固定的依赖、旧基础设施和合同测试；不运行学员自行修改的 Cargo.toml、tests、support或build.rs。请另行运行自己的 cargo test，保留两份结果。源码变化后历史报告失效；报告可编辑，仅用于本地导航，不是防作弊认证。

这是一套可信代码的教学执行器，不是恶意代码沙箱。限时与输出限额只用于常见误操作，不能保证杀死所有后代进程，也不保证资源隔离。不要评测陌生源码。

已完成基础课程的学员从本页开始，不需要重复旧关卡。进阶 A 提供可运行的功能扩展和机制实验；B“第三方生态与库工程”、C“成熟异步运行时与网络应用”尚未交付。本轮没有把标准库线程当成Tokio，也没有宣称整条进阶主线已完成。

## 官方资料（用于核对机制，不是要求逐页通读）

- [Rc 与 Weak、引用环](https://doc.rust-lang.org/book/ch15-06-reference-cycles.html)：弱引用不拥有数据，upgrade处理关闭状态。
- [RefCell 与内部可变性](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html)：运行时借用仍须遵守规则。
- [闭包类型与调用约束](https://doc.rust-lang.org/reference/types/closure.html)：捕获方式与Fn/FnMut/FnOnce不是同一维度。
- [Send 与 Sync](https://doc.rust-lang.org/book/ch16-04-extensible-concurrency-sync-and-send.html)：线程转移和共享引用的不同保证。
- [thread::spawn](https://doc.rust-lang.org/std/thread/fn.spawn.html)：线程入口、Send与生命周期约束。
- [trait 对象](https://doc.rust-lang.org/book/ch18-02-trait-objects.html)：运行时多态与静态分发的取舍。

**验证范围：**开发验证只记录实际执行的平台；本版不继承旧RC或MSRV资格。面向Windows x64 MSVC、Linux x86_64，不声明macOS。普通学员不需要Python或Playwright。
