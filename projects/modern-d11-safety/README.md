# D11-B · 内存安全、覆盖率引导测试与依赖审计

入口：[中文讲义](../../course/BOOK_D11_SAFETY.md)。

- learner：Rust 1.100.0 独立安全接口练习，两个 TODO，安装后可离线。
- miri-practice：两个明确标注 MIRI-ONLY 的故意 UB 反例，只在 Miri 下运行，不是正常 Cargo target。参考修复只在完整源码包中。
- fuzz-practice：真实 cargo-fuzz/libFuzzer harness，复制到已完成 D11-A 帧处理工程；不是再造帧处理项目。
- audit-practice：真实 RustSec 历史漏洞的 lockfile-only 教学夹具，禁止构建或运行旧依赖。
- reference：完整 source 包中的参考解答，不进入无答案学员包。

Miri 与 libFuzzer 的自动验收只声明 Ubuntu x86_64，固定 nightly-2026-11-17；Windows 原生稳定接口测试仍独立通过 CI。RustSec 用固定数据库快照重放课堂实验，不宣称该快照永远最新。网页入口和真人效果另验。
