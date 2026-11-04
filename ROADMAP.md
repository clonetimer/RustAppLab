# Rust AppLab 0.19.0-rc.1 · 本轮收尾与后续范围

## 已完成的功能范围

基础17包、12款Studio与FirstCard、TaskPocket/PageMint独立作品；进阶A；B1–B3多程序复用；B4 Serde JSON导出；C1 Tokio/Reqwest异步下载；C2 Axum工单服务。

C2已包含持久化、真实并发客户端验证和优雅关闭，不再另建一门内容重复的C3。本轮以C2为功能边界，候选发布只整理文档、使用入口、交付包和证据。

## 接下来怎样推进

先按 [开始指南](README_START_HERE.md) 使用，在 [试用记录](course/USER_TRIAL.md) 保存真实体验。只处理阻断、错误提示、教学衔接和兼容性问题；没有实际问题的模块保持现状。

GUI、WASM、公网/TLS、重试和其他框架放入可选方向。只有产生明确学习需求时重新评估，不作为当前RC或Stable的功能欠账。

Stable仍需真人学习记录及当前版本工程准入；不以新增功能替代验收。详见 [发布规则](RELEASE_POLICY.md)。
