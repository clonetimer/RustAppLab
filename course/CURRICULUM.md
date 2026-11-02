## 进阶 B（0.19.0-dev.2）：业务库与多程序复用

概念阅读：[Rust Book 对照学习](BOOK_GUIDE.md)；`learn reading <工程路径>` 支持 FirstCard、十二款 Studio、两款独立作品、进阶 A/B。

- B1 数据契约：使用独立的 `taskcore` 库解析、验证并汇总 TSV 任务。
- B2 业务逻辑：可测试的增改接口，失败不修改内存中的原记录。
- B3 Cargo Workspace：两个程序共享同一个业务库，真实运行 CLI 与 Markdown 报告，保留个人测试和交付清单。
- B4 第三方 Serde 依赖管理：独立 json-export Workspace、固定 lock、显式依赖准备、DTO/Serialize derive、只读 JSON 导出及四阶段累计检查；当前源码跨平台结果见 CI。

参见 [进阶 B 讲义](ADVANCED_B.md)，启动入口 `START_ADVANCED_B_WINDOWS.cmd`。

# Rust AppLab · 应用驱动的完整课程路线（0.19.0-dev.1）

## 基础结课后的共同进阶主线

用户已反馈基础学员通过。新增 [进阶 A](ADVANCED_A.md)：复用 TermGarden 练习共享所有权、内部可变性、弱引用、闭包调用约束、借用接口、泛型/trait对象以及Send/Sync。交付参考实现和16号顶层独立学员工程；不是第21款领域软件。

B1–B4 覆盖库工程与真实 Serde 依赖生态；C 成熟异步网络仍待完成。GUI/WASM属于方向选修。

## 背景与纠偏

用户希望学会 Rust 软件开发，而不只是读几个工具的源码。0.1–0.11.1 的主要作品集中于文件、日志、备份与自动评测；第一张任务卡过后缺乏由浅入深的小作品。后期版本大量投入可靠性边界，不能替代语言主干和应用领域的扩展。0.12.0 转为“阶段能力 → 不同作品 → 独立变体”的课程建设。

学习产出以软件为单位，评测器 LabQuest 不计入学生作品数，第一张任务卡作为热身也不计入 20 款主线作品。**下表有 15 款已有原定范围的可执行教学材料，另有 NetFetch/ServiceDesk **本机同步基础版**（异步目标尚未完成），3 款规划项；没有声称 20 款全部完成。**已有材料中 TaskPocket、PageMint 为独立开发合同/骨架/评测/教师夹具，不是另一套已完成的 Workspace 产品。

## 主线地图：20 款作品，而非 20 个目录

