# 2026-11-02 · B4 Serde JSON 导出增补（0.19.0-dev.2）

- 新增独立 `json-export` 适配工作区，真实 serde / serde_json、固定 v3 lock、DTO/Serialize derive、严格标题 feature 和只读 CLI；基础 Workspace 保持纯 path 依赖。
- 新增 `advanced-b add-json/prepare-json`；固定参考源码准备依赖，独立保留日志，不运行学员构建脚本。旧工程显式添加，不覆盖目录。
- `check 4/all` 与 `learn check` 累计到 B4，显式 `3` 保留原合同。Report v3 记录 JSON 指纹，旧 B3 报告不冒充 B4 通过。
- 完整离线门禁显式检查 B4 参考 Debug/Release、默认/strict、Clippy、doctest 和平台 E2E；CI 单独 fetch，再验证 Windows/Linux 与实际 Rust 1.74。当前源码实际平台结果以 CI 为准。
- 主干全绿后创建不可覆盖的轻量 checkpoint 标签；无 Stable 发布。真实异步网络课程 C 尚未交付。

---

# 0.19.0-dev.2 · 进阶 B1–B3（2026-10-31）

- 新增独立多 Package Workspace：TaskCore 业务库、TaskPocket Pro CLI、TaskPocket Report。
- 新增 LabQuest `advanced-b start/next/check/lesson`，三阶段累计验证 B1 数据契约、B2 业务变更、B3 CLI+报告两个真实进程。
- 真实评测使用平台固定合同，学员修改自己测试文件不能绕过。报告只对当前源码指纹有效；学员源文件受文件大小/数量和链接检查约束。
- 支持不依赖 crates.io 的 Cargo Workspace、path dependencies、strict-titles feature、文档与接口测试。
- **B4 Serde/serde_json 第三方依赖未完成**：当前运行环境 DNS 不可用、无 crates 源码缓存，没有伪造外部依赖验证。
- 不修改 0.17.0-rc.2 冻结 RC 基线，本版开发版不继承 RC/MSRV/Windows 资格。支持声明仍仅 Windows x64 MSVC / Linux x86_64，不声明 macOS。

---

# 0.19.0-dev.1 · 进阶A

- 扩展TermGarden工作区：共享句柄/Weak视图、草稿批次编辑/撤销、后台快照报告。
- 新增advanced start/lesson/check/next；兼容learn菜单和进度总览。
- 累计检查使用独立临时工程、固定合同与依赖；初始核心未实现，旧源报告不当新成绩。
- 新增ADVANCED_A.html、官方机制资料、DESIGN_REVIEW.md和START_ADVANCED_WINDOWS.cmd。
- 新增16号顶层学员工程，保留17个主Workspace包；两个参考例程不算新作品。
- 实际测试结果在外部本轮PROGRESS_REPORT与EVIDENCE；不继承旧Windows/MSRV/RC资格。

---

# 0.18.0-dev.6 · 独立开发与个人测试

- TaskPocket / PageMint 新增只读→业务→完整合同累计检查点。第三点保留全部原合同探针，并包含只读回归。
- learn lesson/hint/check/next/overview 支持独立作品，Windows 续学入口无需新依赖。
- 生成 START.html、DEVELOPMENT_PLAN.md、TEST_PLAN.md、RELEASE_CHECKLIST.md、self_check 测试入口。仅入口烟测默认通过，个人回归作业默认 ignored 且待实现。
- 新 `.app-reports` 与最终 APP_ASSESSMENT 区分；源码/测试/配置改变后提示过期。
- 保留旧 app init/check/hint 接口和原冻结 RC，不改十二款 Studio 核心业务。
- 本版本为教学开发版，Windows/MSRV/真人效果不自动继承历史证据。

---

# 0.18.0-dev.5 · 事务与等待的完整分步教学

