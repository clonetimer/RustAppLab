# BatchMeter · 多文件并行统计台

**本作品的中心思想**：用一款可运行的小软件，把三个连续能力连接起来，而不是完成三道无关语法题。

## 开始前只确认这件事

你当前应位于这个学员工程的目录，能够看到 `Cargo.toml`。终端中的 `cargo` 命令默认在这里运行。`src/main.rs` 是已经接好的命令行外壳；本轮只修改 `src/lib.rs` 的 TODO(1)、TODO(2)、TODO(3)。`tests/milestones.rs` 是公开的验收合同，不能通过删除测试“通关”。

在完整 RustAppLab 根目录创建本工程的命令是 `cargo run --offline -p labquest -- studio start batchmeter my-batchmeter`，然后进入 `my-batchmeter`。本讲义也随学员包提供，包内已有工程就不必再次生成。Windows 可以用 `CONTINUE_WINDOWS.cmd` 选择作品。

三个里程碑依次是：**读取真实文件并明确失败 → 用结构体累计批处理结果 → 用作用域线程与通道保持结果有序**。未实现的后两步不会阻止你运行第一步测试；语法错误仍会阻止整个库编译。

## 先看你要做出的软件

让程序一次读取多个文本文件，输出每份文件的字节数和行数。你先实现串行，再改成有限数量的工作线程。目标是正确划分任务并收集结果，不是“线程越多一定越快”。小文件和缓存环境里，并行甚至可能更慢。

前置要求：理解 TextScope 的借用、LedgerLite 的错误传播。不要把本作品安排在第一张任务卡之后。

```powershell
cargo run --offline -- scan 2 samples/one.txt samples/two.txt
```

默认样例合计应为 2 文件、36 字节、5 行。输出列表顺序和你传入路径的顺序一致，不随线程完成先后改变。每文件只接受 UTF-8 文本，最大 8 MiB；一次最多 8192 个路径，线程数 1..16。

## 第 1 步：先做一件完整的小任务

打开 `src/lib.rs`，实现 `analyze(path: &Path)`。读取文件，统计实际字节数，用 UTF-8 校验内容，再计算行数。`FileStats` 保存路径副本、字节数、行数。文件不存在和编码不合法应返回错误，不是一个全零的成功结果。

路径参数借用 Path，返回结果需要独立保存它，所以用 `to_path_buf()`。使用 `Read::take(8*1024*1024+1)` 读取有界数量的数据；多读一个字节用于判断是否超限。不要只先看 metadata 长度然后不加限制地读，文件可能变化。

```powershell
cargo test --offline --test milestones step1_
```

测试在自己的临时目录建立文件，不会碰你的真实文稿。其中包含中文与空格路径。

## 第 2 步：把结果收集为另一个值

TODO(2) 的 `aggregate` 读取 FileStats 切片，构造 Totals。使用 checked_add 处理字节和行数溢出，空切片得到零文件、零字节、零行。聚合函数没有文件 I/O，可以单独验证。

```powershell
cargo test --offline --test milestones step2_
```

**自己解释**：为什么先生成每份文件的 FileStats，再做聚合，而不是让每个线程直接修改同一个总数？前者减少共享可变状态，便于把读取失败和汇总失败分开定位。

## 第 3 步：从串行到线程，不跳过中间过程

先让 TODO(3) 用 `paths.iter().map(...)` 串行调用 analyze，确认正常与错误路径都会返回明确结果。这一版可以通过行为测试，但还没有实现并行；必须继续做下面的改造，并在学习记录中说明。

将输入切成不超过 workers 个分片。`std::thread::scope` 允许这些线程借用外层 paths，因为作用域返回前会等待线程完成。每个线程处理自己的只读分片，通过 `mpsc::channel()` 发送 `(原始位置, Result<FileStats, String>)`。

发送端可以 clone，多工作线程拥有各自的发送端。闭包使用 `move`，转移发送端句柄而非复制所有文件内容。保存 join 句柄并处理线程 panic。作用域结束后，显式 drop 最初的 tx，确认所有发送端已经销毁，再遍历 rx 收集结果；漏掉这个 drop 会让接收端一直等“也许还有下一条”。

收集的消息按原始位置排序，随后把 Vec<Result<...>> 合成 Result<Vec<...>>。一份文件失败时整个命令失败，不能把坏文件悄悄漏掉然后宣布处理成功。

```powershell
cargo test --offline --test milestones step3_
cargo run --offline -- scan 1 samples/one.txt samples/two.txt
cargo run --offline -- scan 4 samples/one.txt samples/two.txt
```

两次结果应完全一致。测试验证结果与边界，但不能仅凭结果判断你是否真正用了多个线程；维护者/同伴还要查看 scope、spawn 和 channel 的实现。自动验收不是学习能力认证。

## 可观察的故障与交付

给参数列表加入不存在的文件，确认命令报错；把 workers 改为 0 或 17，确认没有开始无意义的工作；在副本里移除 drop(tx)，观察卡住，再恢复。使用 Rust AppLab 的 studio check 时，每个测试阶段有超时保护；自行 cargo test 的实验若卡住，用 Ctrl+C 终止。

交付一批自己的小文本、串行/并行结果对比和结构示意说明。测速时记录文件数、大小、运行次数与缓存影响，不能凭一次快了就宣称普遍提速。后续 async 网络客户端是另一种并发模型，本作品不等于已经教完 async/await。

参考：作用域线程 https://doc.rust-lang.org/std/thread/fn.scope.html ，消息传递 https://doc.rust-lang.org/book/ch16-02-message-passing.html 。

## 一次只检查当前阶段

本工程内可以运行 `cargo test --offline --test milestones step1_`。完成一步，再运行 step2_、step3_，最后执行全量 `cargo test --offline`。完整平台还提供权威检查：回到 RustAppLab 根目录执行 `cargo run --offline -p labquest -- studio check "my-batchmeter" 1`，把最后的 1 改成 2、3 或 all。它会用自带的测试检查当前源码，不信任你编辑后的本地 tests。第 2 步会重新检查第 1 步，第 3 步会重查前两步。

检查会真实编译并运行你写的 Rust，只用于你信任的自编代码，不是安全沙箱。报告按检查次数生成新文件，包含当前源文件摘要；旧报告不能证明修改后的代码仍正确。遇到错误先看第一个 FAIL 和对应函数，再看 HINTS.md。

## 完成标准

功能测试通过只是其中一项。还需用自己的数据运行一次，给一个错误输入，保存真实产物或控制台结果，并在 LEARNING_LOG.md 解释一个关键 Rust 概念。然后为未提供的变体自行增加一个测试。不要把参考程序已通过等同于自己已学会。
