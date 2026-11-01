# BadgeForge · SVG 作品卡生成器

**本作品的中心思想**：用一款可运行的小软件，把三个连续能力连接起来，而不是完成三道无关语法题。

## 开始前只确认这件事

你当前应位于这个学员工程的目录，能够看到 `Cargo.toml`。终端中的 `cargo` 命令默认在这里运行。`src/main.rs` 是已经接好的命令行外壳；本轮只修改 `src/lib.rs` 的 TODO(1)、TODO(2)、TODO(3)。`tests/milestones.rs` 是公开的验收合同，不能通过删除测试“通关”。

在完整 RustAppLab 根目录创建本工程的命令是 `cargo run --offline -p labquest -- studio start badgeforge my-badgeforge`，然后进入 `my-badgeforge`。本讲义也随学员包提供，包内已有工程就不必再次生成。Windows 可以用 `CONTINUE_WINDOWS.cmd` 选择作品。

三个里程碑依次是：**正确转义文本而不破坏中文 → 为作品卡实现 Element trait → 用泛型组合独立 SVG 文档**。未实现的后两步不会阻止你运行第一步测试；语法错误仍会阻止整个库编译。

## 先看你要做出的软件

这次作品不只在终端显示数字。你将生成一张可以在浏览器打开、放进作品集的 SVG 卡片。完成后在本工程目录运行：

```powershell
cargo run --offline -- make "我的 Rust 作品" "编写 · 运行 · 分享" my-badge.svg
```

双击 `my-badge.svg` 用浏览器查看。已有同名文件会被拒绝，不会覆盖之前的作品；换一个新文件名继续实验。标题最多 20 个 Unicode scalar，副标题最多 40 个，布局不是自动排版引擎。

## 第 1 步：把用户输入当文本而不是标签

打开 `src/lib.rs` 的 `escape_xml`。输入里可能有 `& < > " '`。它们应分别转为 `&amp; &lt; &gt; &quot; &apos;`，中文原样保留。先替换 &，否则你刚生成的 `&lt;` 会再次被转成 `&amp;lt;`。

```powershell
cargo test --offline --test milestones step1_
```

文本 `<script>` 要显示为字面文字，不应该变成标签。已有文本 `&lt;` 本来就是用户输入的五个字符，应转成 `&amp;lt;`。不要把“多次调用仍不变”的幂等性强加给这个原始文本转义函数。

## 第 2 步：第一次实现 trait

`svg` 模块中有 `pub trait Element { fn render(&self) -> String; }`。它表达“这个类型能够输出一段 SVG 内容”。`Badge` 保存标题和副标题，`impl svg::Element for Badge` 则说明 Badge 如何做到这件事。

TODO(2) 在 `render` 中生成 `<g>...</g>`：包含背景 `<rect>` 和两段 `<text>`。标题与副标题必须先经过上一步转义。图形的颜色和排版可以自己选择，测试不要求一模一样；需要有文本元素、数据随输入变化且不产生未经转义的标签。

可以先只生成两段文本，再补背景。`self.title` 是当前卡片的数据，`&self` 表示渲染只读借用，不消耗卡片。为了少写双引号转义，可以使用 `r#"..."#` 原始字符串；如果字符串内部存在 `"#`（例如颜色值），就使用 `r##"..."##`。

```powershell
cargo test --offline --test milestones step2_
```

调用 trait 方法时，要让 trait 在当前模块可见，例如 `use badgeforge::svg::Element;`。遇到“method not found”先检查 trait 导入，不要立刻把方法复制成另一份。

## 第 3 步：为什么这里需要泛型

TODO(3) 的签名是 `document<T: svg::Element>(element: &T) -> String`。它不关心 T 是否为 Badge，只要求它具有 render 行为。组合完整 XML 声明、SVG 根标签、命名空间、viewBox、element.render() 和闭合标签，就能把任意符合约定的图形元素包成独立文件。

```powershell
cargo test --offline --test milestones step3_
```

测试会另外实现一个 Dot 圆点类型传给 document，避免你只对 Badge 特判。完成后自己新增一张矩形卡片类型，复用 document，看看为什么不用改生成文档的函数。

## 从代码到可分享作品

全部测试通过后运行 make 命令；再使用标题 `A&B <Rust>` 生成第二张卡片。浏览器应显示这些文字，不把它们解释成标签。用文本编辑器打开 SVG，检查文件中的实体和两个文本字段。

保存源代码、SVG 和一段关于 trait 的说明。SVG 可以离线打开，不需要运行本地服务器。CLI 对输入长度做基础限制；复杂字体、换行、可访问性描述和智能布局属于后续改进，不能把本作品当作完整图形设计软件。

**自主扩展**：为卡片增加第三行日期，但保持原有泛型接口；然后写一个自己的 Element 实现。看到不同类型都能通过同一 document 函数，才是真正使用了 trait，而不只是记住语法。

参考：Rust 官方 trait https://doc.rust-lang.org/book/ch10-02-traits.html 和泛型 https://doc.rust-lang.org/book/ch10-01-syntax.html 。

## 一次只检查当前阶段

本工程内可以运行 `cargo test --offline --test milestones step1_`。完成一步，再运行 step2_、step3_，最后执行全量 `cargo test --offline`。完整平台还提供权威检查：回到 RustAppLab 根目录执行 `cargo run --offline -p labquest -- studio check "my-badgeforge" 1`，把最后的 1 改成 2、3 或 all。它会用自带的测试检查当前源码，不信任你编辑后的本地 tests。第 2 步会重新检查第 1 步，第 3 步会重查前两步。

检查会真实编译并运行你写的 Rust，只用于你信任的自编代码，不是安全沙箱。报告按检查次数生成新文件，包含当前源文件摘要；旧报告不能证明修改后的代码仍正确。遇到错误先看第一个 FAIL 和对应函数，再看 HINTS.md。

## 完成标准

功能测试通过只是其中一项。还需用自己的数据运行一次，给一个错误输入，保存真实产物或控制台结果，并在 LEARNING_LOG.md 解释一个关键 Rust 概念。然后为未提供的变体自行增加一个测试。不要把参考程序已通过等同于自己已学会。
