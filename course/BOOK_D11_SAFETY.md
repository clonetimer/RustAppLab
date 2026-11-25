# D11-B · Rust 内存安全与软件供应链验证

[现代化计划](MODERN_RUST_ROADMAP.md) · [D11-A 工程质量](BOOK_D11_QUALITY.md) · [本课入口](../projects/modern-d11-safety/README.md)

先完成稳定版质量课程，再选用额外工具。D11-B 不改变已有 MSRV 或全局默认工具链。普通 Windows/Linux 学员先做两个安全接口任务；Miri、libFuzzer 是 Ubuntu x86_64 独立选修，不能由 Windows 原生测试替代。

| 阶段 | 学员操作 | 实际验收 |
|---|---|---|
| 1 安全边界 | 实现 checked_read 与 initialized_prefix；比较 safe 和 unsafe | 边界、空切片、最大下标、N=0、初始化完整性；两个 RED/GREEN 固定合同 |
| 2 Miri | 在 Miri 中运行两份故意越界/未初始化反例，读报告后修复 | 真实解释执行产生 Undefined Behavior，修复后通过；不原生执行 UB |
| 3 libFuzzer | 复用 D11-A codec，编写或扩展 fuzz target | 真实覆盖率引导执行、固定预算、保留 corpus/反例、恢复后重放 |
| 4 RustSec | 审计历史漏洞锁文件并比较修复版 | 同数据库、同工具，指定公告命中与消失；不执行旧依赖；额外扫描当前实际项目锁文件 |

## 安全不变量

unsafe 不会关闭借用检查，也不等于更快。checked_read 需要先证明索引在切片范围内，initialized_prefix 必须证明每一槽在读取前已初始化。参考 unsafe 实现仅为学习局部不变量；生产场景优先 get/copy_from_slice 等安全方法。固定 u32 不可泛化为任意具有析构语义的类型。

Miri 是实际 UB 检测工具，不是编译器所有路径证明。按 [Miri 练习](../projects/modern-d11-safety/miri-practice/README.md) 安装日期固定 nightly-2026-11-17 与 miri、rust-src。两个红色程序只进入临时 Miri probe，不通过原生运行来观察 UB。Windows 仅运行稳定合同，不冒称 Miri 平台覆盖。

## 真实 Fuzzing

D11-A 是确定性生成式属性检查；[libFuzzer 练习](../projects/modern-d11-safety/fuzz-practice/README.md) 用 cargo-fuzz 0.13.2 与 libfuzzer-sys 0.4.13 实际覆盖率引导输入变异。对任意字节尝试 decode，接受时要求重新 encode 后与原字节一致；对合法 payload 要求 round-trip 和单比特错误拒绝。先正常运行 4096 次，再在临时参考副本取消 checksum 比较，要求真实 fuzzer 发现并保存反例；恢复真实源码后重放实际 artifact，并再跑一次正常预算。XOR 仅教学错误检测，不是安全认证或加密。有限运行不保证无 bug；覆盖率统计不是 100% 分支覆盖率声明。

固定直接依赖，首次 Cargo 解析/下载后保存实际 Cargo.lock 与哈希；执行 fuzz 阶段离线，不允许锁文件发生变化。CI 记录工具、计数和 crash artifact，不能声称跨时间解析出的传递依赖已完全相同。

## 真实 RustSec 审计

[审计练习](../projects/modern-d11-safety/audit-practice/README.md) 使用 cargo-audit 0.23.0 与固定 RustSec advisory-db 提交 b9b8a3c2cb1f2a3c76ab17a551193568c9f8b7a4（2026-11-25）。历史 regex 1.5.4/1.5.5 的对照只生成并审计锁文件，不编译或运行易受影响的旧 crate；验证 RUSTSEC-2022-0013 指定公告是否消失，不开发漏洞利用代码。

同时对平台 Cargo.lock、C1/C2、D10-B、D11-A 的真实锁文件保存完整 JSON 报告与摘要。发现其他公告应如实记录并单独评估，报告阶段不自动抹掉发现或声称项目安全。固定快照不等于实时数据库；本实验关闭 yanked 查询且不作二进制/OS 依赖完整审计。正式生产应更新数据库并评估受影响配置、可达性与修复回归。

维护者在平台根目录执行 python scripts/modern_d11b_check.py --mode stable --output d11b-stable-evidence；Linux 另分别运行 --mode miri / fuzz / audit（不同输出目录）。额外工具首次准备需联网；CI 真实安装固定版本并保存每个模式的日志。所有变更均在临时副本，原始教材与锁文件不改。尚待当前提交 CI。真人 IDE、网页入口、实际学习结果仍需另验。

参考：[Miri](https://github.com/rust-lang/miri) · [Rust Fuzz Book](https://rust-fuzz.github.io/book/) · [cargo-fuzz 0.13.2](https://github.com/rust-fuzz/cargo-fuzz/releases/tag/0.13.2) · [cargo-audit 0.23.0](https://github.com/rustsec/rustsec/releases/tag/cargo-audit/v0.23.0) · [RustSec 公告](https://rustsec.org/advisories/RUSTSEC-2022-0013.html)。
