# Miri 专项（Linux）

安装 rustup toolchain install nightly-2026-11-17 --profile minimal --component miri --component rust-src 。在完成的独立 learner 副本中执行 cargo +nightly-2026-11-17 miri setup，再执行 cargo +nightly-2026-11-17 miri test --locked --offline。

probes 下的两份源码刻意包含 UB，**仅在 Miri 中解释执行**。将一份复制到个人练习副本 examples/miri_probe.rs；执行 cargo +nightly-2026-11-17 miri run --locked --offline --example miri_probe，阅读真实 Undefined Behavior 诊断、位置与原因。修复为安全访问或先正确初始化，再用同一 Miri 命令重放。每次移除个人副本中的 probe 后再继续一般测试。不得 cargo run、rustc 直接执行这些故意错误的程序。

通过 Miri 仅证明此次被解释执行的路径没有发现其检测范围内的 UB，不构成所有输入、平台或 FFI 的安全证明。
