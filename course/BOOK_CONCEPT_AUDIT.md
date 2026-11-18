# 原书概念抽样审计 · rc.2 → rc.3

**本次118个抽样概念群中，有教学链的目标从38项增至87项，新增49项；仍有31项保留为部分教学、代码关联或未定位教学。**

基线为 `rc.2`，提交 `ed255fe31ec1be2faba04ffb720660d13938e6a6`。本审计在 rc.3 提交前完成，逐项判定依据为当时工作树；最终提交号与CI结果由该版本的发布记录/同提交CI另行提供。审计日期为2026-11-18。

范围仅为原书第4–6章、8–10章的18节。118是本次定义的抽样概念群数量，跨章目标可能涉及同一机制；它不是整本书知识点的全量互斥划分。**31项残余不是全书欠账总量，这些统计也不是学习者掌握率。**

全部118个稳定ID、前后状态和逐项证据见[机器可读审计明细](../verification/concept-audit-rc3.json)。

## 1. 方法与证据边界

本轮先按指定旧版中文原书的小节拆分目标，再核对已有讲义、参考实现、学员骨架、行为合同和失败/修复对照。实施后沿用同一批ID和标题逐项复核，最后合并两组记录。跨章节已有合格材料可以复用；只出现API、关键词或一条测试调用，不足以将目标提升为有教学链。

| 状态 | 本文含义 |
| --- | --- |
| `teaching` · 有教学链 | 关键区分有讲解，有可追踪的参考行为或失败/修复对照，并安排预测、修改或解释任务。 |
| `partial` · 部分教学 | 已有直接材料，但目标中的一个或多个关键子点仍待补充。 |
| `related` · 代码或关联 | 有实现、测试使用或邻近课程，尚未形成围绕该目标的明确教学链。 |
| `gap` · 未定位教学 | 在本次已审范围内，未定位明确教学实例或合理知识索引。 |

合并时将所有权审计的 `present` 与另一组的 `teaching` 统一为 `teaching`，将 `missing` 统一为 `gap`。原证据代号仍可追踪，并已展开成真实仓库路径和定位说明。目标含多个子点时，未完成的子点继续写入 `remaining_gap`，状态保留为 `partial`。

手工复制的代码练习、自动文档测试和业务行为合同是不同证据。课程提供可运行材料，不等于学习者已完成预测、个人修改、解释或试用。本审计没有重新运行Rust测试，没有把旧版CI或实现报告当作当前版本的发布验收。

JSON中的路径相对完整源码包根目录。基线行号指向rc.2提交；当前材料优先按具体函数、诊断块和章节名查找。下文及JSON用代码格式记录源码路径，不把完整源码包独有的 `apps/`、`projects/` 文件做成学员讲义中的Markdown跳转链接。

## 2. 六章统计

四项状态的排列顺序均为 **teaching / partial / related / gap**。下表只比较同一批118个目标，不将这些数量与全书目录条目相除。

| 章节 | 抽样概念群 | 基线四项状态 | 当前四项状态 | 新增有教学链 |
| --- | ---: | --- | --- | ---: |
| 第4章 · 所有权、引用与切片 | 22 | 8 / 8 / 0 / 6 | 17 / 5 / 0 / 0 | 9 |
| 第5章 · 结构体与方法 | 16 | 6 / 7 / 0 / 3 | 11 / 5 / 0 / 0 | 5 |
| 第6章 · 枚举与模式匹配 | 14 | 9 / 2 / 0 / 3 | 12 / 2 / 0 / 0 | 3 |
| 第8章 · 常见集合 | 23 | 5 / 12 / 2 / 4 | 16 / 4 / 1 / 2 | 11 |
| 第9章 · 错误处理 | 14 | 3 / 4 / 3 / 4 | 7 / 4 / 1 / 2 | 4 |
| 第10章 · 泛型、trait 与生命周期 | 29 | 7 / 13 / 3 / 6 | 24 / 5 / 0 / 0 | 17 |
| **合计** | **118** | **38 / 46 / 8 / 26** | **87 / 25 / 2 / 4** | **49** |

当前31项残余由25项 `partial`、2项 `related`、4项 `gap` 组成。87项有教学链包括38项基线已具备目标及49项本轮补齐目标；基线已具备的目标没有被重新算作本轮新增。

## 3. 本轮三份讲义与参考材料

