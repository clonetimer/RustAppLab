# Rust AppLab 0.19.0-rc.1 · 候选发布与Stable准入

## 固定本轮范围

基础Studio、独立作品、进阶A、B1–B4、C1及C2为当前交付范围。Windows x64 MSVC与Linux x86_64为目标平台；macOS不作支持声明。候选版不以继续增加C3/C4/GUI/WASM作为发布前提。

## 三种状态分别记录

**开发检查点**：同一提交的现有CI通过后创建 `checkpoint/v<VERSION>-<SHA>`。它既不是正式产品版本，也不说明真人学习效果。

**RC候选版**：`0.19.0-rc.1`作为GitHub prerelease发布，不标记Latest或Stable。除了现有Linux/Windows、Rust1.74、真实浏览器、C1/C2全部准入，还要求源码归档新目录重放、首次学习路径检查、对应平台运行包与文件校验。缺少这些结果时只能保留候选源码，不能宣称已完成RC准入。`scripts/release_decision.py`的历史格式保持兼容；新的候选汇总同时核验C1/C2与交付证据，避免只凭旧判定器忽略新增课程。

**Stable**：保留上述工程证据，还需要当前版本实际学员在FirstCard、UnitLab和独立作品路线的真人观察、提示使用与未解决阻断处理记录。自动化修复TODO用于验证教学工具，不是一个真实学员。未完成的观察使用 [试用模板](course/USER_TRIAL.md) 留空，不编造通过结果。

## 证据与发布原则

每份结果绑定版本、source_id及执行前后源码指纹。源码、课程、交付工具改变后重新验收；同一候选版本不混用不同源码日志。保留原始命令输出、压缩包SHA-256、构建环境和Git提交。候选汇总与发布资产在完整CI成功后生成；不覆盖已有同名版本资产或挪动既有标签。

源码归档和Windows/Linux运行包分别验证；源码可复现不等于不同操作系统二进制应该相同。运行包的构建平台与编译器版本以包内 `BUILD_INFO.json` 为准。Windows Server CI不等于用户本机Win10实测。

## 历史与边界

冻结 `0.17.0-rc.2` 的RC_READY和原始证据保持原样，只证明当时源码；不继承到本版本。历史材料见 `verification/` 与 [变更记录](RELEASE_NOTES.md)。

启动C2时同步校验已有日志；请求期磁盘工作在独立阻塞线程执行。回环HTTP不作公网服务、账号认证或数据安全沙箱；StoreDesk依赖系统SQLite。系统支持详情见 [SUPPORTED_PLATFORMS](SUPPORTED_PLATFORMS.md)。