- StoreDesk、AsyncBoard 新增 6 张微任务卡、18 条提示，十二款 Studio 共 36/108；HTML 与命令行复用同一份内容。
- 新增 RELIABILITY_PATH.html、Markdown、TSV 样例与空白学习记录，区分平台参考演练和学员独立运行。
- 真实 StoreDesk 失败批次回滚、成功批次、同 SKU 连续调整、重复编号保护；独立 Python SQLite 逻辑状态核对。
- AsyncBoard 提供 wake_lab，解释 async 函数体惰性、未唤醒 Pending、真实定时唤醒；说明 wait_ms 是配置而非实测，超时不能打断阻塞 poll。
- 两款新增真实分步 CLI 回归，拒绝“没有整批事务”和“省略 await”的错误实现；不改已有业务接口。
- 不声明 macOS。Windows 当前开发版、MSRV 与真人教学有效性不继承旧 RC 证据。具体实测结果见本版交付报告。

# 0.18.0-dev.4 · 个人工单档案与系统应用教学

- BatchMeter、MiniKV、NetFetch、ServiceDesk 新增 12 张微任务卡、36 条分层提示，总数 30/90，覆盖十款作品。
- 调整学习路线，补齐跨作品 handoff，避免把带基础设施的 MiniKV 误列为从零独立开发。
- 新增 `course/SYSTEMS_PATH.md`、联合作品学习记录模板及真实四应用联合验收脚本。
- 新增四款实际 CLI 学习流程回归：初始失败、单步隔离通过、后步未实现失败、完整参考通过、修改后旧记录过期。
- 不更改 Rust 应用业务实现，不扩大公网/异步/数据库能力，不声明 macOS。独立教学开发版，不继承冻结 RC 资格。
- 实测数字、未验证范围见本轮 PROGRESS_REPORT 与 evidence，不将自动演练等同真人学习效果。

---

# 0.18.0-dev.3 · 从文件作品延伸到交互软件（教学开发版）

- 新增 LedgerLite、BadgeForge、TermGarden 共 9 张微任务卡、27 层循序提示；每步给出真实函数、预测问题、个人数据实验和复盘。不会替学员自动填代码。
- 新增只读 `learn overview <父目录>`；同时识别普通学员工程及 Studio/ 子目录，区分未检查、编译失败、阶段失败、源码改动造成旧报告过期、三步历史通过记录。历史可编辑，不发合格认证。
- Windows `LEARN_NEXT_WINDOWS.cmd` 增加 P 入口查看同级作品，旧学员工程无需重建。
- 新增 `course/FOUNDATION_NEXT_STEPS.md` 与 `course/PROGRESS_OVERVIEW.md`，并加入无答案学员包。
- 不添加新应用，不变更 0.17.0-rc.2 RC_READY 冻结基线。当前教学版需独立验证，详见验证证据。

---

# 0.18.0-dev.2 · 基础学习反馈迭代

新增 learn lesson/hint/check；UnitLab、QuizDeck、TextScope 九张微任务卡与 27 条分层提示。新生成工程附带 TASK_CARDS.md，旧工程直接使用新导航而无需覆盖源码。修复累计报告 2/3 被误指回第 2 步的问题；完整通过后转向自己的数据交付。编译失败解释首条诊断、位置，功能失败指向当前函数；支持带 BOM 的 UTF-16 外部日志。Windows 续学入口增加持续菜单，保留 native stderr/退出码，普通学员不再需要手动抓取编译日志。

正式范围仍是 Windows x64 MSVC 与 Linux x86_64，不声明 macOS。此为开发版，冻结 0.17 RC 基线未变，新版不继承旧资格。

本轮 Linux Rust 1.99.1 全仓 Debug 309 / Release 299 通过，各 15 项原教学挑战跳过；文档测试 2 通过。新增教学路径自动演练不等于真人学习效果验收；新增 Windows PowerShell 菜单尚待原生实测。详细原始日志见随版本 evidence 包。

---

## 0.18.0-dev.1（教学引导迭代；不是 Stable）

- 保持原 0.17.0-rc.2 RC_READY 冻结档案不动，建立独立开发版本。
- 宣布发布支持范围为 Windows x64 MSVC 与 Linux x86_64；不再声明 macOS，也不把它作为 Stable 必要条件。
- LabQuest 新增 `learn route / next / diagnose`：学习路线、已有项目的下一步和编译错误首条解释，均不修改学员源码，也不把旧报告当作当前通过证据。
- 新增 Windows 双击学习导航和两份教学实验文档：当前任务自诊断、真实所有权/借用编译失败对照。
- 旧版 RC 的跨平台工程测试不能自动继承为此开发分支的验收结果；需要当前源码下独立验证。

