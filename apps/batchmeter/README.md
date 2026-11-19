# BatchMeter · 统计多份文件，保持结果有序

**这次要做成什么：**读取多份 UTF-8 文本，显示逐文件统计与总数；改用多个线程后，输出仍与输入路径顺序一致。

**开始条件：**完成 [TermGarden](../termgarden/README.md)，能区分只读借用、可变状态与错误返回，并记得 TextScope 的字节和行数口径。

本目录是已实现的参考软件；你的练习代码放在下面新建的 `my-batchmeter/` 中。

## 1. 看目标，创建自己的工程

在 **RustAppLab 根目录**（能看到根 `Cargo.toml`）运行：

```powershell
cargo run --offline -p batchmeter -- scan 2 apps/batchmeter/samples/one.txt apps/batchmeter/samples/two.txt
cargo run --offline -p labquest -- studio start batchmeter my-batchmeter
cargo run --offline -p labquest -- learn lesson my-batchmeter 1
cargo run --offline -p labquest -- learn check my-batchmeter 1
```

参考样例合计应为 `files=2 bytes=36 lines=5`。

学员首次检查因 TODO 未实现而 FAIL 是正常的。先看第一条失败；若是编译器或链接器找不到，则应先修好环境。

打开 `my-batchmeter/START.html` 或 `TASK_CARDS.html`；只编辑 **`my-batchmeter/src/lib.rs`**，每次一个 TODO。命令行外壳 `src/main.rs` 已提供，先不改。目录已存在就继续使用；新建另一份练习时换名称。

## 2. 三个检查点

| 步骤与函数 | 要完成的行为 | 本次概念 |
|---|---|---|
| TODO(1) · `analyze` | 有界读取一个文件，校验 UTF-8，返回路径、字节数和行数 | `Path`、I/O、`Result`，错误与空文件的区别 |
| TODO(2) · `aggregate` | 只借用统计结果求和，空批次为零，溢出报错 | 切片、结构体、`checked_add` |
| TODO(3) · `analyze_many` | 有限线程和通道处理路径，按输入索引还原顺序 | `thread::scope`、`move`、`mpsc`、`join` |

第 3 步可先写串行版理解流程，再完成线程改造。**串行实现也可能满足行为测试，是否真正并行还要阅读代码和解释。**

每步先写一组输入和预测，再修改、保存。第一步通过后，在 **平台根目录**继续：

```powershell
cargo run --offline -p labquest -- learn lesson my-batchmeter 2
cargo run --offline -p labquest -- learn check my-batchmeter 2
cargo run --offline -p labquest -- learn lesson my-batchmeter 3
cargo run --offline -p labquest -- learn check my-batchmeter 3
```

后一步会累计回归前面的步骤。提示用 `cargo run --offline -p labquest -- learn hint my-batchmeter 1`；进度用 `cargo run --offline -p labquest -- learn next my-batchmeter`。详细解释见 [batchmeter 讲义](../../course/studio/batchmeter.md)。

## 3. 用自己的输入完成一次作品

三步通过后，在学员 `samples/` 保存两个 UTF-8 文件：`mine-a.txt` 是一行 `Rust`，`mine-b.txt` 是一行 `学习`，各有末尾换行。

从 **平台根目录进入学员目录**执行：

```powershell
cd my-batchmeter
cargo run --offline -- scan 1 samples/mine-a.txt samples/mine-b.txt
cargo run --offline -- scan 4 samples/mine-a.txt samples/mine-b.txt
cargo run --offline -- scan 2 samples/mine-a.txt samples/not-created.txt
cargo run --offline -- scan 0 samples/mine-a.txt
```

前两次输出应完全一致：先 a、后 b，合计 2 文件、2 行；无 BOM 且使用 LF 时合计 12 字节。后两条应失败。第二个文件先算完，也不能改变面向用户的行序。

## 4. 交付与解释

在 **学员目录**运行 `cargo test --offline` 回归全部测试。保存自己的文件、串行与并行结果及实现说明。把预测、错误和解释写入 `LEARNING_LOG.md`，再补一个自己的边界测试。

用自己的话回答：线程拿到了哪些数据？为什么消息带输入索引？为什么所有发送端都释放后接收循环才能结束？

测试通过只说明已检查的行为符合合同；个人输入、真实产物和解释能帮助你判断学到了什么。

线程数限 1–16，单文件最多 8 MiB，一次最多 8192 个路径，仅接受 UTF-8。线程更多不保证更快；没有 async 网络能力。

## 下一款：MiniKV

接下来把线程、trait 和借用用于可持久化的键值库，比较消息传递与共享状态。在 **RustAppLab 根目录**运行（若仍在学员目录，先 `cd ..`）：

```powershell
cargo run --offline -p labquest -- studio start minikv my-minikv
cargo run --offline -p labquest -- learn lesson my-minikv 1
```

继续阅读 [MiniKV README](../minikv/README.md)。
