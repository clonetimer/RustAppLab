# C1 重建：真实异步 HTTP 下载

基线：B4 main bf6bc6a3115f979526aa4c6ef9e4af61e0acfd01。用户明确要求从 C1 重新开始；旧本地提交和测试数不作为本次证据。

## 范围与方案

采用独立 projects/advanced-c1 工作区（Rust 1.85 / Edition 2021），根课程继续 Rust 1.74 / 无 registry 依赖。不改写原同步 NetFetch，也不将新依赖加入根工作区：前者便于同步/异步对照，后者保护基础离线体验。使用固定 Tokio 1.48.0、Reqwest 0.12.24；完整传递图由实际 Cargo 1.85 解析并锁定，不预设依赖数量。

C1 只连接数字 IP 的回环 HTTP，不解析域名，不跟随重定向，不使用系统代理。TLS、公网、Axum 和重试属于后续阶段。运行 download 与自包含 demo；学员分三步实现 validate_jobs、fetch_one、download_batch，配合同一套固定行为测试。

## 接口与行为

Job { name, url }；Limits { concurrency, connect_timeout, total_timeout, idle_timeout, max_bytes }；Receipt { name, bytes }。公开函数 validate_jobs(&[Job], Limits)、fetch_one(Job, Limits, watch::Receiver<bool>)、download_batch(Vec<Job>, &Path, Limits, watch::Receiver<bool>)。

1–64 个任务，1–8 路并发，单正文 1–8 MiB，三类超时均大于零且最多60秒。名称为单个跨平台安全文件名，拒绝保留设备名、大小写不敏感重复、路径穿越。输出顺序按任务输入顺序，不受完成顺序影响。正文按 chunk 检查上限，不相信 Content-Length。

下载到新建同级暂存目录，全部成功才发布新目标目录；失败/取消/调用方丢弃 future 清理本次暂存数据。网络子任务只返回有界内存正文，不持有文件写入句柄；控制器写盘，取消时 abort 并 join 全部任务。禁止覆盖已有目标。教学场景使用可信本地父目录、单写者；不承诺敌对目录竞争、断电原子性、远端服务停止执行请求。文件写入是有界同步本地 I/O，不冒充异步文件系统课程。

## 平台与分发

LabQuest advanced-c1 start/prepare/check/next/lesson：显式准备固定参考依赖，评测复制学员 src 但固定 Cargo 清单/锁/测试。读取进度不运行程序，源码变更使成绩过期。首次准备联网，后续 locked/offline；vendor 包通过空缓存和移动目录验证，保留许可证。源码、锁文件、教材及验收记录及时推送 feature 分支。

## 验证

先运行可编译 TODO 的红测试，再实现；真实 loopback 服务器证明并发而非只看总用时。覆盖响应头/正文停顿、分块超限、错误状态、拒绝重定向、取消前后、掉 sender、已有输出、失败清理、确定排序和 Windows 路径。Linux/Windows stable、实际1.85、基础1.74与已有浏览器回归。全部所需结果确认后合并并打阶段标签，不发布 Stable。

资料：https://rust.bootcss.com/ch11-03-test-organization.html；https://docs.rs/tokio/1.48.0/tokio/task/struct.JoinSet.html；https://github.com/seanmonstar/reqwest/tree/v0.12.24；https://doc.rust-lang.org/cargo/commands/cargo-vendor.html。