# Rust AppLab 0.17.0-rc.2 · Windows 浏览器验收路径补齐

- 用户确认 0.17.0-rc.1 在其实际 Windows 环境可用；本记录将其作为用户报告的功能 smoke，不冒充当前候选的全量原生工程证据。
- 新增 `VERIFY_BROWSER_WINDOWS.cmd` 与 `scripts/verify_browser_windows.ps1`，在真实 Windows Chrome/Edge/Playwright Chromium 上直接访问 LocalHub、ServiceDesk，并执行真实表单、搜索、关闭、服务重启恢复和移动视口检查。
- `scripts/browser_e2e.py` 现在自动发现常见 Windows Chrome/Edge 路径；不会修改浏览器策略，也不会在网络导航被管理员阻断时降级成 mock。
- 新增 `scripts/fresh_archive_check.py`，对最终源码 ZIP 做 CRC、路径安全、SOURCE_MANIFEST、全新解压 Debug/Release/文档测试和代码指纹检查，报告绑定 ZIP SHA-256。
- 本轮 rc.2 工程验收 17/17 步骤通过；Debug 284、Release 274、文档测试 2 项通过。最终源码 ZIP 全新解压后再次通过同样的测试门槛。Linux 浏览器仍被 `ERR_BLOCKED_BY_ADMINISTRATOR` 阻断，真实浏览器门槛未通过。Rust 1.74、macOS 和真实学员接受度仍是 Stable 缺口。

---

# 0.17.0-rc.1 · 冻结与验收候选

- 保留现有 17 包、12 Studio、独立作品和所有课程；没有增加软件数量。
- 首次在本环境完成未修改 0.16.0 的单次全仓 Release 复验；历史超时未复现，不武断归因。
- 新增统一输出捕获：同时承接 stdout/stderr，明确超时/输出过量/关闭 stdin；入门编译有等待上限。
- 入门评测不继承外层交叉编译目标；新增大输出和跨目标反例测试。
- Python 原生验收、Windows 单击原生验收与可执行程序打包、真实浏览器 E2E 和严格资格判定工具。
- 保持旧 RC 的真实浏览器门槛。候选命名不等于资格已通过；当前状态见 [本轮记录](verification/RC_QUALIFICATION.md)。
- Windows/macOS/MSRV/真实学员结果未伪造为通过；历史版本证据保留。
- 修正 Windows 验收启动器中的路径转义控制字符，新增字节/路径检查；静态检测不冒充原生运行。
- 当前单次全仓 Debug 284、Release 274、文档 2 项通过；发布工具 13 项通过。15 项保留挑战不计入通过数。

---

# Rust AppLab 版本记录

## 0.16.0 · AsyncBoard 异步基础作品（2026-10-29）

- 新增真正的 async/await 定时提醒程序：非忙轮询的单线程 Future 执行器、单个定时器驱动线程、Waker 唤醒、并发任务完成顺序与 TSV 报告。明确不是 Tokio 网络运行时。
- 完成三阶段 Studio 学员任务（解析校验、TSV 报告、async fn 中等待），有逐步提示、错误实现拒绝和 CLI 交付；Windows 菜单增加 12，但没有 Windows 原生测试结论。
- 当前容器无法解析 crates.io，Tokio、Reqwest、Axum 未取得依赖或锁定编译；不能宣称完成异步网络深化。
- 全 Workspace、Linux Release、学员包和最终打包验收以 `verification/V0_16_0_EVIDENCE.txt` 为准，结果未执行前不提前承诺通过。

---

## 0.15.0 · 本机网络客户端与工单服务（2026-10-29）

- 新增 NetFetch 同步本机 HTTP 下载器，提供真实回环 TCP 下载、HTTP 内容长度检查、批量暂存提交、拒绝覆盖与 2 MiB 上限。
- 新增 ServiceDesk 同步本机工单网页，支持实际浏览器表单、HTTP GET/POST、工单状态、UTF-8 日志持久化和刷新后恢复；边界为单写入者、有限协议、非公网。
- 两款参考程序均加入三步 Studio 学员练习与权威回归，Windows 续学菜单增加 10/11；入门默认保持 UnitLab。
- 新增 scripts/smoke_network.py，真实网络端到端联合 NetFetch/ServiceDesk；其它旧作品与 32 章讲义保留。
- 由于没有取得 Tokio/Reqwest/Axum crate，异步/框架版仍是待办，不能把 0.15.0 同步基础版记为异步课程完成。
- 当前完成情况以 verification/V0_15_0_RESULTS.json 和实际测试日志为准。Windows/macOS 原生结果另待用户反馈。

