# 0.13.0 当前门槛与统计口径

13 个 Workspace Package，八款 Studio 软件、24 个分步里程碑；保留 32 篇原有讲义、两项独立开发作品及 FirstCard 热身。主线材料 14/20，尚未交付的六款不能计入完成。

新门槛：TermGarden 真实交互与重放，任务持久化及错误脚本不部分提交；StoreDesk 使用真实 SQLite，跨进程保存、参数绑定、批次失败同时撤销库存与流水；两款模板失败→只完成第一步通过→参考实现全通过。完整当前统计见 `RUST_1_99_1_0_13_0_EVIDENCE.txt`，不是把历史测试数字相加。

新增 Windows 菜单和 SQLite SDK 链接仍待原生验证；macOS、Rust 1.74、真实学员独立完成情况亦待补。已有用户对0.11.1的确认仅适用于该次修复。SQLite 包含系统原生依赖；不能因为 Cargo --offline 成功就称为完全自包含发行包。无生产数据库/断电/安全审计认证。

---
以下是保留的历史门槛，不代表本轮执行结果。

# Rust AppLab 0.11.1 · Windows 新手路径修复验收门槛

**状态：Linux 已通过，Windows 原生脚本未在本环境执行；仍为教学开发版。**

- 0.11.0 的单层相对路径 `my-first-card` 会被误判为父目录不存在。此问题必须能在旧版复现，在 0.11.1 通过相同 CLI 验证。
- 新增测试必须从单独工作目录以相对路径 `beginner start my-first-card` 真实创建项目并用 `beginner check my-first-card` 通过；已有目录拒绝覆盖；`missing/card` 不创建父目录；已存在 `sub/` 可创建 `sub/card`。
- Windows 向导必须使用 `$Project` 完整路径；Windows CI 冒烟必须同时检查单层相对路径，不能仅验证绝对路径。
- Linux Rust 1.99.1 实测：Debug 108 通过、Release 98 通过、各 15 个 ignored、失败 0；Rustfmt、Debug/Release Clippy `-D warnings`、Release 构建通过。
- Windows 真实用户反馈已证明 0.11.0 运行失败；Windows 上 0.11.1 能否成功尚须学员回报或实际 Windows CI 的运行记录，不得等同于 Linux 自动化测试通过。

---

# Rust AppLab 0.11.0 · 零基础入门与 Windows 学员发布门槛

**当前等级：教学开发版，非 RC / Stable。** 0.11.0 重点是初学者可完成的第一行 Rust，而不是新增复杂软件数量。

- **最低产品闭环**：`beginner start` 只在不存在的目录创建独立 Cargo 小程序；初始程序真实编译但检查必须失败；只写死首组输出必须失败；正确使用动态变量的实现必须通过中文/英文/空格/无参数四组真实运行，并生成 `FIRST_WIN.md`。
- **错误友好性**：编译失败提示检查括号引号分号，不应把功能未完成与安装错误混淆；异常无限循环 4 秒内返回反馈；三层提示按需显示；已有用户目录和完成记录不覆盖。
- **Windows 学员入口**：`START_HERE.html` 可直接打开，`START_WINDOWS.cmd` 启动 PowerShell；明确检查 Rust 及 MSVC 构建工具、打开记事本、保存 Ctrl+S、重试流程。`scripts/windows_beginner_smoke.ps1` 已接入 Windows CI，但**当前执行容器没有 Windows PowerShell / MSVC 原生验证结果**。
- **教学效果实测仍缺**：学员独立修改一行的成功率、首次成功前失败原因、按需获取提示级别和是否能生成 TXT 作品，均应采集匿名课堂反馈，不等于 Cargo 回归测试结果。
- **工程门槛**：Rustfmt、Debug/Release Clippy 禁止警告、全 Workspace Debug/Release 测试、Release 构建、源码白名单打包和全新解压复验。最新 Linux 证据见 `RUST_1_99_1_0_11_0_EVIDENCE.txt`。
- **仍需完成**：Windows/macOS 原生测试、Rust 1.74 最低版本、真实浏览器直连 LocalHub 以及 SafeSync 的物理故障耐久验证。不要提前宣称 Stable。

---

# Rust AppLab 0.10.0 · 跨领域独立作品教学质量门槛

**当前等级：教学开发版，非 RC / Stable。** 本轮保留五个 Cargo Package，并在 LabQuest 中增加第二款独立作品 PageMint。

