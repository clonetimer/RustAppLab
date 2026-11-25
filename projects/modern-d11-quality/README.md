# D11-A · Rust 工程质量实验室

先完成 D8、D9 基础，再读 [D11-A 中文讲义](../../course/BOOK_D11_QUALITY.md)。不改变经典路线 MSRV。

- learner：无答案项目，复制到自己的目录；Rust 1.100.0、rustfmt、Clippy，安装后可离线。
- reference：源码包的答案；不进入学员包。
- 四步实现一款教学用帧编解码器及两种计数实现，配固定测试、可复现生成输入、单比特故障注入、CLI 和实际 Release 性能测量。
- tools-practice：个人副本真实格式化与 Clippy 红/绿实验。
- debug_probe：真实断点、局部/参数、调用栈；CI 在 Ubuntu 上使用 GDB 批处理；Windows IDE 实际操作需学员自己验证。
- D11-B 的 Miri、libFuzzer、RustSec 漏洞数据库和更深入的 unsafe 验证是独立选修，未包含在本课。

XOR 只是教学用校验，不能当作 CRC、加密校验或生产协议。不需 Python 即可做普通学员练习；维护者的独立重放脚本另需 Python 3.11+。