---

# 0.14.0 · MiniKV 综合应用与生态依赖准备

新增MiniKV参考软件、三步学员骨架和完整讲义：自定义错误、Journal泛型与内存/文件后端、借用型Iterator、跨进程持久化、Arc/Mutex实际共享实验。不是SQLite/Redis替代品，无事务/压缩/断电保证。

Studio九款27里程碑，学员包十二工程，根Workspace十四Package。Windows续学菜单新增9，默认仍是UnitLab。增加编译失败与修正后成功的两项文档测试；评测错误禁用继承的冗长回溯。

新增ecosystem plan/init/prepare及维护者Windows入口，生成独立网络探针、记录失败阶段、保留锁文件并准备vendor。准备流程的单元测试含模拟执行器，不算真实网络验证。当前容器不能获取依赖，NetFetch/ServiceDesk仍未完成；候选版本不是最新或安全审计声明。

保持0.11.1路径修复；用户此前的Windows确认不扩大为本版原生通过。实际测试结果见verification/V0_14_0_RESULTS.json和对应日志。

---

# 0.13.0 · 状态交互与 SQLite 教学版

新增TermGarden与StoreDesk两个参考应用及独立Studio学员工程，八款共24个里程碑；原有32章与六款分步应用保留。新增两篇完整讲义和分层提示，不把底层FFI当作新手前置。

TermGarden：按行事件输入、稳定任务ID、完成/撤销/筛选、纯渲染、Box<dyn Renderer>双界面、任务保存与脚本重放。不是raw-mode TUI，多进程同时写同一文件不支持。

StoreDesk：真实系统SQLite、v1数据库身份与版本检查、参数化插入、库存查询、库存与流水同事务、错误/栈展开回滚。有限原生适配器不是成熟crate课程；自动schema迁移和生产安全/物理故障认证未交付。

Studio：新作品支持文件与样例进入独立工程/临时验收；阶段1不会被未完成阶段2/3阻断；原生SQLite链接失败显示环境提示。Windows续学菜单增加7/8，默认仍为UnitLab，SQLite预检仅选择8时执行。新增Windows数据库/终端冒烟并接入CI；配置存在不等于已通过原生运行。

交付：完整源码、11工程学员包、13程序Linux包、真实数据库/任务示例与验证记录。实际测试计数以本轮verification日志为准。原有0.11.1 Windows用户确认没有扩大为0.13.0全量验证。

---

# 0.12.0 · 多作品分层教学扩展

- 新增 UnitLab、QuizDeck、TextScope、LedgerLite、BadgeForge、BatchMeter 六款参考软件，11-package Workspace。每款有三段核心函数的学员骨架、完整CLI、样例、六项核心测试和两项真实CLI测试。
- 新增 Studio catalog/start/check/lesson/hint：分阶段检查、累计回归、独立临时编译、源码摘要报告、编译/运行超时。原有 beginner/app/CASE 命令兼容。
- 交付六篇有明确编辑位置、解释、预测、错误反馈、自主变体和交付要求的讲义，以及18个里程碑。不是新增18款软件。
- 新增 WORKSHOP.html、CONTINUE_WINDOWS.cmd、Windows续学菜单和原生CI脚本；保留0.11.1修复。用户已报告0.11.1测试通过；新流程不擅自视为原生通过。
- 学员练习包扩展为 FirstCard + 六款Studio + TaskPocket + PageMint，共9个独立工程，不包含完整核心参考答案。
- 建立20款主线作品与语言能力覆盖表：12款已有材料、8款规划，明确async/数据库/GUI/WASM/智能指针等未完部分。
- 本轮具体测试数量与执行结果以 verification/V0_12_0_RESULTS.json 和执行日志为准，不用预估数字充当证据。

