# 现代 Rust 选修课程地图

先完成基础作品，再按兴趣选择专题。不必一次学完所有进阶课程。经典课程维持 Edition 2021 / Rust 1.74 最低支持版本；C1/C2 使用 Rust 1.85.0，现代课程使用独立 Edition 2024 / Rust 1.100.0 工具链。网页操作与准备方式见完整源码包的 course/WORKBENCH.md。

| 课程 | 学习内容 | 入口 |
|---|---|---|
| D8 | Edition 2024 与迁移 | [讲义](BOOK_D8_EDITION_2024.md) |
| D9-A | 现代稳定语言特性 | [讲义](BOOK_D9_STABLE_FEATURES.md) |
| D9-B | 标准库与 Cargo | [讲义](BOOK_D9_STDLIB_CARGO.md) |
| D10-A | Future、Pin、Waker、取消与异步原理 | [讲义](BOOK_D10_ASYNC_CORE.md) |
| D10-B | Tokio、Stream 与异步工程 | [讲义](BOOK_D10_TOKIO_STREAM.md) |
| D11-A | 工程质量、诊断与工具 | [讲义](BOOK_D11_QUALITY.md) |
| D11-B | Miri、模糊测试、依赖审计 | [讲义](BOOK_D11_SAFETY.md) |
| E1-A | HTTPS 与授权 | [讲义](BOOK_E1_TLS_AUTH.md) |
| E1-B | SQLite 事务与观测 | [讲义](BOOK_E1_SQLITE.md) |
| E2 | 原生桌面 GUI | [讲义](BOOK_E2_GUI.md) |
| E3 | 嵌入式与实时任务 | [讲义](BOOK_E3_EMBEDDED.md) |
| E4 | C ABI 与 WebAssembly | [讲义](BOOK_E4_INTEROP.md) |
| E5 | 科学计算与机器学习 | [讲义](BOOK_E5_SCIENCE.md) |

D10-B、E1-A、E1-B、E2、E3 使用准备好的离线学员包；依赖与安全要求见 [网络依赖安全说明](NETWORK_SECURITY_BASELINE.md)。其余八门可在网页创建个人练习。GUI 原生窗口、ARM/QEMU、C/WASM、Miri/Fuzz 和 IDE 专项仍按对应讲义执行；网页 host 测试不替代这些操作。

可按问题查阅 [稳定特性索引](MODERN_STABLE_FEATURE_INDEX.md)。实际学会的能力以个人代码、实验与解释记录为依据，不以读完目录计算。
