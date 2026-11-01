# 0.17.0-rc.1 验收状态

**结论：候选源码和 Linux 工程验收已完成；没有签发完整 RC 合格或 Stable。**
`rc.1` 是被冻结的候选包编号，不是已通过全门槛的别名。真实浏览器仍被环境策略阻断，保留为 RC 阻塞，不改为“可选”。

## 已执行的验收

环境：Linux x86_64 / glibc 2.41；实际 rustc 1.99.1、cargo 1.99.1。Cargo 使用 locked/offline；没有把最新编译器结果当作 Rust 1.74 验证。
当前代码指纹（说明文档和结果不进入这个代码指纹，完整分发字节另由 SOURCE_MANIFEST 校验）：
`61d865e5df90f5921c42f1edd542e04d7699de39b74bb9ac84c64cb9bfc4b89a`

| 验收项 | 实测结论 |
|---|---|
| 未修改 0.16.0 基线 Release 全仓单次测试 | 265 passed，15 ignored；旧超时没有复现 |
| 当前 Debug 全仓单次测试 | 284 passed，0 failed，15 ignored |
| 当前 Release 全仓单次测试 | 274 passed，0 failed，15 ignored |
| Rust 文档测试 | 2 passed；包括预期无法编译的借用反例 |
| 发布工具 Python 回归 | 13 passed；包括缺证据不能晋级、启动器控制字符/目标路径检测 |
| Rustfmt、两配置严格 Clippy、Release 构建 | 全部通过 |
| 实际应用冒烟 | Studio、终端/SQLite、MiniKV、本机 HTTP、AsyncBoard、SafeSync SHA-256 均通过 |
| 学员包独立编译 | 15/15 工程通过（Linux） |
| 未完成学员模板的第一阶段 | 12/12 正确拒绝；不是 12 个已完成作品成绩 |
| Linux 包解压与独立启动 | 17/17 程序 help 成功、内置二进制哈希一致 |
| 真实浏览器 | 两款网页都遭 ERR_BLOCKED_BY_ADMINISTRATOR；未通过，不作 DOM 模拟替代 |
| Rust 1.74 | 当前实际编译器不是 1.74.0，MSRV 报告明确 BLOCKED_ENVIRONMENT |
| Windows/macOS、真实学员 | 缺当前版本原生/学习证据；没有计为通过 |

15 个 ignored 是原有待学员实现的挑战，不计入产品通过数。Release 比 Debug 少 10 项，原因是原有 Debug 专用故障注入条件编译。

最终源码 ZIP 的全新目录编译与全量 Debug/Release 结果在同批交付的外部 `RustAppLab-0.17.0-rc.1-FRESH_ARCHIVE_CHECK.json` 中记录，并绑定该 ZIP 的 SHA-256；不能用本段说明替代实际复验。

## 可审查的原始证据

[工程汇总](rc/native-linux/engineering.json) · [Debug 完整日志](rc/native-linux/debug-tests.txt) · [Release 完整日志](rc/native-linux/release-tests.txt) · [工具测试](rc/native-linux/tooling-tests.txt)

[浏览器真实阻断](rc/browser/browser.json) · [MSRV 缺口](rc/msrv/msrv.json) · [学员与程序包核对](rc/delivery.json)

工程报告逐步记录命令、退出码、耗时和原始日志 SHA-256，前后代码指纹相同。HTML/JSON 成功的直接 HTTP 请求没有被用来冒充浏览器表单通过。

## 本轮修复和失败记录

新增统一子进程输出捕获，避免等待退出后再读管道导致大量输出阻塞；有超时、输出上限、关闭 stdin、退出后清理和编译目标隔离测试。这一风险有反例证明，但不是已查明的所有历史 Release 超时根因。
第一次新实现全仓验证发现超时提示少了已有的“无限循环”指引，原有断言拦下该回归；恢复提示后重新完整执行，未删除或放宽该 Rust 测试。
静态检查又发现新 Windows `.cmd` 中 `\verify` 被字符串转义为控制字符，现已修正，并加入脚本字节和目标存在性回归。随后重新完整验收；静态通过仍不能代替 Windows 原生运行。
完整附带证据包保留失败尝试与最终成功记录，按指纹区分；不拼接不同源码的通过数。

## 下一次验收如何使用

维护者在 Windows 双击根目录 `VERIFY_RELEASE_WINDOWS.cmd`，在相邻新目录获得日志/汇总，只有全工程成功后才复制本机 `.exe`。不要求学员运行该发布验收，不覆盖他们自己的作品。
真实浏览器另运行 `scripts/browser_e2e.py`，需要维护者环境的 Playwright；不得删掉真实表单或重启恢复检查来消除阻塞。
[发布规则](../RELEASE_POLICY.md)保留 RC/Stable 分层要求；[已知缺口](../KNOWN_ISSUES.md)仍保留未交付的异步框架、聊天、桌面、WASM 课程。