---

# Rust AppLab 0.11.1 · Windows 新手目录创建热修复（2026-10-28）

- **来自 Windows 学员的首次真实故障**：在 `START_WINDOWS.cmd` 的第 1 步，LabQuest 编译成功，但 `beginner start my-first-card` 返回“上级文件夹不存在”。此处并非 MSVC、Rust 安装或链接错误。
- **根因修复**：`Path::parent()` 对单层相对路径返回空路径，原实现误以为父目录不存在；现在将空父路径视为当前工作目录 `.`，保留对缺失嵌套父目录的保护。
- **双重修复**：Windows PowerShell 启动器对 `beginner start` 与 `beginner check` 使用已经存在的完整目录路径，以减少用户目录和工作目录歧义。
- **回归**：新增单层相对路径从创建到校验的真实 CLI 测试，以及缺失/存在嵌套相对父目录测试；Windows CI 冒烟脚本也增加了同一相对路径检查，防止仅测试绝对路径再次漏报。
- **Windows 编译体验**：修复 PageMint 仅在 Unix 添加额外测试时的条件变量 `unused_mut` 警告。
- **范围与证据**：Rust 1.99.1 Linux 中真实复现旧版失败，并在修复后用相同命令测试通过；Windows 原生启动器和 PowerShell 仍需 Windows 学员或真实 Windows CI 结果确认。保留 0.11.0 学习工程及学员数据，绝不建议删除已有文件。

---

# Rust AppLab 0.11.0 · 零基础入门与 Windows 学员引导（教学开发版）

- 针对学员明确反馈，不再以新增复杂功能为本轮目标；保留五个 Package、原有 32 关，增加原有课程之前的独立第 0 阶段。
- 新增根目录 `START_HERE.html` 图文页面、`START_WINDOWS.cmd` 启动器和 PowerShell 交互向导；初学者无需先理解 Cargo workspace 或从五款应用中自行选择。
- LabQuest 新增 `beginner start/check/hint`：生成只有一行 TODO 的 FirstCard Rust 小程序，真正编译并用四种输入检查动态输出；初始答案、固定输出、错误语法、无限循环均有明确反馈。
- 通过后生成 `FIRST_WIN.md`（不覆盖已有记录），学员可用 PowerShell 将自己生成的任务卡保存成 TXT。
- 增加 Windows 原生 CI 冒烟脚本供未来验证；**当前 Linux 环境没有 Windows 原生通过证据**。本轮验证记录以真实运行结果为准，继续保持开发版，不宣布 Stable。

---

# Rust AppLab 0.10.0 · 跨领域独立软件实战（教学开发版）

- 新增 PageMint 独立工程生成与真实编译/CLI/HTML 黑盒评测（Unix 9 组、其他平台 8 组）；初始源码保留 TODO，不提供业务实现。
- 添加 `labquest app catalog` 与 `app init/check/hint pagemint`，原 0.9.0 TaskPocket 单参数命令继续兼容。
- PageMint 核心路径：按文件名整理 `.note`、UTF-8 和空行处理、正确 HTML 转义、生成可离线打开的网页、拒绝覆盖和损坏输入。
- 新增第 30、31 关和无教师答案的学员练习包；教师完整版保留可独立验证的参考实现与负向测试。
- **版本资格**：0.10.0 仍非 RC/Stable。Linux 的自动化测试通过不能代替真实学员实测、跨平台原生测试或浏览器网络完整链路。

---

## 0.9.0 · 独立软件开发与迁移验收（2026-10-28）

- **不再只改预置案例**：新增 `labquest app init <新目录>`，创建完全独立的 TaskPocket 学员项目；需求合同、Rust 代码骨架与开发日志齐全，但业务逻辑必须亲手完成。
- **黑盒行为评测**：`app check` 离线编译一次、在多个真实数据库和独立进程中运行九组验收。核对标准输出、数据文件、Unicode、幂等、异常码及出错后字节不变，不要求采用特定算法。
- **学习反馈**：失败项给出定位建议并保存 `APP_ASSESSMENT.md`，同时 `app hint 1/2/3` 提供分级辅助；保留 0.7–0.8 CASE 01 流程不变。
- **正反例实测**：初始不完整程序与固定返回 `added=1` 的实现无法通关；独立的维护者参考实现能够连续两次 9/9 通过；拒绝覆盖学员自写报告。
- **新增课程 28–29**：从需求合同、里程碑到跨进程测试、Release 和开发日志，形成独立应用作品交付；不新增无必要的运行时 crate。
- **风险和范围**：评测器执行用户代码，不是隔离沙箱；超时只终止直接子进程。未具备真实人类学员成效和多平台/MSRV 验收，仍为教学开发版。

