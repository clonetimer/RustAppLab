# D0 · Rust Book 中文版逐节覆盖审计（基线）

- **参考目录**：[rust.bootcss.com](https://rust.bootcss.com/)，Rust 1.37 / Edition 2018 书系，第 1–20 章、附录 A–G；不是当前 Edition 2024 Rust Book 的同一章节编号。
- **源码基准**：`clonetimer/RustAppLab` main `fed4a2d37a72de88dfb353474544efde9bdd537a`（2026-11-17 审计时）。
- **证据规则**：表格中“已有实作”只表示可以定位到相关教学或可执行实例，**不表示**该小节原书所有示例、知识点、学员练习与测试均已完整覆盖；“部分”表示只有关联知识或实现；“缺口”明确有需要补的专项内容；“开发中”只证明开发分支已有材料、仍须 CI。
- **验收判据**：每项最终需要有教学目标、版本适配、学员实验或合理的知识索引、可重现检查及用户体验验证。官方原书代码无需照抄，允许使用现有真实作品替换；不能用出现某个关键词冒充教学覆盖。
- **当前统计**：共 **81** 项审计条目，其中 已有实作 65、部分 3、缺口 0、开发中 12、资料索引 1。D1–D4 随 PR #6 合并；D5 与 D6-A 随 PR #8 合并，Run 197 的 17 个 Jobs 全部成功（含 Linux/Windows 原生、Rust 1.74 MSRV、源码/学员包与交付验证）。D6-B 的 12 个基础知识缺口已进入独立参考实验开发，待 CI 签核。这是条目盘点数，**不是**知识点覆盖率或学习达标率。
- **推进路线**：D1 Cargo（14.1–14.5）→ D2 智能指针（15.1–15.6）→ D3 对象模型/模式（17–18）→ D4 Unsafe/高级 Trait/宏（19）→ D5 原生多线程 Web Server（20）→ D6-A 附录 A–G → D6-B 基础知识残余缺口；随每阶段修订本表。

| 原书小节 | 主题 | 审计状态 | 现有代码/讲义证据（仓库相对路径） | 后续核查或补充 |
|---|---|---|---|---|
| 1.1 | 安装 Rust 与工具链 | 已有实作 | `README_START_HERE.md` | 系统工具安装由用户完成；无需重复开发 |
| 1.2 | Hello World | 开发中 | `course/BOOK_D6_FOUNDATIONS.md` | 最小 crate、main、println 与 Cargo 运行实验；代码见 `projects/book-d6-foundations/foundation-lab/src/main.rs`，待 CI |
| 1.3 | Hello Cargo | 已有实作 | `course/BEGINNER_WINDOWS_GUIDE.md` | 补充 Cargo 术语复习即可 |
| 2 | 猜数字游戏教程 | 开发中 | `course/BOOK_D6_FOUNDATIONS.md` | 输入、解析、比较、循环、无效输入与退出；可运行 guess 子命令，待 CI |
| 3.1 | 变量与可变性 | 已有实作 | `course/FOUNDATION_PATH.md` | 已有任务卡 |
| 3.2 | 数据类型 | 已有实作 | `course/studio/unitlab.md` | 需补数组/元组专题验证 |
| 3.3 | 函数 | 已有实作 | `course/studio/unitlab.md` | 现有公开合同测试 |
| 3.4 | 注释 | 开发中 | `course/BOOK_D6_FOUNDATIONS.md` | 行/块注释、项文档、crate 文档与 doctest，待 CI |
| 3.5 | 控制流 | 已有实作 | `course/studio/quizdeck.md` | 继续复用分支与循环 |
| 4.1 | 所有权 | 已有实作 | `course/studio/textscope.md` | 含移动错误实验 |
| 4.2 | 引用与借用 | 已有实作 | `course/RUST_ERROR_LABS.md` | 含借用编译错误实验 |
| 4.3 | 切片 | 已有实作 | `course/studio/textscope.md` | UTF-8 与字符串切片 |
| 5.1 | 定义结构体 | 已有实作 | `course/studio/quizdeck.md` | 业务数据结构 |
| 5.2 | 结构体示例程序 | 开发中 | `course/BOOK_D6_FOUNDATIONS.md` | Rectangle 结构体、字段、构造与面积，待 CI |
| 5.3 | 方法语法 | 开发中 | `course/BOOK_D6_FOUNDATIONS.md` | impl、self、关联函数与方法调用，待 CI |
| 6.1 | 枚举 | 已有实作 | `course/studio/unitlab.md` | Unit enum |
| 6.2 | match | 已有实作 | `course/studio/unitlab.md` | 分支转换 |
| 6.3 | if let | 开发中 | `course/BOOK_D6_FOUNDATIONS.md` | Option 分支的 if let 与 match 对照，待 CI |
| 7.1 | 包和 crate | 已有实作 | `course/ADVANCED_B.md` | B1–B3 多 crate 工程 |
| 7.2 | 模块作用域与私有性 | 已有实作 | `course/ADVANCED_B.md` | 业务库与应用入口 |
| 7.3 | 路径 | 已有实作 | `course/ADVANCED_B.md` | 路径依赖 |
| 7.4 | use 关键字 | 已有实作 | `course/studio/badgeforge.md` | 练习中导入接口 |
| 7.5 | 拆分模块文件 | 已有实作 | `course/ADVANCED_B.md` | 多模块/多 crate |
| 8.1 | Vector | 已有实作 | `course/studio/quizdeck.md` | 题库列表 |
| 8.2 | String | 已有实作 | `course/studio/textscope.md` | UTF-8 与所有权 |
| 8.3 | HashMap | 开发中 | `course/BOOK_D6_FOUNDATIONS.md` | HashMap、Entry API、词频统计与确定性输出，待 CI |
| 9.1 | panic! | 开发中 | `course/BOOK_D6_FOUNDATIONS.md` | panic 与程序不变量边界，catch_unwind 仅作行为实验，待 CI |
| 9.2 | Result | 已有实作 | `course/studio/ledgerlite.md` | 错误传播及文件失败 |
| 9.3 | panic 还是返回错误 | 开发中 | `course/BOOK_D6_FOUNDATIONS.md` | 外部输入 Result 与内部不变量 panic 的决策练习，待 CI |
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
| 12.5 | 环境变量 | 开发中 | `course/BOOK_D6_FOUNDATIONS.md` | 环境变量优先级、OsString 与可注入配置路径测试，待 CI |
| 12.6 | 标准错误输出 | 已有实作 | `course/03_error_handling.md` | 非零退出码、stderr |
| 13.1 | 闭包 | 已有实作 | `course/ADVANCED_A.md` | Fn / FnMut / FnOnce |
| 13.2 | 迭代器 | 已有实作 | `course/studio/textscope.md` | 过滤、聚合与排序 |
| 13.3 | 用迭代器改进 I/O 项目 | 开发中 | `course/BOOK_D6_FOUNDATIONS.md` | BufRead、lines、try_fold 与 I/O 错误传播，待 CI |
| 13.4 | 循环与迭代器性能比较 | 开发中 | `course/BOOK_D6_FOUNDATIONS.md` | 同输入等价实现、black_box、release 模式计时与测量纪律，待 CI |
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
| A | 关键字 | 已有实作 | `course/BOOK_D6_APPENDICES.md` | 当前 Strict/Reserved/Weak 完整清单、raw identifiers 与 Edition 差异；11 项测试、2 项 compile-fail 文档测试，Run 197 PASS |
| B | 运算符与符号 | 已有实作 | `course/BOOK_D6_APPENDICES.md` | 常见运算符、符号语境、优先级与 Add 重载实验；Run 197 Linux/Windows/MSRV PASS |
| C | 可派生 Trait | 已有实作 | `course/BOOK_D6_APPENDICES.md` | 九种标准 derive trait 的能力、约束、Debug/Display 区分与 compile-fail 实验；Run 197 Linux/Windows/MSRV PASS |
| D | 实用开发工具 | 部分 | `course/BOOK_D6_APPENDICES.md` | rustfmt、cargo fix/rustfix、Clippy、rust-analyzer、rustdoc 的现代用法与复现命令；工具链使用需结合实际工程进一步练习 |
| E | Edition 与版本 | 部分 | `course/BOOK_D6_APPENDICES.md` | 2015/2018/2021/2024 对照、MSRV 边界与迁移流程；独立迁移实操仍需学员完成 |
| F | 翻译版本 | 资料索引 | `course/BOOK_D6_APPENDICES.md` | 区分旧中文译本基线与当前官方书，提供翻译/版本导航；不要求单独编码练习 |
| G | Rust 开发流程与 Nightly | 部分 | `course/BOOK_D6_APPENDICES.md` | Stable/Beta/Nightly 通道、rustup 命令与不稳定功能隔离规则；Nightly 隔离练习仍需学员完成 |

## 不纳入覆盖率的情况

- C1/C2 的 Tokio 与 Axum 是有价值的增量，但不能替代原书第 20 章的同步线程池设计；SQLite FFI 中出现 `unsafe` 也不能证明已学会 Unsafe Rust。
- 编译测试、自动化门禁验证可执行行为，不能自动证明学员理解所有权、接口设计与编译错误；每阶段还应保留个人输入、新测试、解释性答题与试用记录。
- 旧 Rust Book 的 API、Cargo 行为可能随工具链变化；运行命令以 `Cargo.lock`、`rust-version` 和官方 Cargo Book 对照验证，不能只凭中文版描述更新当前工程。

## 里程碑签核表

| 里程碑 | 交付定义 | 当前情况 |
|---|---|---|
| D0 | 原书全目录逐节证据和风险映射 | 本表已创建，待人工复核 |
| D1 | 5 节 Cargo 讲义 + 隔离 Workspace + Cargo 外部子命令 + 真 CI | PR #6 已合并；D0–D4 验收矩阵 17/17 CI Jobs PASS |
| D2 | 智能指针 6 节 + RAII/Deref/Weak 实验 | PR #6 已合并；D0–D4 验收矩阵 17/17 CI Jobs PASS |
| D3 | 面向对象与模式匹配 6 节 | PR #6 已合并；D0–D4 验收矩阵 17/17 CI Jobs PASS |
| D4 | 第 19 章 5 节安全隔离实验 | D4-A2 17/17 CI Jobs PASS，PR #6 已合并 |
| D5 | 第 20 章 3 节线程池与关闭 | PR #8 已合并；Run 197 的 Linux/Windows/MSRV、交付与浏览器矩阵 17 Jobs 全部成功 |
| D6-A | 附录 A–G | A–C 已有可执行实验并通过 Run 197；D/E/G 为参考讲义，F 为资料索引 |
| D6-B | 12 项基础知识残余缺口 | 独立 Foundations Workspace 与讲义已建立，待 CI 签核 |