- **代码与平台证据**：Linux x86_64 / Rust 1.99.1 已运行 rustfmt、Debug/Release 严格 Clippy、Debug/Release Workspace 测试及 Release 构建；两配置均 0 失败，各 15 项有意忽略的学员挑战。详见 [`RUST_1_99_1_0_10_0_EVIDENCE.txt`](RUST_1_99_1_0_10_0_EVIDENCE.txt)。
- **PageMint 行为门槛**：空目录、有序 HTML、动态源变化、UTF-8 与 HTML 特殊字符、忽略非目标文件、错误输入不产生 HTML、拒绝覆盖、非法 CLI，在 Unix 上加测符号链接。正向参考程序 9/9 通过；未实现、固定输出和漏 HTML 转义的反例必须失败。Windows 对应 8 项基础场景，**本轮未实际在 Windows 运行**。
- **兼容性门槛**：0.9.0 旧 `app init/check/hint` 入口仍指向 TaskPocket；新 `app init/check/hint pagemint` 调用独立网页练习。
- **防抄答案交付门槛**：源码包允许维护者审计回归测试 fixture，但另提供不含参考实现的学员包；学员包保留完整需求、TODO 工程、样例与课程文档。
- **学习效果尚缺证据**：机器评测只能说明程序行为；没有真实学员独立通关时间、提示使用情况、作品能否迁移到第三个领域的证据。不得据此宣布教学 Stable。
- **执行边界**：`app check` 会编译和执行学员代码，不是安全沙箱；超时只保证尝试结束直接子进程。没有最低版本 Rust 1.74、Windows/macOS 原生、LocalHub 真实浏览器 HTTP 链路、SafeSync 断电一致性的完整本轮证据。

---

# Rust AppLab 0.9.0 · 独立产品教学验收门槛

- **当前等级：教学开发版，非 RC / Stable。** 新增 TaskPocket 独立工程生成与九组黑盒评测，具体本机证据见 [`RUST_1_99_1_0_9_0_EVIDENCE.txt`](RUST_1_99_1_0_9_0_EVIDENCE.txt)。
- **实战验收要求**：新手初始程序必须无法通过业务测试；固定 `added=1` 的假实现必须失败；正确独立 Rust 实现应在全新目录、不同数据文件、跨进程读写、Unicode、异常保护下达到 9/9，重复验收仍通过。
- **安全边界**：`app check` 会编译并执行学员代码，不是沙箱；每次运行 5 秒、构建 120 秒超时，但仅尝试终止直接子进程；不要用于不可信软件。评分与报告不构成任何安全认证。
- **教学效果尚未验证**：目前只验证评测器行为，仍缺真实人类学员独立通关时间、提示层级使用率、作品可维护性和迁移到另一应用的效果。不得将自动测试当作学员学习效果。
- **平台限制**：当前仅 Linux x86_64、Rust 1.99.1 实测；Windows/macOS 原生、MSRV Rust 1.74、LocalHub 真实浏览器直连 HTTP、SafeSync 真实断电数据持久性仍缺完整证据。

---

# Rust AppLab 0.8.0 · 发布质量门槛与实际证据

## 0.8.0 学习迁移判据（2026-10-28）

0.8.0 新案例的 `test-app` 必须在**四种实际运行场景**下逐行验证输出。Linux x86_64 / Rust 1.99.1 最终实测为 **Debug 84 项通过，Release 74 项通过，0 失败；各有 15 项学员挑战按设计忽略**，格式、严格 Debug/Release Clippy 与 Release 构建通过。新增反例包含固定错误数、`>=` 比较符边界错误、错误程序超时可终止、旧 `Case-v1` 保持兼容，以及 `next/predict` 不修改学习产物。测试结果详见 `RUST_1_99_1_0_8_0_EVIDENCE.txt`。

评估教学成果时仍需真实人类学员的独立操作反馈；当前只证明工具行为满足既定判据。此前 Windows/macOS、MSRV 与 LocalHub 浏览器端到端缺口仍未解决，继续标记为教学开发版。

## 0.7.0 历史结论（2026-10-28）

**教学开发版，非 RC 或 Stable**。本轮在 Linux x86_64 / Rust 1.99.1 对五个 Cargo package 完成 `cargo fmt`、严格 Debug/Release Clippy、Debug/Release 全量测试以及 Release 构建：**Debug 78 项基础测试通过、Release 68 项通过、0 失败；两种配置各 15 项学员挑战按设计 ignored**。LabQuest 的真实操作流程已验证六步全绿；学员未实现 SUMMARY 时必须失败，修改 Rust 代码后 `test-app` 实际运行并成功，随后完成报告、快照校验和结案。见 `RUST_1_99_1_0_7_0_EVIDENCE.txt`。