# Rust AppLab 发布记录

## 0.8.0 · 真实代码迁移测试与学习反馈（2026-10-28）

- **解决判题漏洞**：0.7.0 仅验证当前配置，学员可用固定的错误统计蒙混过关。新案例的 `test-app` 对当前配置和 150/400/750ms 进行一次 Cargo 编译、四次真实 Rust 程序运行，正确结果必须依赖输入；未通过任何一种场景均不写入新运行证明。
- **防止新手操作挂起**：Rust 项目构建设置 120 秒等待上限，单次程序运行设置 5 秒上限；超时尝试终止直接进程并报告可能的无限循环。它不是恶意代码沙箱。
- **新手导航**：新增只读 `next`，显示实际下一步、Rust 技能关联、推荐命令及逐层提示；`practice/predict` 支持提前预测错误记录数并给出边界解释，不记录成绩。
- **兼容设计**：保留 0.7.0 `Case-v1` 的单输入判题和 `Program-Proof-v1`，新增 `Case-v2` 与 `Program-Proof-v2`，未将旧证明冒充为迁移测试完成。
- **课程交付**：新增第 26–27 关和有明确成功/失败标准的学习流程；仍沿用已有四款产品，未新增外部 crates 或通用教学框架。
- **本机结果**：Rust 1.99.1 Linux x86_64，Debug 84 项、Release 74 项基础测试通过，0 失败，两种配置各 15 项学员挑战跳过；格式、Debug/Release Clippy（禁止警告）与 Release 构建通过；五个 Release 程序完成全新 CASE 01 的实际 6/6 通关。
- **验证状态**：见 `verification/RUST_1_99_1_0_8_0_EVIDENCE.txt`；Linux 1.99.1 本地测通不代表 Windows/macOS 或真实学习者体验通过。未提前发布 Stable。


## 0.7.0 · 案例式学习与交互任务反馈（2026-10-28）

- **回归教学目的**：停止以增加工具数量为主要进度指标。新增 LabQuest 案例工作台，让学员像执行一次完整的软件事故调查一样运用已有产品，而不是逐一阅读孤立章节。
- **新增 CASE 01**：`start/status/board/hint/simulate/test-app/finish`。案例包含独立、可实际编译的 Rust 小项目，两份固定模拟日志、可修改配置、六项自动判定、三层提示、离线 HTML 进度板以及最终 Markdown 调查报告。
- **代码练习不可跳过**：预置小程序刻意不输出 SUMMARY；只有学员修改 `src/main.rs`、运行 Cargo 并满足 13 行输出契约，才会生成运行证据。当前源码树或配置更改后旧证据失效。
- **综合产出**：FileQuest HTML 文件地图、LogLens 事故与修复报告、SafeSync 校验快照、Cargo 真实程序运行记录及 `CASE_REPORT.md`。`status` 不执行用户代码，`test-app` 只在显式调用时运行。
- **教学材料**：第 24–25 关、LabQuest 快速入口、失败反馈与异常状态说明；保持原有 00–23 和 15 项延后挑战不变。
- **Linux 1.99.1 实测**：Debug 基础测试 78 项、Release 68 项、失败 0；两种配置均有 15 项学员挑战默认忽略。严格 rustfmt、Debug/Release Clippy、Release 构建均通过；完整人工案例实现 6/6、5 个文件的 SHA-256 备份校验。
- **限制**：Chromium 无头截图本轮超时，不能计为可视化验收；Windows/macOS 及 Rust 1.74 仍无本轮独立实测。LabQuest test-app 需要本机 Cargo，执行学员代码不是沙箱。继续保持开发版，不标记 RC/Stable。


## 0.6.0 · 源目录一致性与暂存身份分离（2026-10-28）