| 讲义 | 主要任务 | 完整源码包证据 |
| --- | --- | --- |
| [所有权、结构体方法与字符串切片](BOOK_OWNERSHIP_PRACTICE.md) | move/Copy/Clone、接收者、借用失败与修复、UTF-8切片和Option | `projects/book-d6-foundations/foundation-lab/src/ownership.rs`；对应 `projects/book-d6-foundations/foundation-lab/tests/ownership.rs` |
| [集合与错误传播练习](BOOK_COLLECTIONS_ERRORS.md) | Vec访问/修改、字符串拼接、HashMap策略、match与?及错误转换 | `projects/book-d6-foundations/foundation-lab/src/collections.rs`；对应 `projects/book-d6-foundations/foundation-lab/tests/collections.rs` |
| [泛型、trait与生命周期](BOOK_GENERICS_LIFETIMES.md) | 泛型结构/枚举/方法、trait约束、默认实现、借用返回和生命周期关系 | `projects/book-d6-foundations/foundation-lab/src/generic_lifetimes.rs`；对应 `projects/book-d6-foundations/foundation-lab/tests/generic_lifetimes.rs` |

三份讲义都要求先预测，再运行，在个人副本中修改，最后解释结果。参考源码中的成功例和 `compile_fail` 例应分别阅读；某个预期失败例被编译器拒绝，不代表对应业务功能或个人理解已经通过。具体运行命令以各讲义为准。

## 4. 本次31项残余

下表保留原目标的完整标题和稳定ID。剩余说明只描述尚未满足的教学子点，不撤销已经完成的部分；逐项原有证据和新增证据仍保存在JSON中。

