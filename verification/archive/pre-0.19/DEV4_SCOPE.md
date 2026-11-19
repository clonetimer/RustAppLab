# 0.18.0-dev.4 验证范围

本版是独立教学开发版，保留 0.17.0-rc.2 冻结基线，不继承其 RC 或 MSRV 资格。声明支持 Windows x64 MSVC、Linux x86_64，不声明 macOS。

本轮 Linux x86_64 / Rust 1.99.1 全仓单次 Debug 320 项、Release 310 项通过，各 15 项既有挑战跳过；文档测试 2 项；Python 工具测试 25 项；格式与 Debug/Release 严格 Clippy、Release 构建通过。新四作品 CLI 分步回归包含在上述总数内。

四应用联合演练是真实执行：14 次 CLI 调用、6 次直接 HTTP 交互、2 次服务启动，核对 JSON 内容、关闭状态、重启恢复、字节与行数、索引与禁止覆盖。这是维护者参考程序演练，不是真人学员成绩，也不证明学员代码采用了并发策略。

新任务卡与联合讲义完成桌面/移动 DOM 演练。环境策略拒绝 file:// 页面导航（ERR_BLOCKED_BY_ADMINISTRATOR），DOM 测试使用 set_content；不能宣称原生浏览器本地文件访问通过。新版本 Windows、Rust 1.74 与真人学习效果尚无本轮实测结果。

最终压缩包的全新解压复验、完整原始日志和源文件差异清单随外部 evidence/交付报告发布，以实际结果为准。不可把历史 verification 文件当作本版新验证。