- **不新增产品**：保留四款独立应用和已有基础课程；SafeSync 升级到 0.6.0，其他 crate 的产品版本号不强制同步上调。
- **复制时校验**：复制每个文件时按 64 KiB 块累计 SHA-256 与字节数，并与扫描时的清单逐文件比较；若源数据已改变，立即中止操作。
- **提交前复核**：暂存完整验证后再次扫描源目录；本轮新增测试在暂存校验后、最终复核前确定性修改源文件，验证错误返回、目标不存在及 `Drop` 清理。仍**不是目录原子快照**。
- **用途隔离**：新暂存采用 `.sfs~B~` / `.sfs~R~` 标记备份与恢复，不再把恢复暂存误识别为可提交备份；另增加格式校验。0.5.0 无类型暂存明确显示 `LEGACY`，只能在核实来源后使用 `recover-legacy`，不会伪造其不可推断的操作身份。
- **异常边界**：新增复制中途注入普通 I/O 错误、内容合法但用途不符的暂存、格式异常暂存、旧版兼容等回归，覆盖文件未覆盖与清理契约。
- **教学扩展**：增加 22、23 两关，强调可观察实验证据、选择正确恢复操作与方案边界，而不是引入无用途的并发框架。
- **证据和限制**：Linux x86_64 Rust 1.99.1 实测；仍需 Windows/macOS、Rust 1.74、LocalHub 浏览器真实网络与物理断电/磁盘耗尽专项证据，继续为教学开发版。


## 0.5.0 · SafeSync 中断恢复与性能基准（2026-10-28）

- **保留四款应用结构**：不引入新的产品、远程服务或第三方 Rust 依赖。
- **真实功能新增**：SafeSync `stages <parent>` 列出暂存目录与 READY / INCOMPLETE / UNSAFE 状态；`recover <stage> <target>` 对同父目录、目标名吻合、校验完整、目标尚不存在的快照进行提交，其他情况拒绝并保留证据。
- **确定性异常注入**：仅 debug 构建可使用 `RUST_APPLAB_SAFESYNC_FAULT`，通过子进程 exit(86) 留下真实阶段状态；同时测试普通 I/O 错误导致的 RAII 临时目录清理。覆盖完整备份提交前中断、复制中断、恢复中断、错误目标、符号链接暂存。
- **跨平台路径校验增强**：限制 Windows 保留设备名与部分不跨平台的特殊字符，避免把 Linux 上合法却难以在 Windows 恢复的名称收入快照。
- **课程与度量**：新增第 20–21 关；可复现的 `bench_safesync.py` 在临时目录生成字节数据并执行真实 Release CLI，由 Python `hashlib` 独立校验，附带环境和时间数据。不设置未经论证的性能硬门槛。
- **发布证据边界**：Linux x86_64 Rust 1.99.1 本机验证；CI 已增加 Rust 1.74.0 Linux MSRV 任务，但未取得 Windows/macOS/MSRV 实际通过记录。`recover` 只处理完整快照，仍无断点续传、断电持久性、目录写入 fsync、敌对并发写入保证。继续为教学开发版。

## 0.4.0 · SafeSync 备份恢复教学开发版（2026-10-28）

- **新增第四款真实应用 SafeSync**：`demo / backup / verify / restore / missions`，递归备份普通文件、保留空目录与二进制内容，逐文件 SHA-256 校验并可恢复到全新目录。
- **安全行为优先**：复制到独立临时目录，复验后通过同父目录 rename 提交；恢复先完整验证备份，不覆盖现有路径；源数据不变时重复备份返回 Unchanged，变化后需新建快照。
- **输入与异常约束**：拒绝符号链接、非普通文件、危险相对路径、额外/缺失目录项和清单损坏；说明不保存权限/时间戳/ACL，不支持可信身份认证、恶意并发变更与断电耐久保证。
- **课程新增 16–19**：按事故恢复→摘要校验→恢复安全→自动化交付组织，三项学员任务以按需运行的 RED→GREEN 测试形式提供。
- **LocalHub 浏览器验证补充**：实际 Rust 本地 HTTP 返回 JSON；Chromium 可执行页面 DOM/JS、搜索过滤和移动端布局检查；但浏览器直接访问本机 HTTP 被执行环境策略拦截，完整浏览器到 HTTP 闭环仍待外部环境复验。
- **验证状态**：Linux x86_64 Rust 1.99.1，结果详见 `verification/RUST_1_99_1_0_4_0_EVIDENCE.txt`；Windows/macOS、MSRV 1.74 及更广泛故障注入尚无通过证据，继续标记开发版。



