# RustAppLab 现代化教学计划（2026-11-25）

面向已经完成基础课的学员，使用新稳定 Rust 解决真实任务；保持经典路线可继续使用。此计划不是 rc.5 发布范围变更，不自动提升 VERSION，不承诺已有学员升级。先走 [D7 固定新版 Book 映射](BOOK_MODERN_COVERAGE_D7.md)，再逐项交付。

## 教学与验收纪律

- 经典：Edition 2021 / MSRV 1.74，C1/C2 固定 1.85.0；现代：独立 Edition 2024 / Rust 1.100.0，后续版本显式评估。
- 每个新概念必须包含中文目标、一个最小成功案例、相关反例或故障、为什么失败、可执行检查、一个改变输入或需求的迁移题。先以真实学员能做什么为目标，避免只展示 API。
- 保留学生源码、固定测试和参考实现的区分；不能以自动通过推导真人掌握。默认不执行不受信任代码，网络课程初期仅回环。
- 每阶段独立 PR、Windows/Linux 证据、源码清单；新增工作区须同时处理源包/学员包、入口和离线依赖；不将维护者脚本当成学员课程。
- 稳定特性以官方 release notes、Edition Guide、版本化文档为准；nightly 只做独立选修，不强制依赖。

## 阶段与可验收交付

| 阶段 | 目标 | 完成条件 | 当前状态 |
|---|---|---|---|
| D7 | 最新 Book 基线 | 固定官方发布快照；87 项唯一 ID；旧 ID 一对一映射；六节异步分开标记；证据路径检查；自动结构测试 | 本 PR 建立基线；新版内容复核仍待做 |
| D8 | Edition 2024 教学环境 | 独立 workspace + rust-toolchain 固定 1.100.0；最小工程、2021/2024 对照、resolver 3、迁移诊断、运行/测试；不改旧 MSRV；学员入口 | [代码与讲义](BOOK_D8_EDITION_2024.md) 已提交，待 CI |
| D9 | 1.75–1.100 稳定特性 | 版本增量审计和优先级；语法、Trait、生命周期捕获、async closure、Never、DropGuard 等代表性练习；按稳定范围逐项验收 | D9-A [九项实际任务](BOOK_D9_STABLE_FEATURES.md) 与 [26 版本索引](MODERN_STABLE_FEATURE_INDEX.md) 已提交待 CI；D9-B [逐版代表性审计及标准库/Cargo 实验](BOOK_D9_STDLIB_CARGO.md) 已提交，待当前 CI；完整 API 清单、Unsafe 等专项不纳入覆盖率 |
| D10 | 异步核心课 | 六节目标逐项关闭；Future、Poll、Waker、Pin/Unpin、Stream、join/select、async trait、取消与阻塞；以已有应用串起概念而非重复项目 | D10-A 六步实验双平台专项通过（Run 45586040166）；D10-B [七步真实 Tokio/Stream 实验与离线交付](BOOK_D10_TOKIO_STREAM.md) 双平台专项及完整 PR CI 通过，#20 已合并（b6f2abec） |
| D11 | 现代开发工具 | IDE 断点、Clippy、Miri、属性测试、Fuzzing、基准与依赖审计；独立运行证据及学员迁移记录 | D11-A [工程质量与工具实验](BOOK_D11_QUALITY.md) 完整 CI 与专项通过，#21 已合并（c470154）；D11-B [Miri/libFuzzer/RustSec 实验](BOOK_D11_SAFETY.md) 专项 5/5 PASS，完整回归和 E1-0 安全修复合并（#23，0e71b443）；真人 IDE 与网页入口待验 |
| E1 | 网络工程 | TLS、认证、数据库、可观测性、错误边界；明确本地教学与公网安全差异 | E1-0 [依赖安全收口](NETWORK_SECURITY_BASELINE.md) 已合并；E1-A [真实 TLS/认证](BOOK_E1_TLS_AUTH.md) 与 E1-B [SQLite 事务与观测](BOOK_E1_SQLITE.md) 双平台专项和完整集成均通过，#24/#25 已合并（main 2f4a53dc） |
| E2 | 桌面 GUI | 一款完整可交互软件，模型/视图/状态分离，跨平台测试 | [SignalDesk](BOOK_E2_GUI.md) 十一套 CI 通过并合并 #26（0491d65），Linux 实际窗口/Windows 控件及离线交付通过；Windows 实机未验 |
| E3 | 嵌入式与实时 | no_std、硬件抽象、仿真与硬件分别验证 | [Embedded Monitor](BOOK_E3_EMBEDDED.md) 首轮专项全通过；真实 ARM/QEMU 与固定锁；合并 E2、修正 Rustfmt 后重验 |
| E4 | 互操作与 WASM | C ABI、安全封装、跨语言所有权；浏览器 WASM 独立小项目 | [SignalBridge](BOOK_E4_INTEROP.md) 候选：真实 C 动态链接、Rust/JS WASM 共享分类逻辑，三步 TODO；待当前专项与完整 CI |
| E5 | 科学计算和 AI | 数值/数据处理、性能测量；可选 AI 应用，准确说明计算后端 | 待评估 |