- **新课评价**：新手能否独立完成、是否需要读取第三层提示，尚无真实学习者的反馈，不能因为测试全绿就宣称教学效果已通过验证。
- **Web 验证边界**：LabQuest 进度板 HTML 由自动测试验证文字/转义/进度逻辑；本环境 Chromium 无头截图超时，**没有视觉回归通过证据**。LocalHub 浏览器直连本机 HTTP 的历史限制仍存在。
- **工具链/平台**：本轮只有 Linux Rust 1.99.1 的实测，Rust 1.74、Windows/macOS 原生及完整真实浏览器闭环仍未取证。

## 历史 0.6.0 结论（2026-10-28）

**仍为教学开发版**。本轮 0.6.0 Linux x86_64 Rust 1.99.1 已真实执行四应用 Workspace 编译、单元/集成测试、Clippy、LocalHub HTTP API 闭环和 SafeSync 实际备份/恢复。**Debug 69 项基础测试通过、0 失败；Release 59 项通过、0 失败；两种配置均有 15 项学员挑战按设计忽略**（包含 0.5.0 新增的异常中断测试）。0.4.0 曾用 Python hashlib 独立验证 201 个备份文件（含 5 MiB 以上二进制）的 SHA-256 与恢复内容；0.5.0 新增三种 Release 真文件负载，并由 Python hashlib 独立逐文件校验。原有 0.2.0 首轮实测：27 项基础测试通过、9 项学员挑战按设计忽略；格式检查不通过、Clippy 1 处告警，均在本轮修复。

0.6.0 Linux 本轮结果见 `verification/RUST_1_99_1_0_6_0_EVIDENCE.txt`；0.5.0 历史结果见 `verification/RUST_1_99_1_0_5_0_EVIDENCE.txt`，0.4.0 结果见 `verification/RUST_1_99_1_0_4_0_EVIDENCE.txt`，0.3.0 原始证据仍保留在 `verification/RUST_1_99_1_LINUX_TEST_EVIDENCE.txt`。**单机 Linux 结果不等于 Windows/macOS、Rust 1.74 MSRV、浏览器兼容性或 Stable 合格。**

## 当前验收矩阵

| 需求 | 验证方式 | Linux 证据边界 |
|---|---|---|
| FQ-01 | FileQuest `demo/scan/find/report` CLI 闭环 | `apps/filequest/tests/cli_smoke.rs` 真运行 |
| FQ-02 | 文件扫描、符号链接、忽略目录、特殊文件名、HTML 转义 | `apps/filequest/tests/acceptance.rs` 真运行 |
| LL-01 | LogLens `demo/analyze/compare/report` | `apps/loglens/tests/cli_smoke.rs` 真运行 |
| LL-02 | 日志解析、日期、错误率、未识别、报告保护 | 单元与 `acceptance.rs` 真运行 |
| LH-01 | `demo` 后真实 TCP 服务器、端口 0、自动退出 | `apps/localhub/tests/cli_smoke.rs` 真运行 |
| LH-02 | `/`、静态 JS/CSS、`/api/files`、`/api/logs`、`/api/health` | `apps/localhub/tests/acceptance.rs` 真运行 |
| LH-03 | 非法 Host、非 GET、参数、未知路径和 8192B 头部 | `acceptance.rs` + `http.rs` 单元测试真运行 |
| LH-04 | 浏览器交互、中文路径和移动端布局 | 已用 Chromium 验证 **DOM/JS 渲染与搜索**，数据来自真实 Rust HTTP 但在浏览器内使用模拟 fetch；浏览器直连本机 HTTP 被管理员策略阻断，**完整浏览器端到端待验证** |
| SS-01 | `backup/verify/restore` 真实文件闭环和 SHA-256 标准向量 | `apps/safesync/tests/acceptance.rs`、`cli_smoke.rs` 与 SHA-256 单元测试 |
| SS-02 | 文件损坏、额外文件、路径穿越、符号链接、覆盖/幂等拒绝 | `apps/safesync/tests/acceptance.rs` 的异常/边界实测 |
| SS-03 | `stages` 标记 READY / INCOMPLETE / RESTORE / LEGACY / UNSAFE，`recover` 仅接受完整的同父目录同目标备份 | `apps/safesync/tests/recovery.rs` 真子进程 exit(86)、CLI 与文件系统验收 |
| SS-04 | I/O 失败清理与异常退出遗留暂存两种相反行为 | `ordinary_io_error_rolls_back_staging_through_drop` 等测试 |
| SS-05 | 32 MiB / 64 文件、8 MiB / 512 文件、32 MiB / 2 文件 Release 基准与外部 SHA-256 | 三份 `SAFESYNC_0_5_0_BENCHMARK*.json` 真正运行生成 |
| SS-06 | 在复制期间核对实际读取的大小和 SHA-256；暂存通过后重新扫描源目录 | `ops.rs` 单元测试及 `recovery.rs` Debug 子进程 I/O 故障回归 |
| SS-07 | 备份与恢复暂存用途隔离、旧版无标记暂存的显式迁移 | `recovery.rs` 中完整恢复暂存反例、畸形名称及旧版 CLI 测试 |
| EDU-01 | 00–27 关、分级提示、学员挑战 | `scripts/structural_check.py` 验证结构；**15 项教学挑战按设计不运行** |
| QA-01 | rustfmt / Debug 与 Release Clippy / Cargo tests / Release 构建 | 本轮全绿；Debug 84 项、Release 74 项通过，15 项挑战未执行，详见 `RUST_1_99_1_0_8_0_EVIDENCE.txt` |
| LQ-02 | CASE 01 v2 的四场景真实程序测试、硬编码与边界反例、`next`/`predict` 只读反馈、旧案例兼容 | `apps/labquest/tests/case_e2e.rs`，`coach.rs`，Linux 1.99.1 实测；不代表真实学习效果结论 |
| LQ-01 | CASE 01 六项任务判定、源码变更证据失效、离线 HTML 进度板、真实 Cargo 运行 | `apps/labquest/tests/case_e2e.rs` 的真实临时目录回归 + 手工跨产品通关；浏览器视觉未通过 |
| QA-02 | Windows/macOS 的 CI、Rust 1.74 MSRV | CI 已加单独 Linux Rust 1.74.0 MSRV job；**未获取实际通过记录** |

