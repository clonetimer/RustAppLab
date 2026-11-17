# D0 · Rust Book 中文版逐节覆盖审计（基线）

- **参考目录**：[rust.bootcss.com](https://rust.bootcss.com/)，Rust 1.37 / Edition 2018 书系，第 1–20 章、附录 A–G；不是当前 Edition 2024 Rust Book 的同一章节编号。
- **源码基准**：`clonetimer/RustAppLab` main `fed4a2d37a72de88dfb353474544efde9bdd537a`（2026-11-17 审计时）。
- **证据规则**：表格中“已有实作”只表示可以定位到相关教学或可执行实例，**不表示**该小节原书所有示例、知识点、学员练习与测试均已完整覆盖；“部分”表示只有关联知识或实现；“缺口”明确有需要补的专项内容；“开发中”只证明开发分支已有材料、仍须 CI。
- **验收判据**：每项最终需要有教学目标、版本适配、学员实验或合理的知识索引、可重现检查及用户体验验证。官方原书代码无需照抄，允许使用现有真实作品替换；不能用出现某个关键词冒充教学覆盖。
- **当前统计**：共 **81** 项审计条目，其中 已有实作 42、部分 17、缺口 4、开发中 17、资料索引 1。D1（14.1–14.5）、D2（15.1–15.6）、D3（17.1–17.3、18.1–18.3）和 D4（19.1–19.5）已进入可运行参考实验与跨平台门禁；D4 已完成 Linux/Windows、Rust 1.74 与交付候选包验收。这是条目盘点数，**不是**知识点覆盖率或学习达标率。
- **推进路线**：D1 Cargo（14.1–14.5）→ D2 智能指针（15.1–15.6）→ D3 对象模型/模式（17–18）→ D4 Unsafe/高级 Trait/宏（19）→ D5 原生多线程 Web Server（20）→ D6 附录与残余基础缺口；随每阶段修订本表。

| 原书小节 | 主题 | 审计状态 | 现有代码/讲义证据（仓库相对路径） | 后续核查或补充 |
|---|---|---|---|---|
| 1.1 | 安装 Rust 与工具链 | 已有实作 | `README_START_HERE.md` | 系统工具安装由用户完成；无需重复开发 |
| 1.2 | Hello World | 部分 | `course/BEGINNER_WINDOWS_GUIDE.md` | FirstCard 可运行，但缺 Hello World 专门对照 |
| 1.3 | Hello Cargo | 已有实作 | `course/BEGINNER_WINDOWS_GUIDE.md` | 补充 Cargo 术语复习即可 |
| 2 | 猜数字游戏教程 | 部分 | `course/studio/quizdeck.md` | QuizDeck 涵盖输入、判断和随机性相关之外的基础；原书猜数字练习未复现 |
| 3.1 | 变量与可变性 | 已有实作 | `course/FOUNDATION_PATH.md` | 已有任务卡 |
| 3.2 | 数据类型 | 已有实作 | `course/studio/unitlab.md` | 需补数组/元组专题验证 |
| 3.3 | 函数 | 已有实作 | `course/studio/unitlab.md` | 现有公开合同测试 |
| 3.4 | 注释 | 部分 | `course/RUST_ERROR_LABS.md` | 缺文档注释/普通注释专题练习 |
| 3.5 | 控制流 | 已有实作 | `course/studio/quizdeck.md` | 继续复用分支与循环 |
| 4.1 | 所有权 | 已有实作 | `course/studio/textscope.md` | 含移动错误实验 |
| 4.2 | 引用与借用 | 已有实作 | `course/RUST_ERROR_LABS.md` | 含借用编译错误实验 |
| 4.3 | 切片 | 已有实作 | `course/studio/textscope.md` | UTF-8 与字符串切片 |
| 5.1 | 定义结构体 | 已有实作 | `course/studio/quizdeck.md` | 业务数据结构 |
| 5.2 | 结构体示例程序 | 部分 | `course/studio/ledgerlite.md` | 不是原书矩形案例，需概念一一校对 |
| 5.3 | 方法语法 | 部分 | `course/FOUNDATION_NEXT_STEPS.md` | 存在方法使用，缺专门学员合同 |
| 6.1 | 枚举 | 已有实作 | `course/studio/unitlab.md` | Unit enum |
| 6.2 | match | 已有实作 | `course/studio/unitlab.md` | 分支转换 |
| 6.3 | if let | 部分 | `course/03_error_handling.md` | 代码中有示例，缺专项练习 |
| 7.1 | 包和 crate | 已有实作 | `course/ADVANCED_B.md` | B1–B3 多 crate 工程 |
| 7.2 | 模块作用域与私有性 | 已有实作 | `course/ADVANCED_B.md` | 业务库与应用入口 |
| 7.3 | 路径 | 已有实作 | `course/ADVANCED_B.md` | 路径依赖 |
| 7.4 | use 关键字 | 已有实作 | `course/studio/badgeforge.md` | 练习中导入接口 |
| 7.5 | 拆分模块文件 | 已有实作 | `course/ADVANCED_B.md` | 多模块/多 crate |
| 8.1 | Vector | 已有实作 | `course/studio/quizdeck.md` | 题库列表 |
| 8.2 | String | 已有实作 | `course/studio/textscope.md` | UTF-8 与所有权 |
| 8.3 | HashMap | 部分 | `course/SYSTEMS_PATH.md` | MiniKV 内部键值结构，缺 HashMap 独立解释 |
| 9.1 | panic! | 部分 | `course/RUST_ERROR_LABS.md` | 错误实验已有，panic 恢复边界待补 |
| 9.2 | Result | 已有实作 | `course/studio/ledgerlite.md` | 错误传播及文件失败 |
| 9.3 | panic 还是返回错误 | 部分 | `course/studio/minikv.md` | 实际设计规则存在，缺决策练习 |
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
| 12.5 | 环境变量 | 部分 | `course/INDEPENDENT_PATH.md` | 有工程应用，但无章节对应练习 |
| 12.6 | 标准错误输出 | 已有实作 | `course/03_error_handling.md` | 非零退出码、stderr |
| 13.1 | 闭包 | 已有实作 | `course/ADVANCED_A.md` | Fn / FnMut / FnOnce |
| 13.2 | 迭代器 | 已有实作 | `course/studio/textscope.md` | 过滤、聚合与排序 |
| 13.3 | 用迭代器改进 I/O 项目 | 部分 | `course/25_labquest_rust_code.md` | 含迭代器，但原书对照未复测 |
| 13.4 | 循环与迭代器性能比较 | 缺口 | `—` | 缺可重复的独立对比实验 |
| 14.1 | Profile 构建配置 | 开发中 | `course/BOOK_D1_CARGO.md` | D1 已添加 profile 实验，待 CI |
| 14.2 | crate 打包与发布 | 开发中 | `course/BOOK_D1_CARGO.md` | 仅安全打包检查，不做实际 registry 上传 |
| 14.3 | Cargo Workspace | 开发中 | `projects/book-d1-cargo/Cargo.toml` | 复用原 B 课程，再加 D1 独立工作区 |
| 14.4 | cargo install | 开发中 | `course/BOOK_D1_CARGO.md` | 仅本地路径和隔离 root |
| 14.5 | Cargo 自定义子命令 | 开发中 | `projects/book-d1-cargo/cargo-applab-map/src/main.rs` | 添加真实 cargo-* 命令，待 CI |
| 15.1 | Box | 开发中 | `projects/book-d2-smart-pointers/smart-pointer-lab/src/lib.rs` | 递归类型、堆间接层与 Box trait object 已有可运行示例，待 CI |
| 15.2 | Deref | 开发中 | `projects/book-d2-smart-pointers/smart-pointer-lab/src/lib.rs` | 自定义 MyBox、Deref/DerefMut 与 String 到 str 强制转换，待 CI |
| 15.3 | Drop | 开发中 | `projects/book-d2-smart-pointers/smart-pointer-lab/src/lib.rs` | 自动析构与 std::mem::drop 时机实验，待 CI |
| 15.4 | Rc | 开发中 | `projects/book-d2-smart-pointers/smart-pointer-lab/src/lib.rs` | Rc clone、strong_count 与单线程共享所有权实验，待 CI |
| 15.5 | RefCell | 开发中 | `projects/book-d2-smart-pointers/smart-pointer-lab/src/lib.rs` | RefCell 运行时借用冲突、try_borrow_mut 与 Rc<RefCell<T>> 实验，待 CI |
| 15.6 | 引用循环 | 开发中 | `projects/book-d2-smart-pointers/smart-pointer-lab/src/lib.rs` | parent-child 强拥有/Weak 回边与父节点释放实验，待 CI |
| 16.1 | 线程 | 已有实作 | `course/SYSTEMS_PATH.md` | BatchMeter 并发 |
| 16.2 | 消息传递 | 已有实作 | `course/SYSTEMS_PATH.md` | mpsc |
| 16.3 | 共享状态 | 已有实作 | `course/ADVANCED_A.md` | Arc 与状态边界 |
| 16.4 | Send 与 Sync | 已有实作 | `course/ADVANCED_A.md` | 跨线程快照 |
| 17.1 | 面向对象特性 | 开发中 | `projects/book-d3-oop-patterns/oop-pattern-lab/src/lib.rs` | 封装不变量、组合与 trait 多态参考实验，待 CI |
| 17.2 | Trait Object | 开发中 | `projects/book-d3-oop-patterns/oop-pattern-lab/src/lib.rs` | Box<dyn Draw> 多种具体类型运行时分发，待 CI |
| 17.3 | 状态模式 | 开发中 | `projects/book-d3-oop-patterns/oop-pattern-lab/src/lib.rs` | Draft→PendingReview→Published 消费式状态转换，待 CI |
| 18.1 | 使用模式的位置 | 开发中 | `projects/book-d3-oop-patterns/oop-pattern-lab/src/lib.rs` | match、if let、while let、for、let-else 与函数参数模式，待 CI |
| 18.2 | 模式可反驳性 | 开发中 | `projects/book-d3-oop-patterns/oop-pattern-lab/src/lib.rs` | let-else/if let 对照与 compile_fail doctest，待 CI |
| 18.3 | 模式语法 | 开发中 | `projects/book-d3-oop-patterns/oop-pattern-lab/src/lib.rs` | 字面量、多模式、范围、结构体/枚举/切片/元组解构、ref、@、守卫，待 CI |
| 19.1 | Unsafe Rust | 已有实作 | `projects/book-d4-advanced-features/advanced-lab/src/lib.rs` | 裸指针、split_at_mut 安全抽象、unsafe fn、extern C、unsafe trait、union、受限 static mut；Linux/Windows PASS |
| 19.2 | 高级 Trait | 已有实作 | `projects/book-d4-advanced-features/advanced-lab/src/lib.rs` | 关联类型、默认泛型参数、运算符重载、完全限定语法、超 Trait、Newtype；Linux/Windows PASS |
| 19.3 | 高级类型 | 已有实作 | `projects/book-d4-advanced-features/advanced-lab/src/lib.rs` | Newtype、类型别名、Never、DST、?Sized；Linux/Windows PASS |
| 19.4 | 高级函数/闭包 | 已有实作 | `projects/book-d4-advanced-features/advanced-lab/src/lib.rs` | 函数指针、非捕获闭包强制转换、枚举构造函数、返回闭包与 Box<dyn Fn>；Linux/Windows PASS |
| 19.5 | 宏 | 已有实作 | `projects/book-d4-advanced-features/advanced-lab/src/lib.rs` | macro_rules!、derive、属性宏与函数式过程宏由本地 d4-proc-macros crate 实现；结构体/枚举展开测试通过 |
| 20.1 | 单线程 Web Server | 部分 | `course/SYSTEMS_PATH.md` | ServiceDesk 同步 HTTP 服务器，不完全同于原书项目 |
| 20.2 | 多线程 Web Server | 缺口 | `—` | BatchMeter 线程与 C2 Tokio 不等于手写线程池 |
| 20.3 | 优雅关闭 | 部分 | `course/ADVANCED_C2.md` | Axum 优雅关闭已有，缺原书线程池收尾机制 |
| A | 关键字 | 缺口 | `—` | 创建按 Edition 2018/2021 区分的离线索引 |
| B | 运算符与符号 | 部分 | `course/studio/unitlab.md` | 示例覆盖部分，缺完整参考索引 |
| C | 可派生 Trait | 部分 | `course/ADVANCED_B.md` | Serde derive 已实作，标准库 derive 清单待补 |
| D | 实用开发工具 | 部分 | `course/ADVANCED_B.md` | cargo fmt/clippy/test 已覆盖，rustdoc/IDE 辅助待补 |
| E | Edition 与版本 | 部分 | `course/BOOK_GUIDE.md` | 2018、2021、2024 差异导航已有，需系统化 |
| F | 翻译版本 | 资料索引 | `course/BOOK_GUIDE.md` | 不要求编码练习，明确旧译本与官方书对应 |
| G | Rust 开发流程与 Nightly | 缺口 | `—` | 补 Stable/Beta/Nightly 及不稳定特性边界 |

## 不纳入覆盖率的情况

- C1/C2 的 Tokio 与 Axum 是有价值的增量，但不能替代原书第 20 章的同步线程池设计；SQLite FFI 中出现 `unsafe` 也不能证明已学会 Unsafe Rust。
- 编译测试、自动化门禁验证可执行行为，不能自动证明学员理解所有权、接口设计与编译错误；每阶段还应保留个人输入、新测试、解释性答题与试用记录。
- 旧 Rust Book 的 API、Cargo 行为可能随工具链变化；运行命令以 `Cargo.lock`、`rust-version` 和官方 Cargo Book 对照验证，不能只凭中文版描述更新当前工程。

## 里程碑签核表

| 里程碑 | 交付定义 | 当前情况 |
|---|---|---|
| D0 | 原书全目录逐节证据和风险映射 | 本表已创建，待人工复核 |
| D1 | 5 节 Cargo 讲义 + 隔离 Workspace + Cargo 外部子命令 + 真 CI | 已提交实验工程，持续集成待验证 |
| D2 | 智能指针 6 节 + RAII/Deref/Weak 实验 | 待实施 |
| D3 | 面向对象与模式匹配 6 节 | 待实施 |
| D4 | 第 19 章 5 节安全隔离实验 | 独立 Workspace 已实现；17/17 CI Jobs PASS，候选包已组装 |
| D5 | 第 20 章 3 节线程池与关闭 | 待实施 |
| D6 | 附录 A–G + 基础零散缺口 | 待实施 |