| 顺序 | 软件与真实用途 | 学习主干 | 当前交付状态 |
|---|---|---|---|
| 0（热身） | FirstCard 今日任务卡 | 工具安装、参数、变量、格式化 | 已有；0.11.1 修复获用户 Windows 通过反馈 |
| 1 | UnitLab 单位换算器 | 函数、表达式、f64、enum、match、Result | **0.12.0 新增：参考软件 + 三步学员工程** |
| 2 | QuizDeck 终端题库与错题清单 | String、struct、Vec、切片、可变借用、文件解析 | **0.12.0 新增：参考软件 + 三步学员工程** |
| 3 | TextScope 文本统计与搜索 | 所有权、&str、显式生命周期、闭包、HashMap、迭代器 | **0.12.0 新增：参考软件 + 三步学员工程** |
| 4 | LedgerLite 本地生活账本 | 定点数、Option/Result、错误传播、持久化、BTreeMap | **0.12.0 新增：参考软件 + 三步学员工程** |
| 5 | FileQuest 文件雷达 | 路径、目录、模块、报告、集成测试 | 已有 00–07 课程与应用 |
| 6 | LogLens 日志分析器 | 流式读取、枚举、解析策略、指标与差异 | 已有 08–11 课程与应用 |
| 7 | BadgeForge SVG 作品卡 | 模块、trait、泛型约束、格式化、输出安全 | **0.12.0 新增：参考软件 + 三步学员工程** |
| 8 | TaskPocket 独立任务管理器 | 自主划分模块、CLI 合同、状态持久化 | 已有独立工程生成器、评测与教师夹具 |
| 9 | PageMint 离线笔记网页 | 多文件转换、排序、HTML、产品交付 | 已有独立工程生成器、评测与教师夹具 |
| 10 | BatchMeter 多文件并行统计台 | 线程、move 闭包、作用域借用、channel、收集与错误 | **0.12.0 新增：参考软件 + 三步学员工程** |
| 11 | LocalHub 本地网页工作台 | TCP/HTTP 基础、JSON、前后端契约 | 已有同步标准库版本；**不算 async 课程** |
| 12 | SafeSync 备份与恢复 | 二进制 I/O、哈希、RAII、异常恢复与工程边界 | 已有应用；先学16–19，20–23作进阶专题 |
| 13 | TermGarden 终端任务花园 | 按行事件、状态机、Box<dyn Renderer>、可测试界面与保存 | **0.13.0新增：参考软件 + 三步工程；非raw-mode TUI** |
| 14 | StoreDesk SQLite 物品库存台 | 参数化SQL、事务/回滚、流水、数据层借用、版本检查 | **0.13.0新增：系统SQLite + 三步工程；未教自动迁移/成熟crate** |
| 15 | NetFetch HTTP 下载器 / 异步下载队列 | 回环 TCP/HTTP、响应边界；目标还包括 async/await、任务与取消 | **0.16.0 完成同步基础版+三步练习；异步/TLS 未实现** |
| 16 | ServiceDesk 本地工单网页 / Web 服务 | 同步 HTTP API、工单状态与持久化；目标还包括成熟框架及异步并发 | **0.16.0 完成同步基础版+三步练习；Axum 异步未实现** |
| 17 | ChatDock 局域网聊天室 | 异步消息、广播、背压、断连与优雅退出 | **规划，未实现** |
| 18 | DeskNotes 桌面笔记应用 | GUI 状态、事件、后台工作、平台打包 | **规划，未实现** |
| 19 | WasmBoard 浏览器数据看板 | WASM、Rust/JS 边界、序列化、浏览器构建 | **规划，未实现** |
| 20 | MiniKV 本地配置与书签仓库（综合基础版） | 自定义Error、Journal trait、泛型、借用Iterator、Arc/Mutex、日志重放 | **0.14.0新增三步工程；尚无事务、压缩、性能优化及物理故障保证** |

LabQuest CASE 01 穿插在第 12 项之后，把文件、日志、Rust 修改、验证与备份组织成综合交付；不把教学平台自身反复升级当作又完成一款学生作品。

## 对不同基础的路线

零基础先走 0→1→2→3，不要求先读大型工具源码；三步学员工程提供 CLI 外壳，学员每次只实现一个小函数。掌握函数与数据后进入 4→5→6→7，学习如何组织和验证真正的输入输出。8→9 是独立开发，不再给出主要业务逻辑。10→12 引入并发、网络和可靠性，不和入门绑在一起。

已有其它编程经验者可以先用 UnitLab、TextScope 做诊断练习，之后转入独立作品；不得以跳过简单项目为由跳过 Rust 借用、生命周期和错误模型。教师可阅读参考应用，但给学员分发无答案 learner-kit；学员也可在完成自己实现后比对参考版，记录差异，不要求死记唯一答案。

## 知识覆盖：代码里出现，不等于已经教会

