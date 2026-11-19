# UnitLab · 把第一行输出变成单位换算器

**这次要做成什么：**输入温度或长度，得到随输入变化的换算结果；输入文字或不兼容的单位时，得到明确错误。你将从 FirstCard 的 `println!` 继续学习“函数把结果返回给调用者”。

**开始条件：**完成 FirstCard，能找到自己的源码、保存文件、重新运行。还没完成时，先看 [FirstCard 之后怎么学](../../course/AFTER_FIRSTCARD.md)。

本目录是已实现的参考软件；你的练习代码放在下面新建的 `my-unitlab/` 中。

## 1. 看目标，创建自己的工程

在 **RustAppLab 根目录**（能看到根 `Cargo.toml`）运行：

```powershell
cargo run --offline -p unitlab -- convert 25 c f
cargo run --offline -p labquest -- studio start unitlab my-unitlab
cargo run --offline -p labquest -- learn lesson my-unitlab 1
cargo run --offline -p labquest -- learn check my-unitlab 1
```

第一条应输出 `25.00 c = 77.00 f`。

学员首次检查因 TODO 未实现而 FAIL 是正常的。先看第一条失败；若是编译器或链接器找不到，则应先修好环境。

打开 `my-unitlab/START.html` 或 `TASK_CARDS.html`；只编辑 **`my-unitlab/src/lib.rs`**，每次一个 TODO。命令行外壳 `src/main.rs` 已提供，先不改。目录已存在就继续使用；新建另一份练习时换名称。

## 2. 三个检查点

| 步骤与函数 | 要完成的行为 | 本次概念 |
|---|---|---|
| TODO(1) · `celsius_to_fahrenheit` | 0、100、-40 等输入得到相应华氏温度 | 参数、`f64`、返回表达式；打印与返回的区别 |
| TODO(2) · `convert` | 温度、长度正反换算；拒绝跨类别换算及非有限数 | `enum`、`match`、`Result`；错误不是 0 |
| TODO(3) · `describe` | 解析文字，调用换算，返回两位小数的说明 | `parse`、`?`、`format!`，组合三个函数 |

每步先写一组输入和预测，再修改、保存。第一步通过后，在 **平台根目录**继续：

```powershell
cargo run --offline -p labquest -- learn lesson my-unitlab 2
cargo run --offline -p labquest -- learn check my-unitlab 2
cargo run --offline -p labquest -- learn lesson my-unitlab 3
cargo run --offline -p labquest -- learn check my-unitlab 3
```

后一步会累计回归前面的步骤。提示用 `cargo run --offline -p labquest -- learn hint my-unitlab 1`；进度用 `cargo run --offline -p labquest -- learn next my-unitlab`。详细解释见 [unitlab 讲义](../../course/studio/unitlab.md)。

## 3. 用自己的输入完成一次作品

三步通过后，先预测下面两组合法换算，再观察两种错误。

从 **平台根目录进入学员目录**执行：

```powershell
cd my-unitlab
cargo run --offline -- convert -10 c f
cargo run --offline -- convert 2500 m km
cargo run --offline -- convert abc c f
cargo run --offline -- convert 1 m f
```

前两条应输出 `-10.00 c = 14.00 f` 和 `2500.00 m = 2.50 km`；后两条应报错。再挑一个你所在地的气温，先手算、再核对，不能只记住演示里的数字。

## 4. 交付与解释

在 **学员目录**运行 `cargo test --offline` 回归全部测试。保存自己的换算输入、预期和实际输出。把预测、错误和解释写入 `LEARNING_LOG.md`，再补一个自己的边界测试。

用自己的话回答：`describe` 的数字最初为什么是 `&str`？为什么函数最后一行加分号可能让返回类型变成 `()`？解析失败后，`?` 为什么让换算停止？

测试通过只说明已检查的行为符合合同；个人输入、真实产物和解释能帮助你判断学到了什么。

当前支持 `c/f/m/km`，没有重量、汇率或单位自动识别。新增厘米是后续扩展，先定义输入输出再实现。

## 下一款：QuizDeck

接下来把单个数值换成多条题目，学习 `String`、`Vec` 和结构体。在 **RustAppLab 根目录**运行（若仍在学员目录，先 `cd ..`）：

```powershell
cargo run --offline -p labquest -- studio start quizdeck my-quizdeck
cargo run --offline -p labquest -- learn lesson my-quizdeck 1
```

继续阅读 [QuizDeck README](../quizdeck/README.md)。
