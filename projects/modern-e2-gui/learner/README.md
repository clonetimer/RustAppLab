# 我的 SignalDesk 桌面应用

安装 Rust 1.100.0 和本机原生构建工具；Linux 需 X11/OpenGL、Wayland client 开发库、xkbcommon 与 fontconfig（如 libwayland-dev、libxkbcommon-dev、libfontconfig1-dev；完整包列表见 course/BOOK_E2_GUI.md），中文需可用 CJK 系统字体。取得通过验收的 E2-learner-offline.zip，解压新目录，不覆盖个人代码。先完成 src/lib.rs TODO(1)–(3) 和 src/gui.rs TODO(4)，每步运行 cargo test --frozen --test contracts step1_（替换编号），最终 cargo test --frozen --all-targets。初始四项合同应失败。

启动真实应用：cargo run --frozen --release（默认中文），或 cargo run --frozen --release -- --english。示例 8 行、阈值 30、告警 3；通过滑块和应用阈值、只看告警/全部数据、重置示例操作。Ctrl+F 切换筛选，Ctrl+R 重置，Ctrl+L 切换语言，Ctrl+Q 退出；不是测试专用命令。

导入自己的小型 UTF-8 文件，第一行 time_s,value，每行两个数值（时间是递增非负整数，数值有限且 [-1000,1000]，最多 4096 行）。不支持带引号的通用 CSV 语法。输入错误须保留旧数据。导出只保存当前可见行到不存在的新路径，拒绝覆盖；若新文件写失败可能留下部分文件，界面会报告错误。

修改阈值并解释图、统计及表格的关联，测试错误文件、中文路径、导出再导入及原始文件不变，填写 LEARNING_LOG.md。不要删除测试或只打印预期文字。完整讲义见 course/BOOK_E2_GUI.md。
