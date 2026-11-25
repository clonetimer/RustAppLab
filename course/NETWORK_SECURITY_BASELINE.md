# E1-0 · 网络课程安全基线与学员迁移（2026-11-25）

[现代化路线](MODERN_RUST_ROADMAP.md) · [网络依赖准备](NETWORK_PREPARATION.md) · [安全验证](BOOK_D11_SAFETY.md)

## 真实发现（修复前）

D11-B [Run 45611450910](https://github.com/clonetimer/RustAppLab/actions/runs/45611450910) 五项专项 PASS；[原始审计产物](https://github.com/clonetimer/RustAppLab/actions/runs/45611450910/artifacts/15020451443) 使用 cargo-audit 0.23.0、固定 RustSec DB b9b8a3c2cb1f2a3c76ab17a551193568c9f8b7a4。C1/C2 参考锁出现 url 2.5.8 [RUSTSEC-2026-0592](https://rustsec.org/advisories/RUSTSEC-2026-0592.html)；C1/C2/D10-B 参考锁出现 Tokio 1.48.0 [RUSTSEC-2026-0386](https://rustsec.org/advisories/RUSTSEC-2026-0386.html)（unsound 警告）。根工作区及 D11-A 未检出公告。原教学审计 PASS 代表工具和历史夹具对照成功，不意味着实际项目不存在风险。公告是锁文件级别，未声称应用调用路径已被利用。

## 本轮最小修复

| 依赖 | 旧版 | 候选版 | 依据 |
|---|---|---|---|
| tokio | 1.48.0 | 1.53.3 | 已修复的 LTS；上游 MSRV 1.71 |
| tokio-macros | 2.6.1 | 2.7.0 | Tokio 1.53.3 的版本约束 |
| url | 2.5.8 | 2.5.9 | RustSec 公告的修复版本 |
| form_urlencoded | 1.2.2 | 1.2.3 | url 2.5.9 的版本约束 |

C1、C2、D10-B 的 learner/reference 清单与锁同时更新；四个版本的 checksum 来自 crates.io-index 对应 registry 记录，不伪造。保留旧锁中的其他版本和业务接口，候选锁由真实 Cargo --locked 独立验证。C1/C2 继续 Rust 1.85，D10-B 继续 Rust 1.100；基础根工作区及 Rust 1.74 不变。旧可选 ecosystem 探针更新 Tokio、固定 URL 解析依赖；不把该探针当成 E1 产品。

同快照 RustSec 将逐份扫描六个参考与学员锁，收齐完整 JSON 后拒绝任何 vulnerability 或 unsound 警告，保留其他警告原文。两个指定公告必须消失；不能仅因 cargo-audit 退出 0 就忽略 unsound。CI 必须验证 C1/C2 固定工具链、实际行为/CLI/合同、vendor 空缓存重放、D10-B 双平台专项与完整平台交付。此处为候选修复，未宣称新 CI 已通过；固定快照不覆盖更新后的公告、yanked、系统库与实际公网运行风险。

## 已有学员工程如何迁移（仅在新包通过验收后）

旧 rc.5 和历史 ZIP 不会被重新写入，也不会自动更新。它们保留历史证据，不代表新版本已修复。建议停用旧依赖组合的网络用途，尤其不要暴露到公网或传入不可信 URL。

1. 保留旧目录、自己的 src、学习记录与测试输出，不覆盖原文件。
2. 在新目录取得修复后、同提交验证的 learner 模板或离线包（C1/C2/D10-B 对应）。只迁移自己的源码改动和必要自写测试，不复制旧 Cargo.toml、Cargo.lock、vendor 或 .cargo/config.toml；若使用旧平台标记的独立项目，按新入口创建新工程。
3. 重新准备对应依赖或使用新离线包的 vendor，确认平台环境变量指向新 vendor；运行原有固定合同与新包的 cargo test --frozen。旧网络探针的固定源码发生变化，重新 init 到新目录，旧 STATUS.md 不代表新验收。
4. 旧成绩与历史源码快照保留；只有新回归成功才记录当前版本的新成绩。不要修改旧根工作区、全局工具链或全局 registry 配置。

## E1 后续边界

- E1-A：在已有 C2 HTTP/JSON 业务上学习真实 TLS 握手与证书信任、认证、授权、错误边界；回环测试需覆盖受信证书、错误信任、缺失/无效凭证和权限不足，不关闭 TLS 校验。不使用真实密钥。
- E1-B：持久化数据、事务与并发、结构化日志、敏感字段脱敏、可观测性、限流/超时与退出恢复。保留固定合同、学员模板、真实测试、离线交付。不是公网生产部署保证。
- E1-0 先实际回归、重审计、合并；新工程不复用已知告警版本。图形化工作台与真人理解另验。

参考：[Tokio 1.53.3 文档](https://docs.rs/crate/tokio/1.53.3) · [url 2.5.9](https://docs.rs/crate/url/2.5.9) · [RustSec](https://rustsec.org/advisories/)。

## 实际收口（2026-11-25）

候选 a43d85f 的 [完整 CI](https://github.com/clonetimer/RustAppLab/actions/runs/45612993818) 与 [D11-B/RustSec 专项](https://github.com/clonetimer/RustAppLab/actions/runs/45612993552) 均成功；[PR #23](https://github.com/clonetimer/RustAppLab/pull/23) 累计合入 D11-B/E1-0，main 0e71b443。六份锁的固定快照准入通过，旧 rc.5 仍为历史资产。E1-A 新增依赖须另审计，见 [HTTPS 教学](BOOK_E1_TLS_AUTH.md)。