## SafeSync 手工操作验收

```sh
cargo run -p safesync -- demo my-sfs-gate
cargo run -p safesync -- verify my-sfs-gate/backup
cargo run -p safesync -- restore my-sfs-gate/backup my-sfs-gate/restored-2
# 在这个新创建的练习目录修改 my-sfs-gate/backup/data/README.txt 中一个字
cargo run -p safesync -- verify my-sfs-gate/backup
# 预期非零退出；不要拿真实工作区测试意外破坏
```

## 操作验证（LocalHub 网络端到端仍需要手工观察）

```sh
cargo run -p localhub -- demo another-localhub-demo
cargo run -p localhub -- serve --root another-localhub-demo/files --log another-localhub-demo/logs/candidate.log
# 打开 http://127.0.0.1:8765/，确认文件总数和日志错误率 37.50%
# 第二终端：
curl -i http://127.0.0.1:8765/api/health
curl -i http://127.0.0.1:8765/api/files?limit=3
curl -i http://127.0.0.1:8765/api/logs
```

启动演示目录需要是全新的，已有目录会被拒绝。FileQuest 目录有 9 个样例文件及 `report.html`，因此 LocalHub 初始快照通常有 **10 个普通文件**。候选日志 8 条被识别、3 条 ERROR/FATAL、1 条未识别，所以占比 **37.50%**；这是日志记录而非 HTTP 请求失败率。

## 安全与资源边界（不得忽略）

1. LocalHub 仅监听 `127.0.0.1`，未实现身份认证；**其他本机进程仍可访问**。不要通过代理、端口转发、SSH 公网映射等方式暴露此服务。
2. HTTP 协议为教学实现，仅支持 HTTP/1.1 GET、串行连接、有限的请求头解析；3 秒读写超时不保证吞吐，慢客户端可造成等待。严禁声称生产级 HTTP 支持。
3. 扫描由使用者通过 `--root` 授权，网页不能改写路径；扫描器跳过链接，但存在路径在检查后被外部进程修改的竞争。启动时扫描完整目录并分析所选日志，**资源占用尚未设硬上限**。
4. JSON 字符串使用专用转义函数，页面使用 `textContent`；但日志消息和路径仍属潜在敏感信息，LocalHub API 可返回高频原始日志消息。
5. FileQuest/LogLens 的 `--force` 对文件覆盖缺乏完全的事务性与并发安全保证，避免用于重要原文件；LogLens 的解析格式和统计口径有明确局限。
6. SafeSync 是普通文件备份教学软件；不保存权限/ACL/时间戳，禁止覆盖但不能保证断电耐久性、实时一致性、反恶意篡改或完全免竞态。`stages` 可读取暂存状态，只有合法 B 类型完整 READY 能 `recover`；旧版经人工确认后才使用 `recover-legacy`；其余 `.part` 由人工辨认后再处理，绝不自动删除；debug 注入 exit(86) 不代表真正断电。
7. 教学挑战默认 `#[ignore]`，属于学员应实现的未来功能。不得纳入基础测试已完成总数。