| 稳定ID | 原书小节 | 目标 | 当前状态 | 剩余教学子点 |
| --- | --- | --- | --- | --- |
| `4.1-03` | 4.1 | 字符串字面值与可增长String、String::from和push_str | `partial` | 可增长String及创建/追加已可运行；字符串字面值与拥有String的存储、可变性区别仍未明确讲解，双类型对照不完整。 |
| `4.1-06` | 4.1 | Copy与Clone区别、标量/全Copy元组可复制、含String不可Copy | `partial` | 标量Copy与String克隆已有对照；仍缺全Copy元组与含String元组的区别，以及赋值/传参后的可用性对照。含String派生Copy的旧附录反例继续有效。 |
| `4.1-07` | 4.1 | 函数传参可移动或复制，返回值把所有权交回调用者 | `partial` | 返回值移动和消费接收者已讲清；仍未以调用者持有的String与Copy标量对比普通函数按值传参，参数移动/复制这一子点缺直接教学对照。 |
| `4.2-07` | 4.2 | 多个共享借用可共存，限制是为了阻止冲突读写 | `partial` | 仍缺多个同时活跃共享引用可以共存的正例，以及共享读取为何互不冲突、限制如何阻止冲突读写的明确解释。不能用for共享遍历或RefCell实验替代。 |
| `4.3-05` | 4.3 | 字符串字面值本身是&amp;str且不可变 | `partial` | 新增例子使用字面值调用&amp;str参数，但没有明确讲字面值自身的&amp;str类型、不可变性及静态数据；实际出现字面值不能作为完整教学证据。 |
| `5.1-02` | 5.1 | 修改字段需要可变的整个实例 | `partial` | 没有明确讲普通字段修改要求整个绑定可变、不能仅声明某字段可变，也没有不可变实例直接字段赋值与mut修复的对照。&amp;self调用修改方法的E0596是相关证据，但不能完全替代字段规则。 |
| `5.1-03` | 5.1 | 字段初始化简写 | `partial` | 仍未讲title与title:title、minutes与minutes:minutes的等价关系，也无让学习者展开/简写并验证的任务；不能仅凭构造代码出现简写升级。 |
| `5.1-06` | 5.1 | 类单元结构体无字段，可承载行为且不是() | `partial` | 指定新增材料没有类单元结构体教学。基线ValidationError例仍只是业务错误类型用法，未解释无字段行为承载及与()的区别。 |
| `5.2-01` | 5.2 | 从独立变量/元组到具名字段，表达相关数据的设计理由 | `partial` | 仍无独立变量/元组/具名字段三版接口对照，以及为何字段名字能表达相关数据意义的修改或解释任务。 |
| `5.3-04` | 5.3 | 方法额外参数可借用另一结构体 | `partial` | 已有额外&amp;str参数及借用文本说明；固定目标要求的另一个结构体借用（如other:&amp;ReadingNote）及为何保留调用者对它的所有权，尚无明确教学对照。 |
| `6.1-03` | 6.1 | 枚举可用impl定义有接收者的方法 | `partial` | 这些trait方法是有效业务证据，但新增讲义没有围绕枚举自身的接收者方法进行教学，也未区分枚举的实例方法与关联函数；不能仅因出现impl/调用就升级。 |
| `6.3-03` | 6.3 | if let简洁性的代价是放弃穷尽检查，字面模式只选一个值 | `partial` | 新增材料未对比if let Some(3)在Some(3)/Some(4)/None下的行为，也未说明新增枚举变体时if let不强制穷尽检查；新增match反例只补了match一侧。 |
| `book-8-1-vec-drop-elements` | 8.1 | Vec 离开作用域同时释放所拥有的元素 | `related` | 仍无Vec本身与其拥有元素在作用域结束时一同释放的定向讲解、预测和行为实验。通用Drop与局部String销毁不能替代这个容器目标。 |
| `book-8-2-string-construction` | 8.2 | String::new/from 与 to_string 的创建方式 | `partial` | 三种创建API继续分散出现；尚缺new创建空字符串、from与to_string对同源文本的关系和选择说明及对应小练习。 |
| `book-8-2-string-integer-index` | 8.2 | 整数索引不支持及 UTF-8/O(1) 原因 | `partial` | 整数索引拒绝、UTF-8多字节原因已有完整链；本概念还含O(1)索引预期与第n个scalar需扫描解码的复杂度区别，当前未明确说明这一子点。 |
| `book-8-3-hashmap-construction` | 8.3 | HashMap&lt;K,V&gt;、new/insert 与同质键值 | `partial` | new/insert已成为明确操作，但K/V分别同质的约束仍未清楚解释或验证；不能仅由HashMap&lt;String,u32&gt;的类型出现将整个概念群判满。 |
| `book-8-3-hashmap-collect` | 8.3 | zip 后 collect 为 HashMap 与类型注解 | `gap` | 未增加从键值对/zip调用collect构造HashMap及必要目标类型注解的教学。 |
| `book-8-3-hashmap-ownership` | 8.3 | Copy、String move、引用键值的有效期 | `partial` | String移动与引用键有效期已补齐；本目标中的Copy值插入map后原绑定仍可用尚未直接说明或作对照。所有权课的一般Copy例子是辅助，未自动形成map迁移任务。 |
| `book-8-3-hashmap-hasher` | 8.3 | 默认哈希的取舍与 BuildHasher 扩展入口 | `gap` | 仍无默认hasher取舍与BuildHasher扩展入口的知识说明/索引；本轮可保留剩余，无需引入第三方实现。 |
| `book-9-1-panic-unwind-abort` | 9.1 | panic 的栈展开与 abort 配置区别 | `gap` | 没有补充unwind与abort的差别或配置语义。局部catch_unwind只是观察panic，不能推出已教两种策略。 |
| `book-9-1-panic-location` | 9.1 | 主动 panic、索引触发 panic 与源位置定位 | `partial` | 新增了可观察的panic案例，但没有让学习者读取报告并定位首个自身源码调用点的目标/操作。 |
| `book-9-1-panic-backtrace` | 9.1 | RUST_BACKTRACE 与调用链定位 | `gap` | 仍无RUST_BACKTRACE开启、读取调用链并定位自身代码的练习。 |
| `book-9-2-result-error-kind` | 9.2 | 根据 io::ErrorKind 区分缺失与其它失败 | `partial` | 补充了某一I/O种类识别，但没有把NotFound恢复策略与其它I/O错误分支做成学习者练习；原提供基础设施中的NotFound分支仍只是局部证据。 |
| `book-9-2-result-unwrap-expect` | 9.2 | unwrap/expect 的 panic 语义与诊断信息 | `related` | 新增代码仍主要在断言中使用unwrap；没有unwrap/expect的成功、panic和定制诊断信息对照教学。 |
| `book-9-3-panic-prototype-assumption` | 9.3 | 示例/测试/原型与已知正确值中的 unwrap/expect | `partial` | 仍缺原型/测试中unwrap、expect适用性及已知正确假设依据的明确判断题；新增测试使用unwrap不构成教学。 |
| `book-9-3-validated-types` | 9.3 | 私有字段与受控构造保证有效状态 | `partial` | 新增ReadingNote/Card字段公开且不约束有效范围，不能补齐受限值类型目标。既有D3私有状态/D4校验构造与输入验证到API类型合同的串联仍待补。 |
| `book-10-1-generic-monomorphization` | 10.1 | 单态化与静态分发的基本机制 | `partial` | 补充了编译期具体化的说明；尚无围绕实际实例类型、静态分发与性能边界的预测或解释任务，仍不足以提升整项。 |
| `book-10-2-trait-coherence` | 10.2 | trait 或类型须为本地、孤儿规则与 newtype | `partial` | 增加回链，但外部类型+外部trait被拒绝的机制/对照仍未形成更完整练习；保留原部分状态。 |
| `book-10-2-trait-copy-clone-borrow` | 10.2 | 泛型取最大值中的 PartialOrd/Copy/Clone 与借用替代 | `partial` | 按值Copy/Clone与借用返回的取舍已形成完整教学链。原概念群仍含PartialOrd；当前仅采用并解释Ord全序，尚未说明PartialOrd允许不可比较值以及为什么本实验选择更强Ord边界，故整项仍为partial。 |
| `book-10-2-trait-blanket-impl` | 10.2 | 满足 bound 的类型统一获得 trait 实现 | `partial` | 从仅代码关联增加为明确迁移提示，但仍缺对任意满足bound的类型统一实现trait的通用说明和定向推导/练习。 |
| `book-10-3-lifetime-method-impl` | 10.3 | 带生命周期结构的 impl 与方法独立借用 | `partial` | 字段生存期与方法self/prefix的关系已经解释；impl Excerpt&lt;'_&gt;中的占位参数及其与impl&lt;'a&gt; Excerpt&lt;'a&gt;声明位置的关系未说明，本目标的impl子点仍未完成。 |

