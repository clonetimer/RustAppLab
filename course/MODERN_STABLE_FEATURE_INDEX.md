# D9 · 1.75–1.100 稳定特性版本索引

以 [Rust 官方发布公告索引](https://blog.rust-lang.org/releases/) 为准，固定 Rust 1.100.0（2026-11-12）作为现代路线工具链；仅列 minor 发布，不把补丁安全更新当新语言特性。以下是 26 个待审计版本入口，不是已逐项穷尽的特性清单。明确列出的代表性特性已有官方公告依据，其余需逐版审阅详细 release notes，D9-B 继续补充。D9-A 工程实现见 [中文讲义](BOOK_D9_STABLE_FEATURES.md)，所有课程验收以同提交 CI 为准。

| 版本 | 官方公告 | 已定位的教学内容或审计状态 |
|---|---|---|
| 1.75.0 | [Release](https://blog.rust-lang.org/releases/1.75.0) | async fn/RPITIT（D9-A async trait 任务；RPITIT 专项待补） |
| 1.76.0 | [Release](https://blog.rust-lang.org/releases/1.76.0) | 待逐版审计（不表示无新增特性） |
| 1.77.0 | [Release](https://blog.rust-lang.org/releases/1.77.0) | 待逐版审计（不表示无新增特性） |
| 1.78.0 | [Release](https://blog.rust-lang.org/releases/1.78.0) | 待逐版审计（不表示无新增特性） |
| 1.79.0 | [Release](https://blog.rust-lang.org/releases/1.79.0) | 待逐版审计（不表示无新增特性） |
| 1.80.0 | [Release](https://blog.rust-lang.org/releases/1.80.0) | 待逐版审计（不表示无新增特性） |
| 1.81.0 | [Release](https://blog.rust-lang.org/releases/1.81.0) | 待逐版审计（不表示无新增特性） |
| 1.82.0 | [Release](https://blog.rust-lang.org/releases/1.82.0) | use<...> 精确捕获（D9-A） |
| 1.83.0 | [Release](https://blog.rust-lang.org/releases/1.83.0) | 待逐版审计（不表示无新增特性） |
| 1.84.0 | [Release](https://blog.rust-lang.org/releases/1.84.0) | 待逐版审计（不表示无新增特性） |
| 1.85.0 | [Release](https://blog.rust-lang.org/releases/1.85.0) | Edition 2024（D8）、异步闭包与 AsyncFnMut（D9-A） |
| 1.86.0 | [Release](https://blog.rust-lang.org/releases/1.86.0) | Trait upcasting（D9-A）；get_disjoint_mut 待扩展 |
| 1.87.0 | [Release](https://blog.rust-lang.org/releases/1.87.0) | 待逐版审计（不表示无新增特性） |
| 1.88.0 | [Release](https://blog.rust-lang.org/releases/1.88.0) | let chains（D8） |
| 1.89.0 | [Release](https://blog.rust-lang.org/releases/1.89.0) | 待逐版审计（不表示无新增特性） |
| 1.90.0 | [Release](https://blog.rust-lang.org/releases/1.90.0) | 待逐版审计（不表示无新增特性） |
| 1.91.0 | [Release](https://blog.rust-lang.org/releases/1.91.0) | 待逐版审计（不表示无新增特性） |
| 1.92.0 | [Release](https://blog.rust-lang.org/releases/1.92.0) | 待逐版审计（不表示无新增特性） |
| 1.93.0 | [Release](https://blog.rust-lang.org/releases/1.93.0) | 待逐版审计（不表示无新增特性） |
| 1.94.0 | [Release](https://blog.rust-lang.org/releases/1.94.0) | 待逐版审计（不表示无新增特性） |
| 1.95.0 | [Release](https://blog.rust-lang.org/releases/1.95.0) | match if-let guards（D9-A） |
| 1.96.0 | [Release](https://blog.rust-lang.org/releases/1.96.0) | 待逐版审计（不表示无新增特性） |
| 1.97.0 | [Release](https://blog.rust-lang.org/releases/1.97.0) | 待逐版审计（不表示无新增特性） |
| 1.98.0 | [Release](https://blog.rust-lang.org/releases/1.98.0) | 待逐版审计（不表示无新增特性） |
| 1.99.0 | [Release](https://blog.rust-lang.org/releases/1.99.0) | 待逐版审计（不表示无新增特性） |
| 1.100.0 | [Release](https://blog.rust-lang.org/releases/1.100.0) | Never、DropGuard、Allocator 核心（D9-A）；ZST、Cargo 新能力及其余 API 待扩展 |

GAT 稳定于 [1.65.0](https://blog.rust-lang.org/releases/1.65.0)，D9-A 将其作为进阶 Trait 前置缺口补课，不计入 1.75–1.100 新增特性。1.100 Never 的旧 Edition fallback 变化可能影响经典代码；保留既有回归门槛。Allocator 仅使用已稳定的核心 Vec/Box 接口，try_new_in 等 allocator_ext 仍不能算稳定必修；自定义 unsafe allocator、conjure_zst 与安全不变量转 D11/后续专题。Cargo 的依赖未使用诊断、依赖冷却时间不等于供应链完全安全，待 D9-B/D11 受控实验。D10 承接真正挂起/唤醒、流与运行时；D9-A 的 ready-only 单次 poll 不替代运行时课程。

官方 [1.75](https://blog.rust-lang.org/releases/1.75.0) · [1.82](https://blog.rust-lang.org/releases/1.82.0) · [1.85](https://blog.rust-lang.org/releases/1.85.0) · [1.86](https://blog.rust-lang.org/releases/1.86.0) · [1.95](https://blog.rust-lang.org/releases/1.95.0) · [1.100](https://blog.rust-lang.org/releases/1.100.0)。
