# D11-A · 现代 Rust 工程质量：可验证的数据处理

[现代化计划](MODERN_RUST_ROADMAP.md) · [D11-A 实验室](../projects/modern-d11-quality/README.md) · [学员操作](../projects/modern-d11-quality/learner/README.md) · [工具诊断](../projects/modern-d11-quality/tools-practice/README.md)

使用固定 Rust 1.100.0 / Edition 2024，安装工具链及 rustfmt、Clippy 后不依赖第三方 crate。经典 Rust 1.74 课程保持独立。先把 learner 和 tools-practice 分别复制到新目录，避免改动课程源。

## 一、从真实故障到可测试的接口

定义教学帧：0xA5、1 字节长度（0–64）、payload、1 字节 XOR 校验（长度与每个数据字节异或）。严格匹配完整帧，不接收尾部多余数据。XOR 仅用于学习错误检测，不保证多比特错误或恶意篡改检测，不是 CRC/MAC。

| 步骤 | 实际要写 | 验收与迁移 |
|---|---|---|
| 1 | encode：限制大小并构造帧 | 空帧、64/65 字节边界、最大字节 |
| 2 | decode：先检查长度，再借用 payload 与核验校验 | 截断、错误 magic、非法长度、多余字节、错误校验；不复制 payload |
| 3 | count_loop | 循环中的包含边界比较，空输入 |
| 4 | count_iter | 用迭代器完成相同契约，确保不漏掉等于阈值 |

逐步执行 cargo test --locked --offline --test contracts step1_（改编号）。然后运行全部合同和 CLI：cargo run --locked --offline --example frame -- "温度"；再追加 corrupt。保留实际退出状态和错误，不能为了变绿修改固定测试。

## 二、属性测试、生成输入与故障注入

tests/properties.rs 是可复现生成式属性测试，不等同于依赖 proptest 的缩减器或 libFuzzer 覆盖率引导。固定三个随机种子，执行 4096 个合法 round-trip、每个长度代表帧的全部单比特故障位置、8192 个任意字节样本（另有 256 个单字节）、1024 组数据的四个阈值与独立直方图 oracle。任意输入不得 panic；接受的帧必须能重新编码为同一字节串。先预测，再看报错中的 case，复现并缩小反例。

维护者还在临时参考副本注入三种缺陷：忽略校验、允许尾部多余字节、将 >= 错改为 >。运行原有属性测试，要求每种 mutant 都被对应属性拒绝；恢复原件后再验证。学员可自己设计一项错误修改，先预测哪个性质会失败。注意这些有界样本不能证明对所有输入都正确。

## 三、格式化与 Clippy

进入自己的 tools-practice 副本，依 README 真正运行 rustfmt 和 Clippy；区分格式差异、needless_return lint 和业务测试。检查失败时不要加入 allow 隐藏警告。Rustfmt 修改代码后仍要编译和回归。Clippy 的警告需理解其适用语境，不能宣称能找到全部安全问题。

## 四、实际断点调试

在个人 learner 目录运行 cargo build --locked --offline --example debug_probe。Linux 可执行 gdb --args target/debug/examples/debug_probe，然后 break debug_probe::diagnose_frame、run、info args、bt、continue。预期在函数内观察 expected=42、actual=43；这是实际参数和栈帧，而不是 printf 模拟。若 Cargo 目标目录被自定义，使用 cargo build 的 JSON compiler-artifact executable 路径。Windows 使用支持 Rust 的 IDE 调试器（例如已配置的 CodeLLDB 或 MSVC 调试环境），在 examples/debug_probe.rs 的函数体设断点，观察变量、调用栈，再尝试单步；本阶段 Windows 自动验收不等于 IDE 真人实测。CI Ubuntu 专项运行真实 GDB batch，保存 stdout/stderr。

## 五、实际 Release 测量

cargo run --locked --offline --release --example measure -- 1000。相同 4096 字节输入、同一阈值，预热后各 5 次测量，black_box 降低编译器直接消除计算的可能；用中位数和计数结果证明两种实现在测量中都实际运行。改变轮数和输入大小、交替测量顺序，重复运行。时间受机器、缓存、优化、调度影响，不设“必须更快”阈值；这是基础微基准，不是 Criterion 统计推断或系统 profiler。学员记录原始数字，不编造性能收益。

## 六、验收与后续

维护者在平台根目录运行 python scripts/modern_d11a_check.py --output d11a-evidence；Ubuntu 有 GDB 时增加 --gdb。所有答案注入、格式化、mutant 和构建在临时副本，不修改教材；保存真实运行日志、文件哈希与个人记录模板。普通学员无需 Python。D11-A 首次 CI 待核验；D11-B 独立补真实 Miri、libFuzzer、RustSec 审计及其固定工具链/数据库证据。IDE GUI 与真人理解仍待人工验收，网页工作台未接入。

参考：[rustfmt](https://github.com/rust-lang/rustfmt) · [Clippy](https://doc.rust-lang.org/clippy/) · [Cargo profiles](https://doc.rust-lang.org/cargo/reference/profiles.html) · [GDB](https://sourceware.org/gdb/current/onlinedocs/gdb.html) · [black_box](https://doc.rust-lang.org/std/hint/fn.black_box.html)。
