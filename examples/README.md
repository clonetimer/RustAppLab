# 练习数据与输出样例

本目录帮助你对照“输入应该产生什么结果”。这里的内容是**可选样例和参考程序产物**，不是学员 Rust 答案，也不会自动记为你的作品或学习成绩。

刚完成 FirstCard 时，请先打开 [后续学习指南](../course/AFTER_FIRSTCARD.md)，创建 UnitLab；不用按子目录顺序把所有样例跑一遍。

## 按当前作品选择

| 内容 | 用途 | 什么时候用 |
|---|---|---|
| [`foundation/`](foundation/README.md) | QuizDeck 的题库 / 答案，TextScope 的笔记 | 相应作品的三个检查点完成后，用文件验证 |
| [`badgeforge-demo.svg`](badgeforge-demo.svg) | 参考程序生成的 SVG 卡片 | 做 BadgeForge 时观察最终产物 |
| [`systems/`](systems/README.md) | 个人工单档案的学习记录模板 | BatchMeter → MiniKV → NetFetch → ServiceDesk |
| [`reliability/`](reliability/README.md) | StoreDesk 事务与 AsyncBoard 等待清单 | 已理解文件持久化、线程与错误处理时 |
| [`minikv/`](minikv/README.md) | 既有数据库、导出和故意损坏的反例 | 对照 MiniKV 的读取、迭代与错误行为 |

样例中的 TSV 使用真正的 Tab 和 UTF-8。做个人实验时复制到新的目录，保留旧作品；不要把示例输出直接当成自己的实测记录。

## 先看一件能打开的作品

直接用浏览器打开 `badgeforge-demo.svg`。它是 BadgeForge 的输出样例，具体文字来自当时的生成输入。

在 **RustAppLab 源码根目录**（含 `apps/` 的目录）生成自己的参考体验，输出文件名必须尚不存在：

```powershell
cargo run --locked --offline -p badgeforge -- make "我的 Rust 作品" "从第一行代码到自己的软件" my-badge.svg
```

应出现 `my-badge.svg`，用浏览器打开能看到标题和副标题。再次使用同一输出路径应拒绝覆盖；这也是需要观察的产品行为。

以上运行的是完整参考软件。要实现自己的 BadgeForge，在 **平台根目录**创建新工程：

```powershell
cargo run --locked --offline -p labquest -- studio start badgeforge my-badgeforge
cargo run --locked --offline -p labquest -- learn lesson my-badgeforge 1
cargo run --locked --offline -p labquest -- learn check my-badgeforge 1
```

初次 TODO 检查失败是正常起点。按 [BadgeForge 讲义](../course/studio/badgeforge.md) 完成 `escape_xml`、trait 实现和泛型文档生成，再在 **自己的工程目录**运行：

```powershell
cargo run --offline -- make "我的 Rust & CLI" "自己的作品说明" my-first-badge.svg
```

## 怎么把样例变成个人学习证据

先记录输入、预测输出，再运行自己的程序。对 SVG 可把标题改为 `A&B <Rust>`，检查特殊字符仍作为文字显示；对题库改一道题，对数据库增加一个自己的键。

至少保留一次正常结果、一次失败结果和原因；说明改了哪个函数、为什么预期会改变。自己完成了输入变化、实现和解释，才有比“参考文件能打开”更多的学习证据。

接下来跟随对应子目录的讲义回到当前作品。[基础路线](../course/FOUNDATION_PATH.md)、[系统应用路线](../course/SYSTEMS_PATH.md)、[事务与等待路线](../course/RELIABILITY_PATH.md) 提供各自的前置要求与下一阶段，不需要安装额外工具来浏览这些样例。