## 发布等级

- **开发版**：已提交源码与 Linux 本机质量证据，已知限制公开。当前 0.8.0 属于此级别。
- **RC 候选**：功能冻结，浏览器交互真实验收、异常边界和资源基本评估完整；保留逐平台验证日志。
- **Stable**：在拟支持的 Windows/macOS/Linux 环境及实际最低 Rust 版本完成构建和功能验收，无阻断缺陷；实测性能和安全边界与文档一致。

## 浏览器实际复验边界（0.4.0）

真实 Chrome/Chromium 可启动，但导航 `http://127.0.0.1:<port>/` 返回 `net::ERR_BLOCKED_BY_ADMINISTRATOR`，这是执行环境的浏览器网络策略限制，不是产品页面正常打开的证据。

替代测试 `python3 scripts/browser_dom_smoke.py` 使用 **真实 Rust HTTP API 提供的 JSON**，通过 Python 读取响应后在 Chromium `set_content` 页面中用 mock fetch 注入，验证：页面数据渲染、动态搜索、恶意文件名作为 textContent 而非 HTML 插入、移动端无水平溢出和 JS 无异常。结果及截图见 `LOCALHUB_UI_DOM_TEST_EVIDENCE.txt`、`LOCALHUB_UI_DOM_DESKTOP.png`、`LOCALHUB_UI_DOM_MOBILE.png`。这只能算组件级 DOM 验证，不能代替浏览器→HTTP 网络请求、CSP/缓存头、真实加载路径的完整端到端测试。


## 0.5.0 故障注入与性能数据（2026-10-28）

- 正常 `backup` → `verify` → `restore` 继续有效；新 `stages` 无副作用列出暂存状态，新 `recover` 只提交已通过独立 `verify` 的完整备份暂存。
- Debug 子进程模拟 `backup_after_verify` 后保留 READY，源目录移除后仍可 `recover`；模拟 `backup_after_first_file` 和 `restore_after_first_file` 后为 INCOMPLETE，不可强制提交；模拟普通 I/O 错误可走 Drop 正常清理。测试代码见 `apps/safesync/tests/recovery.rs`。
- 基准样本文件数分别为 64/512/2，数据 32/8/32 MiB，生成于各自临时目录，完整跑 Release 真实命令，再由 Python `hashlib` 分别核验源、快照和恢复文件。原始环境、耗时和数字在 `verification/SAFESYNC_0_5_0_BENCHMARK*.json`。不以固定 MiB/s 作为发布质量阈值。
- **尚未测试**：Windows/macOS 实际编译运行、Rust 1.74.0 本机或 CI 回报、断电持久性、磁盘空间耗尽的真实文件系统行为、跨平台 Unicode/权限/时间戳保留、非敌对假设以外的并发、LocalHub 浏览器直接连本机 HTTP。不能把这些未完成项当作 Stable 已满足。


## 0.6.0 新增质量证据及未解决问题

- `RUST_1_99_1_0_6_0_EVIDENCE.txt` 保存真实格式检查、Debug/Release 严格 Clippy、两种配置的 Workspace 测试和 Release 构建输出；Debug 69 项通过、Release 59 项通过、0 失败，各有 15 项教学挑战按预期忽略。
- `SAFESYNC_0_6_0_BENCHMARK.json` 是在新二进制上执行的 64 文件、16 MiB 真实操作，并由独立 Python SHA-256 校验原始/快照/恢复数据，`all_content_equal=true`。
- 新增确定性源文件修改、复制中间模拟 I/O 错误、完整恢复暂存误判、旧版显式迁移和畸形类型名拒绝等回归。**模拟 I/O 故障不代表真实 ENOSPC 或断电。**
- **未实测**：最低 Rust 1.74.0 的真实构建与测试，Windows/macOS 本机回归，真实浏览器直接访问 LocalHub 网络接口，恶意并发改写、物理断电与文件系统 `fsync` 持久性；不能将这些项目视为通过。
