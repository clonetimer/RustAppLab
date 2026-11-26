# E2 · Rust 原生桌面 GUI：SignalDesk

[现代化路线](MODERN_RUST_ROADMAP.md) · [教学工程](../projects/modern-e2-gui/README.md)

目标是完成一款可实际打开、操作和保存数据的 Rust 桌面软件，而不只学习 GUI API。使用真实 eframe/egui 0.37.0 与 Glow/OpenGL 后端，固定 Rust 1.100.0（上游 eframe MSRV 1.95）。0.37 的 App::ui 使用 &mut egui::Ui，不能照搬旧 App::update 教程。保持经典路线和网络课程独立。

| 学习任务 | 学员实际编程 | 验收 |
|---|---|---|
| 1 parse_samples | 解析限定数值 CSV，控制文件大小和行数 | 错误头、非法数值/时间、重复时间、NaN、Inf、超限 |
| 2 summarize | 计算计数/均值/最大最小、阈值告警 | 实际输入、多阈值、空数据和非法值 |
| 3 Model::apply | 显式 Action 和失败不改状态 | 筛选、阈值、重置、双语，保留失败前数据 |
| 4 gui::controls | 使用真实 slider/button 产生 Action | egui_kittest 点击实际控件并观察事件；不是源字符串匹配 |
| 5 综合 | 固定 GUI 渲染、折线/表格、文件 I/O | 真实中文路径导入、筛选导出、拒绝覆盖、错误保留数据 |

复制通过验收的 E2-learner-offline.zip 到个人新目录。执行 cargo test --frozen --test contracts step1_（替换编号）；最后 cargo test --frozen --all-targets、cargo run --frozen --release。使用界面选择中文或 English。8 行演示数据、阈值 30，3 个告警（包含 >=）。观察筛选后表格和导出的真实行数，不把筛选误当删除；折线展示整个信号时间轴，表格/导出仅展示可见记录。Ctrl+F 筛选，Ctrl+R 重置，Ctrl+L 切换语言，Ctrl+Q 关闭，按钮与快捷键共享相同状态转换。中文依赖系统字体；若缺字形，安装本机 CJK 字体或启动 --english。

## 架构与边界

src/lib.rs 管理数据与显式状态，不依赖 OS 窗口；src/gui.rs 将状态映射到真实控件并发送 Action；图形由 egui painter 绘制真实样本位置。src/main.rs 使用 eframe 原生事件循环。读文件上限 256 KiB，最多 4096 样本；只支持 UTF-8 两列数值，不支持通用引号 CSV。数值有限且在 [-1000,1000]；时间严格递增。导入失败保留旧状态；导出 create_new，拒绝覆盖，失败可能留下部分新文件。同步本地 I/O 因输入有界用于教学，不能承诺网络盘低延迟或完整后台任务调度。无需网络服务、无需真实遥测，不是生产监测/安全告警。

## 实际验收

独立 CI 真实解析新依赖，固定 RustSec DB 审计漏洞/unsound，保存原始锁和 SHA-256；两平台消费同一锁与 vendor。实际 Debug/Release 合同、逐步 RED/GREEN、搬移空缓存学员包。Ubuntu 以 Xvfb + Mesa 启动真正的原生窗口，使用 xdotool 原生键盘事件触发筛选及语言切换，检查窗口状态标题并捕获三个实际截图（初始/筛选/中文）；不使用模拟截图，不宣称实机 Windows GUI 已验证。Win/Linux 共同执行真实 egui_kittest 控件点击。普通 learner-kit 不含答案，独立 ZIP 提供依赖。当前尚待 CI 与锁冻结；真人学习效果、网页工作台入口、Windows 桌面实际操作仍需验收。

参考：[eframe 0.37.0](https://docs.rs/eframe/0.37.0/eframe/) · [egui_kittest](https://docs.rs/egui_kittest/0.37.0/egui_kittest/) · [官方原生窗口示例](https://github.com/emilk/egui/blob/0.37.0/examples/hello_world/src/main.rs)。
