## 进阶 B 工程边界（0.19.0-dev.2）

`projects/advanced-b/reference` 为独立可运行三 crate Workspace；`learner` 是同接口未实现业务源文件；`labquest advanced-b` 生成新工程并使用平台固定测试合同编译验证，只在显式 check 时执行学员代码。学员可以自定义业务内部模块，但评测不会把学员自带 tests 或 Cargo.toml 当成验收权威。业务库不依赖 UI，CLI 和 Markdown 报告互不依赖。当前仅覆盖本地 TSV，不保证并发写入和断电持久性。

第三方生态扩展（B4）仍缺实测，异步网络（C）亦未完成。本版不改变正式发布支持边界及冻结 RC 资格。

# 0.19.0-dev.1 进阶A变更

基础结课后转入机制与架构设计：复用TermGarden的任务ID和持久化，不用新软件数量代替能力增长。共享编辑器是同线程Rc/RefCell，视图只弱持有；跨线程采用拥有型快照，不共享可变会话。操作草稿+版本校验处理回调重入；撤销有界。普通CLI入口不变。

学员新工程需要实现整个公开接口，不再三处函数填空；固定契约只验证行为，设计评审、个人回归与自己的数据另行交付。B/C未实现。参考实现由平台提供，学员包不含进阶答案，旧core是已学基础设施。

---

# 0.18.0-dev.6 设计增量

本轮核心流程：只读任务卡 → 可选逐层提示 → 修改自己的单个函数 → 明确执行检查 → 看到编译/功能失败类型 → 原地重试 → 使用自己的输入产出成果。

下一步由历史尝试范围建议，而非授权；检查始终重新编译并累计执行，不接受旧报告代替当前行为。对已修改源码，状态显示历史过期，但仍可建议下一个学习目标。编译失败时保留尝试范围，功能失败时定位第一失败步；三步完成后提示产物任务而非继续改已完成函数。普通教学导航不覆盖源码，不自动创建下一款，不新增永久“掌握”标签。

---

# 0.17.0-rc.1 发布收口设计

本轮不更改既有应用和课程的业务范围。四类学员评测入口统一使用限时、限量、关闭 stdin 的文件捕获，避免等待退出再读 pipe 的阻塞风险。编译子任务使用独立 target、固定并行度并剥离父级交叉编译/作业服务器配置。

评测限制不等于恶意代码隔离：输出量以采样检查为准，非硬磁盘配额；只终止直接子进程。启动失败、超时、超量输出均不会颁发成功记录。

发布工具保存版本、代码指纹、原始日志哈希、逐步退出码与测试数量。超时、0 测试、增加 ignored、旧源码或 browser mock 不得被统计为合格。
候选包名与 RC/Stable 资格分离，规则见 [RELEASE_POLICY](RELEASE_POLICY.md)。

---
以下保留各应用设计范围。

# Rust AppLab 0.15.0 · 设计边界

## 教学原则

真实软件和真实产物先行。完整参考源码供教师与维护者回归，学员使用独立工程；LabQuest 以复制的源码和内置测试执行第 1→3 步验收。只有功能行为证据，没有真人能力认证。第一步不需要完成后两步。

## 网络双作品的分工

### NetFetch（标准库同步客户端）

`parse_url` 无 I/O、只识别本机 URL；`parse_response` 从字节解析受限 HTTP/1.1 200 响应并验证 Content-Length；`request` 使用 `TcpStream` 发 GET；`download` 用 `create_new` 拒绝覆盖并写文件；`batch` 在暂存目录完成批量下载后提交。无 TLS、chunked、重定向、公网主机或断点续传。本机 2 MiB 最大正文和 3 秒网络超时。标准库网络行为是教学用受限子集，不是通用 HTTP 客户端实现。

### ServiceDesk（标准库同步服务端）

`validate_title`、`apply`、`render_json` 是学员真正实现的业务函数；`persist/load` 提供受限的追加日志存储；`http.rs` 实现本机 HTTP API 外壳，`samples/index.html` 是浏览器客户端。只接受本机 Host 的有限路由，写入须自定义请求头和同源 Origin（若浏览器发送）；网页用 textContent 显示用户标题。服务串行，每个连接有超时和正文上限；只支持一个写入者，不提供公网身份认证、耐久事务保证或并发服务承诺。

本轮不存在 Tokio/Reqwest/Axum 的真实构建产物。原生态依赖准备器保留、独立于基础 Workspace。学员学习上述两个标准库基础项目不要求联网或 Python。

## 真实验收

每个应用含两项 Step1、两项 Step2、若干 Step3 行为测试。LabQuest 的 Studio check 使用内置测试、真实 CLI 和临时目录；学员修改自己的 `tests` 不能让正式检查器通过。跨应用 `scripts/smoke_network.py` 实际启动 ServiceDesk、从 HTTP 创建/关闭工单、用 NetFetch 读取 JSON/HTML、检查同步存储，覆盖失败路径。

## 平台与安全

Windows 学员入口使用 `CONTINUE_WINDOWS.cmd`，新增 10/11 选项；初学者默认 UnitLab 不变。当前 Windows / macOS 原生数据待复验，Linux x86_64 Rust 1.99.1 为当前可执行环境。评测会运行用户代码，不是安全沙箱。避免将敏感文件或公网服务器当实验对象。


## 0.16.0：离线异步语言桥接，不冒充生产运行时

新增 `apps/asyncboard`：输入验证、真实 async/await、Future/Poll/Waker、单线程执行器和单个定时器线程，最终输出可供用户查看的 TSV 报告。与 BatchMeter（多线程阻塞）及 NetFetch（同步 TCP）形成对照。学员分三步修改 `lib.rs`，`runtime.rs` 作为已提供的底层设施，读懂之后可作为进阶练习。TimerDriver 不负责网络 readiness、超时取消、TLS 或 HTTP 协议解析；成熟 Tokio/Reqwest/Axum 仍需真实下载、锁定和构建依赖后才能进入教学基线。


### 0.18.0-dev.6 · 第四段衔接

BatchMeter → MiniKV → NetFetch → ServiceDesk 的系统应用路线继续保留；StoreDesk、AsyncBoard 本轮补齐 6 张微任务卡、18 条分层提示，总覆盖为 12 款/36 张/108 条。完整路径见 [course/SYSTEMS_PATH.md](course/SYSTEMS_PATH.md)。新建工程有 TASK_CARDS.md，旧工程可直接使用新版 learn lesson/hint，无需覆盖。StoreDesk/AsyncBoard 的引导已补齐，事务与等待实验见 RELIABILITY_PATH.html；成熟异步网络、桌面和 WASM 仍未完成。

本轮具体入口：[第四段：事务与等待](RELIABILITY_PATH.html)。新手不跳级；StoreDesk 先修 LedgerLite，AsyncBoard 先理解 BatchMeter。后续由 TaskPocket/PageMint 检验独立开发，不把 36 个 TODO 完成自动认定为掌握整个 Rust 生态。


## dev.6 独立作品教学

两个独立作品增加累计行为检查点而不提供核心解答。分步通过记录与最终 APP_ASSESSMENT 分离；导航以受限源码指纹提示过期，不发学习能力认证。生成器提供自写测试作业与交付说明模板。自动化仅验证系统行为；真人设计能力和课堂效果仍需实际观察。
