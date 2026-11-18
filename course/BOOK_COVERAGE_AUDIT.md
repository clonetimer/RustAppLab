# D0 · Rust Book 中文版逐节覆盖审计（基线）

- **参考目录**：[rust.bootcss.com](https://rust.bootcss.com/)，Rust 1.37 / Edition 2018 书系，第 1–20 章、附录 A–G；不是当前 Edition 2024 Rust Book 的同一章节编号。
- **历史源码基准**：`clonetimer/RustAppLab` main `fed4a2d37a72de88dfb353474544efde9bdd537a`（2026-11-17 审计时）。
- **最近复核**：2026-11-18；从 PR #10 基线 `6a5827f4e3ce6d009836a7d2beaa9c9285ada24a` 复核原书目录与实际材料，补充 1.3、3.1、3.2、7.2–7.5 的具体练习。新增材料的远端验收以 PR 当前提交为准，不继承 Run 215/222。
- **证据规则**：表格中“已有实作”只表示可以定位到相关教学或可执行实例，**不表示**该小节原书所有示例、知识点、学员练习与测试均已完整覆盖；“部分”表示只有关联知识或实现；“缺口”明确有需要补的专项内容；“开发中”只证明开发分支已有材料、仍须 CI。
- **验收判据**：每项最终需要有教学目标、版本适配、学员实验或合理的知识索引、可重现检查及用户体验验证。官方原书代码无需照抄，允许使用现有真实作品替换；不能用出现某个关键词冒充教学覆盖。
- **当前统计**：共 **81** 项审计条目，其中 已有实作 **79**、部分 **1**（D 的 IDE 人工操作仍待记录）、缺口 0、开发中 0、资料索引 **1**（F）。这是主题映射和指定参考实验的状态盘点，**不是**知识点完整率或学习达标率。D1–D4、D5/D6-A、D6-B 的历史验收分别见 PR #6、PR #8 / Run 197、PR #9 / Run 215；它们不替代本次新增代码的验证。2026-11-18 新增基础实验在 Linux 的 Rust 1.85.0/1.74.0 上 Debug/Release 各 29 项、Doctest 15 项通过；E/G 已有真实迁移和固定日期 Nightly 对照证据。D/E 的最终 Linux/Windows 与交付状态见 PR #10 当前提交的完整验收。
- **推进路线**：D1 Cargo（14.1–14.5）→ D2 智能指针（15.1–15.6）→ D3 对象模型/模式（17–18）→ D4 Unsafe/高级 Trait/宏（19）→ D5 原生多线程 Web Server（20）→ D6-A 附录 A–G → D6-B 基础知识残余缺口；随每阶段修订本表。

| 原书小节 | 主题 | 审计状态 | 现有代码/讲义证据（仓库相对路径） | 后续核查或补充 |
|---|---|---|---|---|
| 1.1 | 安装 Rust 与工具链 | 已有实作 | `README_START_HERE.md` | 系统工具安装由用户完成；无需重复开发 |
| 1.2 | Hello World | 已有实作 | `course/BOOK_D6_FOUNDATIONS.md` | 最小 crate、main、println 与 Cargo 运行实验；代码见 `projects/book-d6-foundations/foundation-lab/src/main.rs`；Run 215 Linux/Windows、Rust 1.74 MSRV、Debug/Release/Doctest PASS |
| 1.3 | Hello Cargo | 已有实作 | `course/BOOK_D6_FOUNDATIONS.md` | Cargo package/crate、manifest/lock、check/build/run 与构建模式复习；三步操作与解释题；不能只以能运行预置工程代替理解 |
| 2 | 猜数字游戏教程 | 已有实作 | `course/BOOK_D6_FOUNDATIONS.md` | 输入、解析、比较、循环、无效输入与退出；可运行 guess 子命令；Run 215 Linux/Windows、Rust 1.74 MSRV、Debug/Release/Doctest PASS |
| 3.1 | 变量与可变性 | 已有实作 | `course/BOOK_D6_LANGUAGE_BASICS.md` | const、let、mut、遮蔽；不可变重赋值、重赋值改变类型、常量赋值的真实编译失败；语言实验含实际解析/累计行为测试 |
| 3.2 | 数据类型 | 已有实作 | `course/BOOK_D6_FOUNDATIONS.md` | 三次数组分数汇总、混合元组、解构/.0、单元、重复数组、get/索引越界；数组长度与元素类型 E0308 反例及修复对照 |
| 3.3 | 函数 | 已有实作 | `course/studio/unitlab.md` | 现有公开合同测试 |
| 3.4 | 注释 | 已有实作 | `course/BOOK_D6_FOUNDATIONS.md` | 行/块注释、项文档、crate 文档与 doctest；Run 215 Linux/Windows、Rust 1.74 MSRV、Debug/Release/Doctest PASS |
| 3.5 | 控制流 | 已有实作 | `course/studio/quizdeck.md` | 继续复用分支与循环 |
| 4.1 | 所有权 | 已有实作 | `course/studio/textscope.md` | 含移动错误实验 |
| 4.2 | 引用与借用 | 已有实作 | `course/RUST_ERROR_LABS.md` | 含借用编译错误实验 |
| 4.3 | 切片 | 已有实作 | `course/studio/textscope.md` | UTF-8 与字符串切片 |
| 5.1 | 定义结构体 | 已有实作 | `course/studio/quizdeck.md` | 业务数据结构 |
| 5.2 | 结构体示例程序 | 已有实作 | `course/BOOK_D6_FOUNDATIONS.md` | Rectangle 结构体、字段、构造与面积；Run 215 Linux/Windows、Rust 1.74 MSRV、Debug/Release/Doctest PASS |
| 5.3 | 方法语法 | 已有实作 | `course/BOOK_D6_FOUNDATIONS.md` | impl、self、关联函数与方法调用；Run 215 Linux/Windows、Rust 1.74 MSRV、Debug/Release/Doctest PASS |
| 6.1 | 枚举 | 已有实作 | `course/studio/unitlab.md` | Unit enum |
| 6.2 | match | 已有实作 | `course/studio/unitlab.md` | 分支转换 |
| 6.3 | if let | 已有实作 | `course/BOOK_D6_FOUNDATIONS.md` | Option 分支的 if let 与 match 对照；Run 215 Linux/Windows、Rust 1.74 MSRV、Debug/Release/Doctest PASS |
| 7.1 | 包和 crate | 已有实作 | `course/ADVANCED_B.md` | B1–B3 多 crate 工程 |
| 7.2 | 模块作用域与私有性 | 已有实作 | `course/BOOK_D6_LANGUAGE_BASICS.md` | 私有 catalog 模块、StudyCard 私有字段、公有 Activity 枚举；外部集成测试和 E0603/E0616 编译失败实验 |
| 7.3 | 路径 | 已有实作 | `course/BOOK_D6_LANGUAGE_BASICS.md` | 同一 crate 内实际 crate::、self::、super:: 路径；明确与 Cargo path 依赖区别，替换原错位映射 |
| 7.4 | use 关键字 | 已有实作 | `course/BOOK_D6_LANGUAGE_BASICS.md` | use、as 别名、pub use 重导出；外部集成测试通过公开入口访问模型 |
| 7.5 | 拆分模块文件 | 已有实作 | `course/BOOK_D6_LANGUAGE_BASICS.md` | lib.rs → language_basics.rs → language_basics/catalog.rs 实际分文件 mod；同一 crate 的模块与多个 crate 分开讲解 |
| 8.1 | Vector | 已有实作 | `course/studio/quizdeck.md` | 题库列表 |
| 8.2 | String | 已有实作 | `course/studio/textscope.md` | UTF-8 与所有权 |
| 8.3 | HashMap | 已有实作 | `course/BOOK_D6_FOUNDATIONS.md` | HashMap、Entry API、词频统计与确定性输出；Run 215 Linux/Windows、Rust 1.74 MSRV、Debug/Release/Doctest PASS |
| 9.1 | panic! | 已有实作 | `course/BOOK_D6_FOUNDATIONS.md` | panic 与程序不变量边界，catch_unwind 仅作行为实验；Run 215 Linux/Windows、Rust 1.74 MSRV、Debug/Release/Doctest PASS |
| 9.2 | Result | 已有实作 | `course/studio/ledgerlite.md` | 错误传播及文件失败 |
| 9.3 | panic 还是返回错误 | 已有实作 | `course/BOOK_D6_FOUNDATIONS.md` | 外部输入 Result 与内部不变量 panic 的决策练习；Run 215 Linux/Windows、Rust 1.74 MSRV、Debug/Release/Doctest PASS |
| 10.1 | 泛型 | 已有实作 | `course/studio/badgeforge.md` | 泛型渲染 |
| 10.2 | Trait | 已有实作 | `course/ADVANCED_A.md` | Renderer 与 trait object |
| 10.3 | 生命周期 | 已有实作 | `course/ADVANCED_A.md` | 签名、借用作用域、compile_fail |
| 11.1 | 编写测试 | 已有实作 | `course/BOOK_GUIDE.md` | 单元/集成/DocTest |
| 11.2 | 运行测试 | 已有实作 | `course/ADVANCED_B.md` | Cargo test 门禁 |
| 11.3 | 测试的组织结构 | 已有实作 | `course/BOOK_GUIDE.md` | 固定公开合同 |
| 12.1 | 命令行参数 | 已有实作 | `course/01_cli_contract.md` | CLI 应用 |
| 12.2 | 读取文件 | 已有实作 | `course/02_filesystem.md` | 文件扫描与读取 |
| 12.3 | 模块化与错误处理 | 已有实作 | `course/ADVANCED_B.md` | 业务库抽离 |
| 12.4 | 测试驱动开发 | 已有实作 | `course/INDEPENDENT_PATH.md` | 先写公开行为合同 |
| 12.5 | 环境变量 | 已有实作 | `course/BOOK_D6_FOUNDATIONS.md` | 环境变量优先级、OsString 与可注入配置路径测试；Run 215 Linux/Windows、Rust 1.74 MSRV、Debug/Release/Doctest PASS |
| 12.6 | 标准错误输出 | 已有实作 | `course/03_error_handling.md` | 非零退出码、stderr |
| 13.1 | 闭包 | 已有实作 | `course/ADVANCED_A.md` | Fn / FnMut / FnOnce |
| 13.2 | 迭代器 | 已有实作 | `course/studio/textscope.md` | 过滤、聚合与排序 |
| 13.3 | 用迭代器改进 I/O 项目 | 已有实作 | `course/BOOK_D6_FOUNDATIONS.md` | BufRead、lines、try_fold 与 I/O 错误传播；Run 215 Linux/Windows、Rust 1.74 MSRV、Debug/Release/Doctest PASS |
| 13.4 | 循环与迭代器性能比较 | 已有实作 | `course/BOOK_D6_FOUNDATIONS.md` | 同输入等价实现、black_box、release 模式计时与测量纪律；Run 215 Linux/Windows、Rust 1.74 MSRV、Debug/Release/Doctest PASS |
| 14.1 | Profile 构建配置 | 已有实作 | `course/BOOK_D1_CARGO.md` | D1 已添加 profile 实验；独立 Workspace 已随 PR #6 合并，D0–D4 验收矩阵 17/17 CI Jobs PASS |
| 14.2 | crate 打包与发布 | 已有实作 | `course/BOOK_D1_CARGO.md` | 仅安全打包检查，不做实际 registry 上传；独立 Workspace 已随 PR #6 合并，D0–D4 验收矩阵 17/17 CI Jobs PASS |
| 14.3 | Cargo Workspace | 已有实作 | `projects/book-d1-cargo/Cargo.toml` | 复用原 B 课程，再加 D1 独立工作区；独立 Workspace 已随 PR #6 合并，D0–D4 验收矩阵 17/17 CI Jobs PASS |
| 14.4 | cargo install | 已有实作 | `course/BOOK_D1_CARGO.md` | 仅本地路径和隔离 root；独立 Workspace 已随 PR #6 合并，D0–D4 验收矩阵 17/17 CI Jobs PASS |
| 14.5 | Cargo 自定义子命令 | 已有实作 | `projects/book-d1-cargo/cargo-applab-map/src/main.rs` | 添加真实 cargo-* 命令；独立 Workspace 已随 PR #6 合并，D0–D4 验收矩阵 17/17 CI Jobs PASS |
| 15.1 | Box | 已有实作 | `projects/book-d2-smart-pointers/smart-pointer-lab/src/lib.rs` | 递归类型、堆间接层与 Box trait object 已有可运行示例；独立 Workspace 已随 PR #6 合并，D0–D4 验收矩阵 17/17 CI Jobs PASS |
| 15.2 | Deref | 已有实作 | `projects/book-d2-smart-pointers/smart-pointer-lab/src/lib.rs` | 自定义 MyBox、Deref/DerefMut 与 String 到 str 强制转换；独立 Workspace 已随 PR #6 合并，D0–D4 验收矩阵 17/17 CI Jobs PASS |
| 15.3 | Drop | 已有实作 | `projects/book-d2-smart-pointers/smart-pointer-lab/src/lib.rs` | 自动析构与 std::mem::drop 时机实验；独立 Workspace 已随 PR #6 合并，D0–D4 验收矩阵 17/17 CI Jobs PASS |
| 15.4 | Rc | 已有实作 | `projects/book-d2-smart-pointers/smart-pointer-lab/src/lib.rs` | Rc clone、strong_count 与单线程共享所有权实验；独立 Workspace 已随 PR #6 合并，D0–D4 验收矩阵 17/17 CI Jobs PASS |
| 15.5 | RefCell | 已有实作 | `projects/book-d2-smart-pointers/smart-pointer-lab/src/lib.rs` | RefCell 运行时借用冲突、try_borrow_mut 与 Rc<RefCell<T>> 实验；独立 Workspace 已随 PR #6 合并，D0–D4 验收矩阵 17/17 CI Jobs PASS |
| 15.6 | 引用循环 | 已有实作 | `projects/book-d2-smart-pointers/smart-pointer-lab/src/lib.rs` | parent-child 强拥有/Weak 回边与父节点释放实验；独立 Workspace 已随 PR #6 合并，D0–D4 验收矩阵 17/17 CI Jobs PASS |
| 16.1 | 线程 | 已有实作 | `course/SYSTEMS_PATH.md` | BatchMeter 并发 |
| 16.2 | 消息传递 | 已有实作 | `course/SYSTEMS_PATH.md` | mpsc |
| 16.3 | 共享状态 | 已有实作 | `course/ADVANCED_A.md` | Arc 与状态边界 |
| 16.4 | Send 与 Sync | 已有实作 | `course/ADVANCED_A.md` | 跨线程快照 |
| 17.1 | 面向对象特性 | 已有实作 | `projects/book-d3-oop-patterns/oop-pattern-lab/src/lib.rs` | 封装不变量、组合与 trait 多态参考实验；独立 Workspace 已随 PR #6 合并，D0–D4 验收矩阵 17/17 CI Jobs PASS |
| 17.2 | Trait Object | 已有实作 | `projects/book-d3-oop-patterns/oop-pattern-lab/src/lib.rs` | Box<dyn Draw> 多种具体类型运行时分发；独立 Workspace 已随 PR #6 合并，D0–D4 验收矩阵 17/17 CI Jobs PASS |
| 17.3 | 状态模式 | 已有实作 | `projects/book-d3-oop-patterns/oop-pattern-lab/src/lib.rs` | Draft→PendingReview→Published 消费式状态转换；独立 Workspace 已随 PR #6 合并，D0–D4 验收矩阵 17/17 CI Jobs PASS |
| 18.1 | 使用模式的位置 | 已有实作 | `projects/book-d3-oop-patterns/oop-pattern-lab/src/lib.rs` | match、if let、while let、for、let-else 与函数参数模式；独立 Workspace 已随 PR #6 合并，D0–D4 验收矩阵 17/17 CI Jobs PASS |
| 18.2 | 模式可反驳性 | 已有实作 | `projects/book-d3-oop-patterns/oop-pattern-lab/src/lib.rs` | let-else/if let 对照与 compile_fail doctest；独立 Workspace 已随 PR #6 合并，D0–D4 验收矩阵 17/17 CI Jobs PASS |
| 18.3 | 模式语法 | 已有实作 | `projects/book-d3-oop-patterns/oop-pattern-lab/src/lib.rs` | 字面量、多模式、范围、结构体/枚举/切片/元组解构、ref、@、守卫；独立 Workspace 已随 PR #6 合并，D0–D4 验收矩阵 17/17 CI Jobs PASS |
| 19.1 | Unsafe Rust | 已有实作 | `projects/book-d4-advanced-features/advanced-lab/src/lib.rs` | 裸指针、split_at_mut 安全抽象、unsafe fn、extern C、unsafe trait、union、受限 static mut；Linux/Windows PASS |
| 19.2 | 高级 Trait | 已有实作 | `projects/book-d4-advanced-features/advanced-lab/src/lib.rs` | 关联类型、默认泛型参数、运算符重载、完全限定语法、超 Trait、Newtype；Linux/Windows PASS |
| 19.3 | 高级类型 | 已有实作 | `projects/book-d4-advanced-features/advanced-lab/src/lib.rs` | Newtype、类型别名、Never、DST、?Sized；Linux/Windows PASS |
| 19.4 | 高级函数/闭包 | 已有实作 | `projects/book-d4-advanced-features/advanced-lab/src/lib.rs` | 函数指针、非捕获闭包强制转换、枚举构造函数、返回闭包与 Box<dyn Fn>；Linux/Windows PASS |
| 19.5 | 宏 | 已有实作 | `projects/book-d4-advanced-features/advanced-lab/src/lib.rs` | macro_rules!、derive、属性宏与函数式过程宏由本地 d4-proc-macros crate 实现；结构体/枚举展开测试通过 |
| 20.1 | 单线程 Web Server | 已有实作 | `projects/book-d5-webserver/webserver-lab/src/lib.rs` | 原生 TcpListener/TcpStream、HTTP 请求解析与有限连接基线；Run 197 Linux/Windows、Debug/Release、Doctest PASS |
| 20.2 | 多线程 Web Server | 已有实作 | `projects/book-d5-webserver/webserver-lab/src/lib.rs` | 固定大小 Worker、MPSC Job 队列与 Arc/Mutex 接收端；Run 197 Linux/Windows、Debug/Release、Doctest PASS |
| 20.3 | 优雅关闭 | 已有实作 | `projects/book-d5-webserver/webserver-lab/src/lib.rs` | shutdown channel、Drop 发送全部终止消息后 join、排空已提交任务；Run 197 Linux/Windows、Debug/Release、Doctest PASS |
| A | 关键字 | 已有实作 | `course/BOOK_D6_APPENDICES.md` | 当前 Strict/Reserved/Weak 完整清单、raw identifiers 与 Edition 差异；精确表格回归与 2 项 compile-fail 文档测试，Run 215 PASS |
| B | 运算符与符号 | 已有实作 | `course/BOOK_D6_APPENDICES.md` | 常见运算符、符号语境、完整优先级与 Add 重载实验；位运算层级及比较/范围约束回归，Run 215 PASS |
| C | 可派生 Trait | 已有实作 | `course/BOOK_D6_APPENDICES.md` | 九种标准 derive trait 的能力、约束、Debug/Display 区分与 compile-fail 实验；Run 197 Linux/Windows/MSRV PASS |
| D | 实用开发工具 | 部分 | `course/BOOK_D6_PRACTICAL.md` | 工具链 CLI、unused_mut 诊断/修复、格式/Clippy/测试/文档生成已有真实运行证据；rust-analyzer 实际 IDE 跳转、诊断和修复仍需人工记录 |
| E | Edition 与版本 | 已有实作 | `course/BOOK_D6_PRACTICAL.md` | Rust 1.85.0 临时副本中先 fix 后改 Edition/MSRV；gen→r#gen 实际差异；12 Debug、3 Doc、12 Release 测试，正式源码快照不变 |
| F | 翻译版本 | 资料索引 | `course/BOOK_D6_APPENDICES.md` | 区分旧中文译本基线与当前官方书，提供翻译/版本导航；不要求单独编码练习 |
| G | Rust 开发流程与 Nightly | 已有实作 | `course/BOOK_D6_PRACTICAL.md` | Linux 的 nightly-2026-11-17 实验通过；Rust 1.85 带 gate 的 E0554 与去 gate 后泛型常量错误双对照；日期/退出码/源码快照均保存，属选修证据 |

## 不纳入覆盖率的情况

- C1/C2 的 Tokio 与 Axum 是有价值的增量，但不能替代原书第 20 章的同步线程池设计；SQLite FFI 中出现 `unsafe` 也不能证明已学会 Unsafe Rust。
- 编译测试、自动化门禁验证可执行行为，不能自动证明学员理解所有权、接口设计与编译错误；每阶段还应保留个人输入、新测试、解释性答题与试用记录。
- 旧 Rust Book 的 API、Cargo 行为可能随工具链变化；运行命令以 `Cargo.lock`、`rust-version` 和官方 Cargo Book 对照验证，不能只凭中文版描述更新当前工程。

## 里程碑签核表

| 里程碑 | 交付定义 | 当前情况 |
|---|---|---|
| D0 | 原书全目录逐节证据和风险映射 | 81 个原书 ID 已逐项核对；映射与关键缺口于 2026-11-18 独立复核；尚未逐句量化整本书知识点完整率 |
| D1 | 5 节 Cargo 讲义 + 隔离 Workspace + Cargo 外部子命令 + 真 CI | PR #6 已合并；D0–D4 验收矩阵 17/17 CI Jobs PASS |
| D2 | 智能指针 6 节 + RAII/Deref/Weak 实验 | PR #6 已合并；D0–D4 验收矩阵 17/17 CI Jobs PASS |
| D3 | 面向对象与模式匹配 6 节 | PR #6 已合并；D0–D4 验收矩阵 17/17 CI Jobs PASS |
| D4 | 第 19 章 5 节安全隔离实验 | D4-A2 17/17 CI Jobs PASS，PR #6 已合并 |
| D5 | 第 20 章 3 节线程池与关闭 | PR #8 已合并；Run 197 的 Linux/Windows/MSRV、交付与浏览器矩阵 17 Jobs 全部成功 |
| D6-A | 附录 A–G | A–C 历史参考实验通过 Run 197；D/E/G 增加真实命令运行器与证据，D 的 IDE 操作单独记录，F 为资料索引 |
| D6-B | 12 项基础知识残余缺口及后续基础复核 | 原 12 项见 PR #9 / Run 215；本轮再补 1.3、3.1、3.2、7.2–7.5，使用当前提交重新验收 |
| D6-C | 工具、Edition 与隔离 Nightly | 唯一副本、失败保留、原始日志/差异/校验；E/G 参考行为已实跑，D 的 IDE 操作与个人学习另记 |

## 2026-11-18 复核补充：三种证据分开记录

| 层次 | 本次如何检查 | 不能推导什么 |
|---|---|---|
| 目录映射 | 原书 74 个正文条目与 7 个附录；测试核对完整 81 个 ID 集合 | 不能将条目比例当知识点完整率 |
| 参考行为 | 基础实验的成功/编译失败对照；工具、迁移与隔离脚本实际执行；源码和交付物校验 | 不能代替 IDE 手动体验或个人理解 |
| 个人学习 | 学习者自己的输入、修改、测试、解释题与试用记录 | 没有记录时保持未评估 |

本轮补的是先前粗映射下遗漏的基础讲解与可复现验证。14.2 不实际上传 crates.io、14.4 使用本地安装与隔离 root，属于保留的课程边界；不要求为了补齐统计而进行真实发布或额外下载第三方工具。

尚未逐句拆解原书全部概念，因此剩余细粒度知识点数量仍未量化。已定位的专项有明确代码与讲义证据；后续审查以发现的具体目标和学习阻断为单位，不能只追求一个百分比。
