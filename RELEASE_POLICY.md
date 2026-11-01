# Rust AppLab 0.18.0-dev.1 版本与准入规则

## 正式声明的平台

**Windows x64 MSVC 与 Linux x86_64。** 不声明 macOS 支持；macOS 原生验收不再列为 Stable 必须条件。详见 [支持矩阵](SUPPORTED_PLATFORMS.md)。

## RC 与 Stable 必须分开

- **冻结的 0.17.0-rc.2**：已根据 Linux、Windows 原生工程、Chrome 真实浏览器及归档证据达到 `RC_READY`；原始发布物和证据必须保持不变。
- **0.18.0-dev.1**：本轮新增学习导航和解释器，是独立开发分支；**不能继承旧版本 RC_READY/Stable 资格**。
- **Stable**：至少需要当前源码的 Linux 与 Windows 工程验收、浏览器关键业务闭环、归档源码与二进制哈希、最低 Rust 1.74.0 兼容记录，以及实际学员在 FirstCard、UnitLab、独立作品路线的观察数据和未解决阻断的处置。自动测试不能取代真人学习记录。

## 证据规则

`scripts/release_decision.py` 按 source_id 和版本号匹配完整原始日志；禁止把不同源码的 Windows、Linux、浏览器证据混入同一次资格判定。源码修改后重跑对应验收。

原候选发布详细规则在 `verification/QUALITY_GATE_HISTORY.md`，原始 RC 资格资料在 `verification/RC2_QUALIFICATION.md`。RC 证据属于冻结历史，不因本分支修改而变化。

## 已知课程范围

17 个 Workspace 包、12 款 Studio、36 个学习里程碑、FirstCard、TaskPocket/PageMint 与 CASE 01 原有案例保留。Tokio/Reqwest/Axum、桌面 GUI 与 WASM 课程仍未完成，不能因支持平台收口就宣称全领域教程已经全部建成。

## 安全边界

学习代码评测只针对学员信任的自编源码，不是恶意代码执行沙箱；本地 HTTP 不可用作公网服务；SQLite 需系统库；备份工具不能代替生产级灾难恢复。