## 本轮开发经验（D7）

旧站与最新版章号不同，不能简单沿用 81 项；用 Rust 发布 tag 的 Book 子模块固定目录与文档。复用已有证据路径，所有条目保留待核验分类，不把重编号当成教学完成。当前项目的 AsyncBoard 已经有教学 executor，C1/C2 有实际 Tokio 应用；现代章节的 Stream、Pin 和动态 Future 仍需针对性学习任务，而不是重新建同类应用。

参考：[官方 Book](https://doc.rust-lang.org/book/) · [Edition Guide](https://doc.rust-lang.org/edition-guide/rust-2024/index.html) · [发布记录](https://blog.rust-lang.org/releases/) · [1.100.0 公告](https://blog.rust-lang.org/2026/11/12/Rust-1.100.0/)。

D7 CI 复核：Run 45579401311 的 tooling/structure 已通过，Linux 旧 D6 诊断复制实验因 ETXTBSY 失败；原始日志见 [Linux artifact](https://github.com/clonetimer/RustAppLab/actions/runs/45579401311/artifacts/15005702349)。只对 Linux 启动时 raw_os_error 26 有界重试，不重试子进程实际失败、不删除断言；修复后的 CI 必须重新验证。

## 开发经验（D8）

同一 Rust 1.100.0 编译器下比较两种 Edition；同一 shared.rs 消除实现差异。告警筛选器有独立 RED 合同和参考解答，只向学员包交付 TODO；维护者将答案注入隔离副本验证 GREEN。Cargo resolver 2/3 使用本地合成目录源实测而非线上不稳定版本。新增项目必须同时进入 source manifest、source_id 与学员包白名单。新版教学使用单独的 Windows/Linux CI 工作，不修改旧 MSRV 或发布版本。当前仍为 CLI 选修，网页工作台接入和真人试用未完成。

## 开发经验（D9-A）

以 1.100.0 发布公告及官方 API 稳定标签为准；GAT 为 1.65 前置，不算新版增量。九个 TODO 与固定参考合同对照，8 组编译失败/修复及 DropGuard 的 unwind/dismiss 行为单独运行；异步只使用立即完成的 Future，不拿单次 poll 假装通用运行时。系统 Allocator 安全调用替代自造 unsafe 分配器，未稳定 allocator_ext 不纳入。D8 原专项 Run 45580912220 双平台成功，D9-A 必须验证当前提交。26 个版本入口不等于 26 个版本完成审计。旧 MSRV 与发布资产不变，CLI 选修；网页与真人观察另行验收。

## 开发经验（D9-B）

D9-A Run 45581790026、D8 回归 Run 45581790007 均双平台通过。D7 Windows 交付 Run 45580476027 的 StoreDesk 旧第三关 8 秒超时有原始日志（artifact 15005899591），不删除测试或放宽时限；主干待完整 CI。D9-B 逐版查阅 26 个官方发布说明的代表主题；16 个合同覆盖选定 API（17 个 TODO），真实学员 RED/GREEN；Cargo 使用独立 path 依赖、配置 include 和实际 lint 拒绝。冷却验证只用临时回环 sparse registry、自造 crate 及 pubtime，校验 Cargo.lock 与执行输出。使用 cargo run 而非硬编码 target 路径以适配新版构建目录。网页工作台和真人观察仍待后续；旧 MSRV 与发布资产保持不变。

## 集成验收修复（D7–D9-B）

D7 修复后 Run 45580476027 第 2 次尝试完整通过；D8/D9-A 的原生和专项通过但候选汇总 Run 45580912179 / 45581790022 拒绝 Windows launcher source_id：PowerShell 目录白名单落后于 Python。统一新工作区范围，新增逐目录指纹变化、构建缓存忽略及两端集合相等测试。D9-B Run 45582928572 的两平台 reference-debug 因 assert_matches! 未导入失败，参考和学员合同均显式 use std::assert_matches，保持相同合同和测试数量。rc.5 已固定为 4e9857b 的发布资产；后续相同 VERSION 的普通主干提交只有在已有发布为非 draft 的 prerelease 且 tag 是当前提交祖先时创建开发检查点并跳过重复发布，不改任何旧标签和资产。D7–D9-B 收敛到 #18 对 main 的累计集成；不以旧专项结果代替当前完整 CI。

## 开发经验（D10-A）

D7–D9 集成修复已在 #18 单独验收，D10-A 不取消其当前 CI。用标准库 Wake + 容量 1 通知通道提供最小单根执行器，协作式截止时间不伪称抢占；测试真正观察首次 Pending、真实跨线程唤醒、最新 Waker 替换、无忙轮询、完成 Future 不再 poll、race loser drop、Pin 与异构 Future 类型。学员六个 TODO 与固定 runtime/合同一致；4 组真实诊断/修复独立运行。课程明确不等同 Tokio/Stream/生产任务调度，D10-B 衔接现有 C1/C2。保持 Windows/Python 源指纹项目范围一致，源码包和学员包白名单同步；无新增第三方依赖。网页及真人验收仍待后续。

## 开发经验（D10-B）

复用 C1 已锁定的 Tokio 1.48.0 与 futures-util 0.3.34 依赖图和 registry SHA-256，不以不必要的全新框架或网络应用替代教学。新增锁须 cargo vendor --locked 与空缓存 frozen 独立核验；用虚拟 Tokio 时钟和在途数量、Waker、Drop/JoinError、线程 ID 与通道握手证明行为，不靠机器耗时猜并发。7 个固定合同（9 个 TODO）、真实 Stream 的 Pending/Some/EOF、Send RPITIT 与非 Unpin Stream 诊断补齐第 17 章代表性目标。离线学员包先保存 TODO 原始字节，只对搬移后的副本注入答案，ZIP 哈希不变。D7–D9 #18 的累计完整 CI、D10-A #19 的集成 CI 仍独立收口；所有新工作区同时更新 Python/PowerShell 指纹范围，保持 rc.5 原发布资产不变。网页与真人效果尚待验收。

D10-B 首轮 [Run 45588756708 Linux 原始日志](https://github.com/clonetimer/RustAppLab/actions/runs/45588756708/job/139117061724) 中 vendor、Debug/Release、两个诊断、搬移学员包、七项 RED/GREEN、CLI 均通过，最后仅因 registry 目录存在拒绝交付。修复不删除离线门槛：两个 Cargo home 初始必须为空；在 frozen 模式读取真实 metadata，核验所有解析出的 registry 包 manifest 位于随包 vendor；记录 registry 全目录及文件哈希，仅允许标准 CACHEDIR.TAG 管理标记和空目录，不允许 index/cache/src 数据。修复后必须重跑双平台验证，不能沿用首轮 FAIL 作为离线交付 PASS。

## 开发经验（D11-A）

D10-B 在 fe2493b9 的完整 PR CI 19/19 适用 Job、五套双平台专项均成功，#20 已合并为 b6f2abec。此前 main bbca9f49 的 Windows StoreDesk 第三关真实里程碑 8 秒超时复现（Run 45589200104 artifact 15011300527），不删测试、不放宽限制、不把原因猜成新异步课错误；新主干仍需实际回归。D11-A 复用独立 Edition 2024 工作区；四步教学帧编解码与计数、三个固定性质、三种真实 mutant、rustfmt/Clippy 红绿、GDB 实际断点和 Release 测量。有限生成输入不等于 libFuzzer，不宣称性能胜负；Windows IDE、Miri、libFuzzer 与 RustSec 另验。学员包无答案，源清单和 Python/PowerShell 指纹同步，经典课程及 rc.5 不变。

D11-A 首轮 Run 45605693046 的 Windows/Linux reference-debug 均失败；[Linux 原始日志](https://github.com/clonetimer/RustAppLab/actions/runs/45605693046/artifacts/15018045979) 确认 Edition 2024 的 gen 保留字被误用为局部变量。参考与学员属性测试统一改名 generator，保留全部种子、生成样本、断言和测试数量；必须重新通过当前提交 CI。这也说明 Edition 迁移检查应真实覆盖测试和 examples，不只检查 lib.rs。

## 开发经验（D11-B）

D11-A 修复后同提交完整 CI 与六套专项通过，#21 合并；D10-B 主干回归 Run 45605275350 也通过。D11-B 稳定接口合同与 Linux specialist jobs 分离，固定 nightly-2026-11-17（沿用 D6 日期）、cargo-fuzz 0.13.2、libfuzzer-sys 0.4.13、cargo-audit 0.23.0 及 RustSec DB b9b8a3c2。故意 UB 仅在 Miri 临时解释执行；用实际 libFuzzer 运行、校验变异反例 artifact 与修复重放，锁文件真实生成后保存并保持不变；历史漏洞夹具只解析/审计锁，不构建不运行，指定公告对照与实际项目所有发现分开，禁用额外 yanked 查询。源包/学员包和 Python/PowerShell 指纹同步。未通过当前 CI 不能宣布 specialist 完成，Windows 原生不冒称 Miri/libFuzzer 支持，旧发布及人类效果保持独立。

## 开发经验（E1-0）

D11-B Run 45611450910 五个专项全部通过，审计 artifact 15020451443 发现 C1/C2 的 url 2.5.8 命中 RUSTSEC-2026-0592、C1/C2/D10-B 的 Tokio 1.48.0 命中 unsound 公告 RUSTSEC-2026-0386。历史 regex 夹具红绿正确、基础根锁/D11-A 无公告；不据此推断全部程序安全。E1 先修补依赖，选 Tokio 1.53.3 LTS（上游 MSRV 1.71）及 URL 2.5.9，按实际 crates.io 索引校验值同步 tokio-macros 2.7.0 和 form_urlencoded 1.2.3；保持 C1/C2 Rust 1.85、D10-B Rust 1.100、所有教学合同和旧 rc.5 原样。旧探针更新 pin 并使用固定 URL 解析器。扩大 RustSec 到六份参考/学员实际锁并要求漏洞与 unsound 均为空才准入，原始 JSON 即使失败仍保留。新锁的实际 Cargo --locked、移植、运行与同快照审计必须通过，未宣称已安全；E1-A 不复用已知告警依赖，保留真人/网页任务。

## 开发经验（E1-A）

#23 在 a43d85f 的八套 CI 全部成功，包括固定数据库六份网络锁漏洞/unsound 准入，累计合入 D11-B 与 E1-0，main 0e71b443；避免单独发布已知风险的中间树。E1-A 直接 path 依赖已完成 C2 路由/单写者，避免重写业务；旧 HTTP Origin 校验原样，HTTPS 只开放显式 API，写入拒绝 Origin。四个 TODO、五个合同；真 rustls/Tokio TLS 1.3、客户端显式信任、错误 hostname/信任和明文拒绝、16 在途握手/2 秒期限、401/403 不改日志、授权后真实重启读取。凭证为随机等长教学值，不宣称完整账号系统。直接版本固定，完整新锁由 CI resolve 一次实际生成并审计，两平台复用哈希；后续冻结通过的锁，避免伪造锁文件。独立学员 ZIP 提供已学 C2 支持代码与 vendor，不含 E1 答案；普通 learner-kit 仅模板。网页、真人效果、E1-B 数据库与可观测性仍待验收。

E1-A Run 45654557009 实际 Cargo resolve/RustSec 通过：101 个 registry 包，0 vulnerability/unsound；[原始锁产物](https://github.com/clonetimer/RustAppLab/actions/runs/45654557009/artifacts/15043180053) SHA-256 44ef895eb533fcf092d77f52c2488513d9dddb1b8faa0ad034d4cf7615d2c3fd。已冻结原始字节，后续只允许 --locked fetch 与复审；两份锁及 artifact 哈希绑定。首轮 Linux 4/5 测试通过，集成在 tests/contracts.rs:99 调用 start_store 前未初始化日志（C2 ensure_store 明确要求先 init），见 artifact 15042116734。增加 C2 的 init_store 公共重导出（仍由原 servicedesk::init 实施，非覆盖），测试/演示显式 init，CLI 增加 init 和不覆盖回归；并新增未初始化启动拒绝断言。不是通过删除测试或放宽存储保护修复。Windows 首轮也失败；当前提交须完整重跑，不提前宣布交付。

## 开发经验（E1-B）

E1-A #24 冻结首轮真实审计锁并修复未初始化的 C2 journal；不删原合同。此前 main 0e71b443 Run 45653594034 Windows launcher debug-tests 再次出现既有 StoreDesk 第三关 8 秒超时（artifact 15042940385）；保留独立风险，不放宽时限或认定新课为根因。E1-B 复用 E1-A guard/TLS（提取不变策略的 protect_router 入口）及 servicedesk::apply 域规则，新增 Rusqlite 0.43.0 bundled SQLite、单阻塞 actor、有界 try_send、真实业务/audit 原子事务、脱敏有界 JSON 访问事件。四个独立 TODO、五个合同，真实 SQL trigger 注入 audit 失败，验证业务回滚；HTTPS 并发与重开持久化、不合法权限不写库。真实 Cargo 从 E1-A 锁解析新图、固定库审计、两平台同锁及空缓存搬移独立学员包；须冻结成功产物后才能交付。仅 E1-B 专用包提供前置答案；全局 learner-kit 不含答案。非生产认证、未实现整体连接限流/幂等/遥测后端、真人与网页未验；不声称关闭了所有 E1 生产场景。E1-B 分支在 #24 之上，不取消其重验。

E1-A #24 九套同提交 CI PASS，已合并 main 5ec8504；#25 原基线 tree 与新 main 相同，转向 main。E1-B 首轮 Run 45656674759 真实锁/审计与 Linux 全部通过，Windows Debug 通过、Release 4/5，step5 的 12 并发中客户端 8 秒超时（artifact 15042239292），不能推断为 DB 根因。冻结首轮原始锁 artifact 15043411603，108 registry 包、SHA-256 20adcb21ee432c82a590fd0da3cbec857adbe572bdad2471698c8e0b07f140db，真实审计 0 vulnerabilities/unsound。测试客户端按完整 HTTP 消息而非必须连接关闭判断，增加连接/握手/发送/头/体阶段诊断，共享原 8 秒绝对期限，保持所有业务断言及并发数，Release 额外重复三次。若仍失败按阶段日志定位，未声称已修复。旧 Windows StoreDesk 风险仍独立。

## 开发经验（E2）

E1-A #24 九套同提交 CI 通过、已合并 main 5ec8504；E1-B #25 仍在独立验收，不以 GUI 开发掩盖 Windows 并发失败。E2 不依赖网络项目，直接从已合并 main 分支，避免继续堆叠。核对 eframe/egui 0.37.0 实际 App::ui API 和 system font fallback，新增真实双语 SignalDesk，四步 TODO、五个合同（包括实际 egui_kittest 点击和中文路径导出/拒绝覆盖）。真实原生 Linux Xvfb/Mesa 窗口键盘事件、三张实际截图和状态标题核验；不宣称 Windows 桌面已实测。初始完整新锁由 CI 生成、RustSec 固定 DB gate、单一 vendor 学员 ZIP 两平台空缓存 replay 后冻结。同步包范围与指纹，不影响旧 MSRV/rc.5。E1-B #25 与 E2 合并前需处理共用文档/manifest 的真实冲突，不丢任何改动。网页与真人效果待验证。

E1-B #25 修复后 Run 45733820495 三个专项及 Run 45733820482 完整回归全部通过，十套 CI 均通过；合并 main 2f4a53dc，保留 12 路并发、8 秒请求期限与三次额外 Release 重放。E2 首轮 Run 45734235763 解析审计通过，Windows Debug/Release、控件和空缓存搬移通过（artifact 15078328936），Ubuntu 参考 Debug 尚未开始合同即缺少 wayland-client.pc（artifact 15077354281）。从原始 artifact 15078287324 冻结 89279 字节 Cargo.lock，SHA-256 0c547514c781df6d8571cb8dac307c66d671e95d22734e1f3bd098b80b81b210，346 registry 包、0 vulnerability/unsound；三项维护状态警告 RUSTSEC-2026-0370/0371/0372（zvariant/zbus_names/zvariant_derive）完整保留，不掩盖，不声称生产级供应链安全。补 libwayland-dev 等真实系统库和 pkg-config 检查；DBus session + Xvfb 原生窗口仍必须实测。将 E2 与已合并 E1-B 的共享源码包/学员包范围、指纹、清单和课程入口显式整合；两份冻结锁哈希绑定，原 E1-B 文件不丢失。重新通过十一个 workflow 后才能合并，E3 不先开新分支。

## 交付回归补强（2026-11-26）

E1-B #25 合并后，main 2f4a53dc 的 [Windows 交付 Run 45749380246 原始产物](https://github.com/clonetimer/RustAppLab/actions/runs/45749380246/artifacts/15085097087) 显示解压源码 Debug 417 passed，Release 的 learning_e2e::storedesk_guidance_rejects_partial_commits 在第 3 步 8 秒超时；旧 runner 删除临时 stdout/stderr，无法确认具体子测试或 I/O 根因。保持 Studio 8 秒期限、原四条 StoreDesk 第三关合同及非零失败策略。修复将直接子进程终止后可获取的 stdout/stderr 末尾各限 4096 字节纳入失败报告，并记录真实 StoreDesk CLI 操作开始、结束及耗时（不打印路径或数据）。learning_e2e 的嵌套 CLI 入口增加测试内互斥，减轻同一测试二进制并行编译和磁盘争用；不改变学员执行逻辑、不跳过任何合同，也不能据此断言已找到超时唯一根因。新增限长日志单元测试及现有超时测试的部分输出断言。需同提交 Windows/Linux 回归验证；E2 当前 CI 保持独立。

## 开发经验（E3）

E2 #26 的冻结锁、Windows/Linux 实际合同及 Linux 原生窗口三张截图在 Run 45750393437 全部通过（artifact 15085692080），完整回归最后 Windows native 尚待收口；不宣称 Windows GUI 实机或真人学习。为不中断耗时中的 E2 当前 CI，E3 从已经合并 E1-B 的独立 main 开发，先独立保留 StoreDesk 限时诊断提交 91f2d346，再提交真实 no_std 核心、embedded-hal 1.0.0 接口、Cortex-M3 SysTick exception 和 WFI、实际 QEMU ARM ELF、4 步 5 TODO 及 5 固定合同。区分模拟传感器/引脚和真实模拟 CPU，故障低电平指令失败明确报 Output 而不是宣称安全。交叉编译 Windows/Linux，QEMU Ubuntu；两份参考/学员 ELF 均真实运行并核验 IRQ/相位/滞环/联锁，不把虚拟 tick 当物理毫秒或 WCET。锁由真实 Cargo 生成、固定 RustSec gate 后冻结。增加 memory.x 源包/学员包白名单与 Python/PowerShell 一致指纹；保持经典根 MSRV/rc.5。当前未实际验证，E2 收口后整合共享打包文件与清单，再完整 CI。

E2 #26 同提交 11 套 CI 均通过并合入 main 0491d65（Run 45750393552/45750393437），保留全部 E1-B 和 GUI 文件。E3 Run 45751986819 首轮实际三项专项 PASS：20 registry 包，0 vulnerability/unsound，bare-metal 0.2.5 的 RUSTSEC-2026-0110 unmaintained 保留；真实 lock artifact 15086121503 SHA-256 db2e90bc00344d3d33fa853f1a513cbb03780c078d491a08ca932293aa22909a 字节一致冻结。Linux artifact 15085584093 两个 ARM ELF 真实 QEMU 均 8 个 SysTick、计划 tick 2/4/6/8，raw 1500/2100/1800/1400，enabled 0/1/1/0，missed 均 0，模拟联锁触发低指令；主机另验错过周期和时钟回绕。Windows artifact 15085459020 主机与 ARM cross build 通过，不声称 Windows QEMU。完整回归 Run 45751986633 五项失败均为诊断补丁的 learning_e2e.rs:3 import rustfmt（artifact 15085578205/15085229153），按原始 diff 修正格式，不更改 8 秒门槛、日志语义或任何合同。合并 E2 的共享文件、manifest 和课程导航后，冻结锁与双平台重验；独立人类/硬件/WCET 仍未评估。

E3 集成后 Run 45754480406 三项嵌入式专项通过；E2 回归 Run 45754480262 的 resolve/audit 与 Windows 合同通过，Ubuntu 原始日志 job 139690815456 显示参考 Debug/Release、四步 RED/GREEN、vendor 与 native-build 已通过，实际窗口 Ctrl+F 成功（8 行、3 告警、3 可见），Ctrl+L 未使标题从 lang=en 转为 lang=zh。失败 artifact 15087616609 保留。代码审查发现快捷键以帧末 Ctrl 状态与 key_pressed 联合判断，快速按下释放可能发生在同帧；按实际 Key 事件的 modifiers/pressed/repeat 匹配，参考与学员公共 GUI 逻辑同步，增加同帧按下/释放、无 Ctrl、重复键及其他键回归测试。未修改真实 X11 截图、三帧哈希与标题断言、锁或学员合同。此为有证据支持的候选修复，根因需当前提交真实重验确认；旧通过结果不代替新提交完整 CI。

## 开发经验（E4）

E3 的修复提交 eb2cafc6 已独立触发完整 CI；E4 以该提交为后续候选基线，不取消 #27、不提前合入。使用无第三方依赖的同一 Rust 安全核心，原生 C ABI 使用调用方所有的定长结构和有界数组，WASM 只暴露标量分类接口，JS 在传入 i32 前拒绝非法值，不共享 Rust 指针。参考与学员保留相同 ABI 头、C 客户端、Web 页面和测试；三个 TODO、三项 RED/GREEN 合同，双平台实际 C 编译和动态链接、实际 wasm32 编译/Node 执行、Ubuntu Chromium 页面交互与截图。源清单及 Python/PowerShell 指纹同提交更新，旧 MSRV、rc.5、已发布资产不变。C 指针有效性仍是调用方安全契约；未声称所有外部输入安全、真人学习效果或 Windows 实际浏览器已验收。必须先收口 E3 后再整合 E4。
