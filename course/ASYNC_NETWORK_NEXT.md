# 异步网络课程进度与后续方向

C1 已提供 Tokio/Reqwest 回环 HTTP 下载、有限并发、超时、取消和暂存发布；C2 提供 Axum JSON 工单服务、共享状态、持久化单写者和优雅关闭。两者都有独立参考/学员工程、固定测试和离线依赖包。实际验收以对应提交的 CI 记录为准。

原 AsyncBoard 是教学执行器，NetFetch/ServiceDesk 同步课程继续保留，作为对照，不代替 Tokio 网络实现。参见 [C1 讲义](ADVANCED_C1.md)、[C2 讲义](ADVANCED_C2.md)。

后续工作聚焦公网/TLS、客户端重试与流式下载等扩展；当前 C2 不宣称生产级鉴权、断电原子性、多进程写入或强制关闭截止时间。Windows/Linux 是现阶段测试目标，macOS 仍需独立验证。
