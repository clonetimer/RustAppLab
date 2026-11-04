# Rust AppLab 0.19.0-rc.1 · 支持平台与环境

| 内容 | Windows x64 MSVC | Linux x86_64 |
|---|---|---|
| 基础平台、进阶A、B1–B4 | Rust≥1.74，MSVC C++构建工具及Windows SDK | Rust≥1.74，C编译/链接工具 |
| C1/C2独立工作区 | Rust1.85.0，锁定依赖 | Rust1.85.0，锁定依赖 |
| 参考运行包 | 当前Windows runner实际构建、解压运行 | 当前Linux runner实际构建、解压运行；glibc/系统库要求见BUILD_INFO.json |
| 浏览器 | Chromium自动化为工程验证；Win10/11用户体验另记 | Chromium实际页面/API流程 |
| SQLite | StoreDesk依赖Windows系统SQLite接口，见专项说明 | StoreDesk编译需SQLite开发库，运行需libsqlite3.so.0 |

不声明macOS、ARM或32位支持。Windows CI来自Windows Server，不等同用户的Win10/11真机安装。普通学员不需Python/Playwright；维护者回归与交付脚本需要Python3.11及以上。

运行包可直接体验参考软件，但LabQuest编译/检查学员源码仍需要Rust工具链。C1/C2的vendor包仅含依赖源码，不替代编译器。详见 [开始指南](README_START_HERE.md)、[SQLite说明](course/SQLITE_SETUP.md) 和 [候选发布规则](RELEASE_POLICY.md)。

原同步NetFetch/ServiceDesk和教学执行器AsyncBoard仍保留；C1/C2是独立的真实Tokio/Reqwest/Axum实现。网络均限定可信本机回环使用，不提供公网/TLS或账号体系。
