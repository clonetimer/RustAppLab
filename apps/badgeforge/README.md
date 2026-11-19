# BadgeForge · 给自己的 Rust 作品做一张 SVG 卡

**这次要做成什么：**输入标题与副标题，生成能在浏览器离线打开的 SVG。亲手实现文字转义、图形元素与完整文档三层，保存自己的作品卡。

**开始条件：**完成 [LedgerLite](../ledgerlite/README.md)，能说明字符串、结构体和借用。

本目录是已实现的参考软件；你的练习代码放在下面新建的 `my-badgeforge/` 中。

## 1. 看目标，创建自己的工程

在 **RustAppLab 根目录**（能看到根 `Cargo.toml`）运行：

```powershell
cargo run --offline -p badgeforge -- make "参考作品" "观察输入如何显示" reference-badge.svg
cargo run --offline -p labquest -- studio start badgeforge my-badgeforge
cargo run --offline -p labquest -- learn lesson my-badgeforge 1
cargo run --offline -p labquest -- learn check my-badgeforge 1
```

应显示 `created=reference-badge.svg`；用浏览器打开，确认两段文字出现。输出文件名必须是新的。

学员首次检查因 TODO 未实现而 FAIL 是正常的。先看第一条失败；若是编译器或链接器找不到，则应先修好环境。

打开 `my-badgeforge/START.html` 或 `TASK_CARDS.html`；只编辑 **`my-badgeforge/src/lib.rs`**，每次一个 TODO。命令行外壳 `src/main.rs` 已提供，先不改。目录已存在就继续使用；新建另一份练习时换名称。

## 2. 三个检查点

| 步骤与函数 | 要完成的行为 | 本次概念 |
|---|---|---|
| TODO(1) · `escape_xml` | `& < > " '` 转成实体，中文保持可读 | 用户文字与 XML 标签的边界、替换顺序 |
| TODO(2) · `Badge::render` | 实现 `svg::Element`，生成包含两段安全文字的图形片段 | `trait`、`impl`、`&self` |
| TODO(3) · `document` | 把任意实现 Element 的值组合成完整 SVG 文档 | 泛型约束、方法调用、组合 |

每步先写一组输入和预测，再修改、保存。第一步通过后，在 **平台根目录**继续：

```powershell
cargo run --offline -p labquest -- learn lesson my-badgeforge 2
cargo run --offline -p labquest -- learn check my-badgeforge 2
cargo run --offline -p labquest -- learn lesson my-badgeforge 3
cargo run --offline -p labquest -- learn check my-badgeforge 3
```

后一步会累计回归前面的步骤。提示用 `cargo run --offline -p labquest -- learn hint my-badgeforge 1`；进度用 `cargo run --offline -p labquest -- learn next my-badgeforge`。详细解释见 [badgeforge 讲义](../../course/studio/badgeforge.md)。

## 3. 用自己的输入完成一次作品

三步通过后，先预测副标题中的特殊字符会怎样保存在 SVG 里。

从 **平台根目录进入学员目录**执行：

```powershell
cd my-badgeforge
cargo run --offline -- make "我的 Rust 学习" "trait & <泛型>" my-badge.svg
cargo run --offline -- make "第二次写入" "观察拒绝覆盖" my-badge.svg
```

第一条应显示 `created=my-badge.svg`。浏览器中应看见原文字，源码中 `&` 和尖括号应成为实体。第二条必须报错，原文件不变。再用自己的作品名生成另一张新卡片。

## 4. 交付与解释

在 **学员目录**运行 `cargo test --offline` 回归全部测试。保存个人 SVG、源码和拒绝覆盖的实测结果；可以增加一个自定义 Element 的测试。把预测、错误和解释写入 `LEARNING_LOG.md`，再补一个自己的边界测试。

用自己的话回答：为什么 `<泛型>` 是文字而不能直接拼成标签？为什么 `document` 不必知道具体类型是不是 Badge？为什么这里借用 `&self` 就够了？

测试通过只说明已检查的行为符合合同；个人输入、真实产物和解释能帮助你判断学到了什么。

标题为 1–20 个字符，副标题最多 40 个字符，均不能含控制字符；没有自动换行、复杂排版或完整字体适配。测试不要求固定颜色和排版。

## 下一款：TermGarden

接下来让数据随着用户操作变化，学习事件、可变借用和状态绘制。在 **RustAppLab 根目录**运行（若仍在学员目录，先 `cd ..`）：

```powershell
cargo run --offline -p labquest -- studio start termgarden my-termgarden
cargo run --offline -p labquest -- learn lesson my-termgarden 1
```

继续阅读 [TermGarden README](../termgarden/README.md)。
