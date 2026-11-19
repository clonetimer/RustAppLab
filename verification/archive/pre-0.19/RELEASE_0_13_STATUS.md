# Rust AppLab 0.13.0 交接状态

已完成两款应用、两篇讲义、六个新里程碑，Studio现为八款24里程碑。全Workspace共13包，主线材料14/20，学员包11工程。新应用源文件apps/termgarden与apps/storedesk；生成器apps/labquest/src/studio.rs；Windows续学scripts/studio_windows.ps1；两个模板及提示在apps/labquest/studio。

Linux Rust1.99.1验证：Debug206/Release196、失败0、忽略15；格式/双配置严格Clippy通过。SQLite独立Python驱动与真实CLI15次通过，旧六款Release CLI14次通过。详情见本目录日志与JSON。别重做已完成作品或宣布Windows已通过；下一重点是菜单7/8原生验收和NetFetch/ServiceDesk生态课程。

实际范围偏差：TermGarden为按行Enter交互而非raw-mode TUI；StoreDesk使用系统SQLite的有限FFI适配器，不是成熟crate及自动迁移课程。环境无法下载crate发行包，但没有交付不可构建的假依赖。后续要补成熟生态，不能将零crate作为硬限制。

Windows运行说明：完整源码解压到新目录，保留旧练习，CONTINUE_WINDOWS.cmd菜单7/8。SQLite预检失败先看course/SQLITE_SETUP.md。仅0.11.1曾由用户反馈通过；本轮Windows/macOS/MSRV/真人教学效果仍待证据。

构建、学员包和Linux包脚本在scripts。不要把target、个人任务文件或个人数据库收进源码包。SOURCE_MANIFEST由package.py重新生成。发布包的全新解压报告单独交付，避免把压缩包自身哈希循环写进被压缩文件。
