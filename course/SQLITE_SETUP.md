# StoreDesk 原生 SQLite 环境说明

## 先区分两件事

Rust 能编译 UnitLab，不代表系统已有 SQLite 链接库。StoreDesk 是第一款使用本机 C 数据库库的课程，仍然没有外部 Rust crate；这不等于“没有外部运行依赖”。本版本不联网下载任何依赖，不附带或替换系统 DLL。

在项目根目录执行 `cargo run --offline -p storedesk -- doctor`。成功后再创建学员工程。它输出真正的运行时 SQLite 版本。

## Windows 10/11 x64 + MSVC

本包通过 `#[link(name="winsqlite3")]` 使用系统 `winsqlite3.dll` 及 Windows SDK 的导入库。微软资料列出了 Windows 内置 SQLite API；但 API 存在不等于本包已在你的 Windows 环境通过。

已安装 Rust 的学员先运行 `rustc -vV`，检查 host 是否为 `x86_64-pc-windows-msvc`。遇到 `LNK1104: cannot open file 'winsqlite3.lib'`，进入 Visual Studio Installer 的“修改”，确认 C++ 桌面开发和 Windows 10 或 11 SDK 组件存在；随后新开终端再试 doctor。不需要为了这条错误卸载 Rust。

本轮没有原生 Windows 执行证据。GNU/MinGW 工具链不是当前 Windows 教学路径；不要从随机网站下载所谓 sqlite DLL 塞到系统目录。

## Linux

需要能被链接器找到的 `libsqlite3` 开发库，以及能被运行时加载的 SQLite 共享库。在 Debian/Ubuntu 类发行版，开发包通常名为 `libsqlite3-dev`；由机器维护者使用发行版包管理器安装。其他发行版包名可能不同。当前 Linux 验证环境的该库已存在，没有在学员工程中附带系统库。

`cannot find -lsqlite3` 通常是开发链接文件缺少；程序启动时报 `libsqlite3.so.0` 则是运行环境问题。不要把同一平台上的编译通过扩大为任意发行版都能运行。

## macOS

使用系统 SQLite 与命令行开发工具；仍需原生构建复验。若链接器或 SDK 路径异常，先修复 Command Line Tools 环境。不能把 Linux 检查当成 macOS 通过。

## 教师与发布者

`src/sqlite.rs` 是本项目有限的教学适配器：使用准备语句和参数绑定，连接与语句通过 Drop 释放，事务通过守卫回滚。不暴露裸指针给学员业务函数，没有自定义数据库引擎。无需让基础薄弱的学员先学习 unsafe/FFI。

当前进阶目标是 SQL 与事务。优先选用成熟库通常比维护自有驱动更适合一般生产项目；后续生态课程应补 rusqlite/SQLx、Cargo.lock、依赖缓存和跨平台构建，不以本轮零 Rust crate 依赖为长期限制。系统库没有随源码固化，所以引擎版本可不同；本轮证据必须记录实际运行版本。

只用自己的教学数据库。没有完成安全审计、断电与磁盘故障测试；本包不为高风险数据提供生产保证。

## 可核对的官方资料

- Microsoft Windows SQLite API 清单：https://learn.microsoft.com/zh-cn/uwp/win32-and-com/win32-extension-apis
- SQLite 准备语句：https://www.sqlite.org/c3ref/stmt.html
- SQLite 参数与缓冲生命周期：https://www.sqlite.org/c3ref/bind_blob.html
- SQLite 连接关闭：https://www.sqlite.org/c3ref/close.html
- SQLite 事务：https://www.sqlite.org/lang_transaction.html
- rusqlite 官方项目，供后续生态课程比较：https://github.com/rusqlite/rusqlite
