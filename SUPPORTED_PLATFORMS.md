# Rust AppLab 正式支持平台与验证范围

**本项目现将发布支持范围限定为 Windows x64（MSVC）和 Linux x86_64。macOS 不在本项目的正式支持声明中，也不作为后续 Stable 资格阻塞项。** 这不等于声称程序在 macOS 必然不能编译，只是不提供兼容性承诺或原生测试资格。

| 内容 | Windows x64 MSVC | Linux x86_64 | macOS |
|---|---|---|---|
| 冻结 RC 0.17.0-rc.2 工程 | 原生通过（用户提供完整工程证据） | 原生通过 | 不声明支持 |
| 冻结 RC 浏览器交互 | Chrome/Edge 实机 E2E 通过 | Chromium E2E 另有受环境策略阻断记录；Rust HTTP/CLI 通过 | 不声明支持 |
| 冻结 RC Rust 1.74.0 | 实测 271 Debug / 261 Release、15 个学员工程 | Rust 1.99.1 工程通过；未单独提供 Linux 1.74 全矩阵 | 不声明支持 |
| 当前 0.19.0-dev.2 工程 | 以当轮 Windows CI / 原生记录为准 | 以当轮 Linux / MSRV 记录为准 | 不声明支持 |

当前源码的具体测试提交、命令与结果见 [2026-11-02 测试记录](verification/2026-11-02-book-guided-quality.md)。进阶 B 独立参考工作区现已加入 Windows/Linux 工程脚本和 Rust 1.74 测试计划。

支持边界：StoreDesk 依赖系统 SQLite 开发/运行库；浏览器应用只能绑定 localhost；用户自写 Rust 代码评测不构成不可信代码隔离沙箱。未完成的 Tokio/Reqwest/Axum、桌面 GUI、WASM 课程不计入已经交付的课程范围。

**关于 Stable：** 本项目不再要求 macOS 原生测试。Windows/Linux 两个声明平台的新版本工程记录、对现有业务功能的回归、真实学员对关键教学路线的观察，以及交付物一致性，仍属于发布资格判断内容。RC_READY 只针对已冻结的 0.17.0-rc.2，不自动继承给后续开发版本。