| 能力 | 当前实际载体 | 尚需补充 |
|---|---|---|
| 表达式、函数、分支、循环、枚举 | UnitLab / QuizDeck 三步讲义与源码任务 | 增加真人新手卡点反馈 |
| 所有权、借用、String / &str | QuizDeck 的拥有数据、TextScope 原文引用 | 可变借用冲突的更系统练习 |
| 生命周期 | TextScope search 的显式 `'a` 与引用测试 | 多输入生命周期、结构体借用进阶 |
| trait、泛型、模块 | BadgeForge Element + document<T>、lib/main分层 | TermGarden已提供运行时Box<dyn Renderer>实例；复杂泛型/对象安全深入仍待补 |
| 集合、闭包、迭代器 | TextScope / LedgerLite / BatchMeter | MiniKV 已有借用型 Iterator 与组合；性能分析待补 |
| Option、Result、错误传播 | 溢出检查、解析、文件、线程错误 | MiniKV 已有自定义错误及 std::error::Error；第三方错误库待网络阶段 |
| 智能指针/共享状态 | TermGarden 的 Box/dyn；MiniKV 的 Arc/Mutex 与实际四线程写入实验 | Rc/RefCell 及更复杂同步原语仍待补，不能因依赖库用过就标完成 |
| 并发 | BatchMeter scoped threads、mpsc、join | 异步取消/背压/同步原语仍未完成 |
| 网络与 Web | LocalHub 同步 TCP/HTTP 基础 | NetFetch/ServiceDesk 本机同步基础版已交付；async HTTP 客户端/服务端/真实框架生态未完成 |
| 数据与界面 | TSV、HTML、SVG 真实产物 | SQLite SQL/事务及按行终端界面已有；raw-mode TUI、GUI、WASM 未完成 |
| 工程交付 | Cargo、测试、CLI 回归、发布脚本、源码包 | Windows 全量、macOS、MSRV、生态依赖缓存继续验证 |

宏使用已出现（println!/format!），不等于讲完宏编写。unsafe、FFI、嵌入式、过程宏作为后续选修，不为追求全覆盖在基础阶段强行引入。可增加受 MiniMCU/MiniMPU 启发的纯软件虚拟机选修，但不能取代常见软件开发主线。

## 后续批次的准入标准

0.13已交付TermGarden与StoreDesk的限定范围，后续不能把系统SQLite适配器当作成熟crate课程，也不能把按行输入当作完整TUI。0.16.0 以可验证的本机同步基础版建立对照；后续必须转向成熟异步与 Web 生态，不应把现有受限手工 HTTP 服务器无限扩建。再进入 ChatDock、DeskNotes、WasmBoard 与 MiniKV。

基础阶段保持无第三方依赖，是为了离线和安装简单；**进阶阶段不得继续把零依赖当作硬约束**。真实教程应教会选择、锁定、升级和离线缓存依赖。新增外部 crate 前须验证 Windows 构建要求、许可证、维护状态与课程可复现性，版本以实施时官方资料核实。

每款作品只有同时具备“可运行软件或完整独立工程、需求与样例、分步讲义、修改位置、分级提示、正常/异常测试、自己的数据交付、明确使用边界”，才可标为已交付。空目录、只列技术栈或一个 README 不计作品。

## 学习验收，而非只有软件验收

每款作品至少完成三件事：解释一个关键 Rust 决策；用未见过的输入验证；独立完成一个小变体并补测试。维护者记录“自行完成/提示后完成/参考实现后复做”，不伪造学习时长、通过率或满意度。当前没有真实学员成效样本可供统计，自动测试只证实材料与程序的行为。

参考知识地图：Rust 官方学习入口 https://rust-lang.org/learn/；The Rust Programming Language https://doc.rust-lang.org/book/；Rust by Example https://doc.rust-lang.org/rust-by-example/。上述资料用于校准覆盖范围，本项目以自己的应用教学为主线。


## 0.13.0 新增的能力证据和未完成项

TermGarden新增事件→状态→渲染模型，实际用Box<dyn Renderer>在两种界面间选择；学员必须完成解析、状态变化和纯渲染，再用自己的任务文件交付。StoreDesk新增参数化插入、跨进程SQLite文件、库存/流水原子批次、未知数据库及较新schema拒绝；学员必须独立完成验证、写入和事务三个函数。