## 5. 原书来源与版本处理

以下18个页面是本次目标拆分的原书来源，均来自 [rust.bootcss.com](https://rust.bootcss.com/)。课程沿用现有工程的Edition、MSRV和锁文件；旧译文的表述不自动成为修改工具链或公开API的理由。

| 小节 | 原书页面 |
| --- | --- |
| 4.1 | [什么是所有权？](https://rust.bootcss.com/ch04-01-what-is-ownership.html) |
| 4.2 | [引用与借用](https://rust.bootcss.com/ch04-02-references-and-borrowing.html) |
| 4.3 | [Slice 类型](https://rust.bootcss.com/ch04-03-slices.html) |
| 5.1 | [定义并实例化结构体](https://rust.bootcss.com/ch05-01-defining-structs.html) |
| 5.2 | [一个使用结构体的示例程序](https://rust.bootcss.com/ch05-02-example-structs.html) |
| 5.3 | [方法语法](https://rust.bootcss.com/ch05-03-method-syntax.html) |
| 6.1 | [定义枚举](https://rust.bootcss.com/ch06-01-defining-an-enum.html) |
| 6.2 | [match 控制流运算符](https://rust.bootcss.com/ch06-02-match.html) |
| 6.3 | [if let 简洁控制流](https://rust.bootcss.com/ch06-03-if-let.html) |
| 8.1 | [Vec](https://rust.bootcss.com/ch08-01-vectors.html) |
| 8.2 | [String](https://rust.bootcss.com/ch08-02-strings.html) |
| 8.3 | [HashMap](https://rust.bootcss.com/ch08-03-hash-maps.html) |
| 9.1 | [panic! 与不可恢复错误](https://rust.bootcss.com/ch09-01-unrecoverable-errors-with-panic.html) |
| 9.2 | [Result 与可恢复错误](https://rust.bootcss.com/ch09-02-recoverable-errors-with-result.html) |
| 9.3 | [panic 还是 Result](https://rust.bootcss.com/ch09-03-to-panic-or-not-to-panic.html) |
| 10.1 | [泛型数据类型](https://rust.bootcss.com/ch10-01-syntax.html) |
| 10.2 | [trait：定义共享行为](https://rust.bootcss.com/ch10-02-traits.html) |
| 10.3 | [生命周期与引用有效性](https://rust.bootcss.com/ch10-03-lifetime-syntax.html) |

旧译文8.3对部分返回类型的描述不一致：教学材料使用 `HashMap::get -> Option<&V>` 和 `Entry::or_insert -> &mut V`。9.2以Result介绍 `?`，课程不将它扩写为“所有情况下只能用于Result”。原书末尾的领域练习可以由具有同等目标的真实作品承担，不要求逐题照抄。

## 6. 如何复核与继续

按稳定ID打开JSON，先读原目标、`baseline_evidence` 和 `new_evidence`，再检查 `remaining_gap`。所有权基线的 `evidence_ids` 已解析为路径及locator，原代号保留用于追溯；源码证据应在完整源码包中查找。

后续只对确实补齐教学子点的目标更新状态，并保留个人输入、修改、解释题和试用记录。发布验收与个人学习验收应各有自己的记录；这份抽样审计不替代其中任何一种。
