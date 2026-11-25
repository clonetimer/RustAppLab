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
| D9 | 1.75–1.100 稳定特性 | 版本增量审计和优先级；语法、Trait、生命周期捕获、async closure、Never、DropGuard 等代表性练习；按稳定范围逐项验收 | D9-A [九项实际任务](BOOK_D9_STABLE_FEATURES.md) 与 [26 版本索引](MODERN_STABLE_FEATURE_INDEX.md) 已提交待 CI；D9-B 逐版补审与标准库/Cargo 尚未完成 |
| D10 | 异步核心课 | 六节目标逐项关闭；Future、Poll、Waker、Pin/Unpin、Stream、join/select、async trait、取消与阻塞；以已有应用串起概念而非重复项目 | 待实施 |
| D11 | 现代开发工具 | IDE 断点、Clippy、Miri、属性测试、Fuzzing、基准与依赖审计；独立运行证据及学员迁移记录 | 待实施 |
| E1 | 网络工程 | TLS、认证、数据库、可观测性、错误边界；明确本地教学与公网安全差异 | 待评估 |
| E2 | 桌面 GUI | 一款完整可交互软件，模型/视图/状态分离，跨平台测试 | 待评估 |
| E3 | 嵌入式与实时 | no_std、硬件抽象、仿真与硬件分别验证 | 待评估 |
| E4 | 互操作与 WASM | C ABI、安全封装、跨语言所有权；浏览器 WASM 独立小项目 | 待评估 |
| E5 | 科学计算和 AI | 数值/数据处理、性能测量；可选 AI 应用，准确说明计算后端 | 待评估 |

## 本轮开发经验（D7）

旧站与最新版章号不同，不能简单沿用 81 项；用 Rust 发布 tag 的 Book 子模块固定目录与文档。复用已有证据路径，所有条目保留待核验分类，不把重编号当成教学完成。当前项目的 AsyncBoard 已经有教学 executor，C1/C2 有实际 Tokio 应用；现代章节的 Stream、Pin 和动态 Future 仍需针对性学习任务，而不是重新建同类应用。

参考：[官方 Book](https://doc.rust-lang.org/book/) · [Edition Guide](https://doc.rust-lang.org/edition-guide/rust-2024/index.html) · [发布记录](https://blog.rust-lang.org/releases/) · [1.100.0 公告](https://blog.rust-lang.org/2026/11/12/Rust-1.100.0/)。

D7 CI 复核：Run 45579401311 的 tooling/structure 已通过，Linux 旧 D6 诊断复制实验因 ETXTBSY 失败；原始日志见 [Linux artifact](https://github.com/clonetimer/RustAppLab/actions/runs/45579401311/artifacts/15005702349)。只对 Linux 启动时 raw_os_error 26 有界重试，不重试子进程实际失败、不删除断言；修复后的 CI 必须重新验证。

## 开发经验（D8）

同一 Rust 1.100.0 编译器下比较两种 Edition；同一 shared.rs 消除实现差异。告警筛选器有独立 RED 合同和参考解答，只向学员包交付 TODO；维护者将答案注入隔离副本验证 GREEN。Cargo resolver 2/3 使用本地合成目录源实测而非线上不稳定版本。新增项目必须同时进入 source manifest、source_id 与学员包白名单。新版教学使用单独的 Windows/Linux CI 工作，不修改旧 MSRV 或发布版本。当前仍为 CLI 选修，网页工作台接入和真人试用未完成。

## 开发经验（D9-A）

以 1.100.0 发布公告及官方 API 稳定标签为准；GAT 为 1.65 前置，不算新版增量。九个 TODO 与固定参考合同对照，8 组编译失败/修复及 DropGuard 的 unwind/dismiss 行为单独运行；异步只使用立即完成的 Future，不拿单次 poll 假装通用运行时。系统 Allocator 安全调用替代自造 unsafe 分配器，未稳定 allocator_ext 不纳入。D8 原专项 Run 45580912220 双平台成功，D9-A 必须验证当前提交。26 个版本入口不等于 26 个版本完成审计。旧 MSRV 与发布资产不变，CLI 选修；网页与真人观察另行验收。
