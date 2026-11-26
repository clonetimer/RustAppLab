# E2 · SignalDesk 原生桌面数据分析工具

先读 [中文课程](../../course/BOOK_E2_GUI.md)。真实 eframe/egui 0.37.0，Rust 1.100.0，Glow/OpenGL 原生后端；与已有 CLI/网页课程独立。不是网页套壳。支持中文/英文切换、数据曲线、阈值、告警筛选、导入与不覆盖导出。数据为本地简单数值 CSV；不连接设备或公网、不作真实安全告警。

learner 是四步 TODO，reference 是答案；同一测试检验数据、状态、真实 egui 控件及文件。首轮实际 Cargo/RustSec 原始锁已冻结，346 registry 包，SHA-256 `0c547514c781df6d8571cb8dac307c66d671e95d22734e1f3bd098b80b81b210`；后续只允许 locked fetch 与重审计。三项上游未维护警告保留，未宣称生产安全。独立学员 ZIP 包含冻结锁/vendor 与 TODO，普通 learner-kit 含模板和讲义。当前候选未验收，不能当成可发布包。

Windows/Linux 的 Rust 行为与 egui 控件合同分别执行。Ubuntu 原生桌面 Xvfb + Mesa 软件 OpenGL，实际窗口键盘交互并截图；不将其当 Windows 实机 GUI 证据。Linux 需 libwayland-dev、libxkbcommon-dev、libfontconfig1-dev 等开发库（详见中文讲义）。中文界面需系统 CJK 字体（Linux 例 fonts-noto-cjk）；无字体包分发。可用 --english 启动英文界面。