本轮不能取代成熟crate学习，也未实现自动schema升级。实施时无法获取crate发行包，选择可离线构建的标准库交互与系统SQLite适配器，而非宣称未编译依赖可用。生态依赖、raw-mode TUI与迁移课程继续排入后续，不删掉目标。

## 0.14.0：综合能力先交付，异步主线保留

在本轮环境未取得第三方发行包的情况下，先完成原路线中的 MiniKV 基础版（可在 BatchMeter 后作为综合选修，不要求按序号20才学习）。没有将其宣称为完整生产级存储引擎。9款 Studio / 27个里程碑，15/20款主线有材料；仍保留原32关。

NetFetch、ServiceDesk、ChatDock、DeskNotes、WasmBoard 五款仍未完成。新增 ecosystem 依赖准备器提供独立 Workspace、锁文件/vendor生成和本机网络探针验证入口；它不计作第16款作品。当前没有真实联网编译通过证据，详见 NETWORK_PREPARATION.md。基础课程仍不要求下载crate。


## 0.16.0：异步知识的先修桥梁 AsyncBoard（不计入原二十款应用目标）

在同步 NetFetch/ServiceDesk 和将来的 Tokio 网络作品之间，新增 **AsyncBoard** 离线提醒工作台。它有完整程序、三阶段无答案工程、中文解释、Waker 驱动示例和真实结果 TSV。这里要理解：`async fn` 返回惰性的 Future、`.await` 是让出执行权、执行器收到唤醒再 poll；异步任务并不自动等于线程，并发等待不等于 CPU 并行。

本程序是教学型定时器执行器，**没有非阻塞 Socket、TCP I/O reactor、TLS，也不是 Tokio 网络工程**。Tokio/Reqwest/Axum 仍是 P0，须取得依赖和可复现编译证据后才能算完成。原 20 款主线作品仍维持 17 款已有各自限定范围的材料，AsyncBoard 是额外的过渡产品；计数需按“完整/基础版/规划”区分。


## 0.18.0-dev.3：第二段初学者微任务及总览

现有 UnitLab、QuizDeck、TextScope、LedgerLite、BadgeForge、TermGarden 六款作品共 18 张微任务卡、54 条递进提示。请按 `FOUNDATION_PATH.md`、`FOUNDATION_NEXT_STEPS.md` 学习，使用 `learn overview <父目录>` 区分未检查、历史通过、源码改变后报告过期。上述记录不替代真实测试和真人教学观察。本阶段没有增加软件项目，依然保留明确的同步网络/异步网络课程边界。


### 0.18.0-dev.5 · 第四段衔接

BatchMeter → MiniKV → NetFetch → ServiceDesk 的系统应用路线继续保留；StoreDesk、AsyncBoard 本轮补齐 6 张微任务卡、18 条分层提示，总覆盖为 12 款/36 张/108 条。完整路径见 [SYSTEMS_PATH.md](SYSTEMS_PATH.md)。新建工程有 TASK_CARDS.md，旧工程可直接使用新版 learn lesson/hint，无需覆盖。StoreDesk/AsyncBoard 的引导已补齐，事务与等待实验见 ../RELIABILITY_PATH.html；成熟异步网络、桌面和 WASM 仍未完成。

本轮具体入口：[第四段：事务与等待](../RELIABILITY_PATH.html)。新手不跳级；StoreDesk 先修 LedgerLite，AsyncBoard 先理解 BatchMeter。后续由 TaskPocket/PageMint 检验独立开发，不把 36 个 TODO 完成自动认定为掌握整个 Rust 生态。


### 独立开发引导（0.18.0-dev.6）

TaskPocket 与 PageMint 新增设计、三个累计行为检查点、自己编写的测试和最终交付指导，见 [第五段](INDEPENDENT_PATH.md)。现有十二款 Studio 的36张微任务卡/108条提示保留；独立作品检查点不冒充新增软件或又六个固定函数练习。