## 0.3.0 · 本地 Web 三应用教学开发版（2026-10-28）

- **实测解决 0.2.0 的构建验证缺口**：安装用户提供的 Rust 1.99.1 Linux x86_64 离线工具链；实际确认前版 27 项基础测试通过；使用 rustfmt 修复格式不一致，修复 LogLens 的 Clippy needless_borrow。
- **新增 LocalHub**：独立 Cargo package，通过路径依赖复用 FileQuest 和 LogLens 的领域库，提供 `demo/serve/missions`、只读 JSON API、HTML/CSS/JS 本地面板。
- **HTTP 安全边界**：固定回环地址、要求本机 Host、HTTP/1.1 GET、请求头 8192 字节、单连接 3 秒超时、CSP、no-store、无客户端可控扫描路径。
- **真实 HTTP 端到端测试**：TCP 客户端请求响应、错误输入、路由拒绝、CLI 子进程退出、头部上限测试；新增学员 13–15 三项默认忽略的挑战。
- **课程扩展到 00–15**，同时更新三产品目录、版本设计、操作文档、白名单源码包、结构校验和 CI 矩阵。
- **状态**：开发版。Linux 工具链上的真实检查结果记录在 `verification/QUALITY_GATE.md` 和测试证据文件。Windows/macOS、Rust 1.74 MSRV、性能与生产安全尚无通过证据；本地浏览器渲染须独立验收。


## 0.2.0 · 双产品教学开发版（2026-10-28）

### 本轮实质性产出

- 新增**第二款独立 Rust 应用** `apps/loglens`，保持与 FileQuest 并列的 Cargo package，而非把解析日志的代码加入文件扫描器。
- LogLens 包含 `demo / analyze / compare / report / missions`：本地 UTF-8 日志逐行解析、错误级别计数、高频消息、两份日志记录级错误率比较、Markdown 离线报告。
- 增加严格模式 `--strict`、未识别行样例、零有效记录说明、明示统计分母；报告默认拒绝覆盖和 Markdown 文本转义。
- 新增 LogLens 的单元测试、真实文件系统验收、CLI 用户流程测试，以及 **4 项默认忽略的 RED→GREEN 教学挑战**。
- 新增 08–11 四章课程、分级提示、学习记录表；更新 README、产品路线、设计基线、双应用 workspace CI/验证脚本。
- FileQuest 报告 CSS 增加 `tr[hidden]` 的明确样式，避免筛选后行隐藏行为依赖浏览器默认值。
- 安全补强：LogLens 拒绝将报告写回输入日志路径；源码包构建脚本改为明确文件类型/目录白名单，避免混入用户个人文件。

### 限制与状态

- 当前执行容器无 `rustc` / `cargo`，无法实测 Rust 编译、rustfmt、Clippy、cargo test 或运行两款软件。因此 **0.2.0 仅标记为教学开发版；新增功能属于已编写源码，不能声称已通过 Rust 运行验收**。
- LogLens 暂时只识别两种单行格式，不含 JSONL、跨行堆栈、时区换算、真实故障自动归因或大规模日志有界聚合。
- `#[ignore]` 的扩展测试为学员任务，不属于当前已实现功能，也不是基础测试成功的证据。
- 正式发布前执行 `cargo fmt --all` 修整格式，之后运行 `scripts/verify.sh` 或 `scripts/verify.ps1`，并保存真实执行日志；跨平台 CI 完成后再考虑 RC。

## 0.1.0 · 初始教学开发版（2026-10-28）

- 建立 Rust workspace 与第一款 FileQuest 本地文件雷达，包括 `demo`、`scan`、`find`、`report`、`missions`。
- 交付离线 HTML、00–07 八章教程、五项默认忽略的学员扩展挑战和基础测试源码。
- 该版本没有真实 Cargo 验证，未申明 RC 或 Stable 资格。
