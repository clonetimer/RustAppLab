# Rust AppLab 0.17.0-rc.2 候选验收状态

**当前结论：NOT_QUALIFIED。** `rc.2` 是候选编号，不是 RC 合格声明。

## 当前源码指纹与工程结果

- `source_id`: `4a3bcddfc6bf23f5915a1554a40f85ed1d6f77dc1aa540a38f4f66fe8e74e580`
- Linux x86_64 / Rust 1.99.1：本机工程验收 17/17 步骤通过；Debug 284 项、Release 274 项、文档测试 2 项通过；15 项学员挑战仍按设计忽略。
- 最终源码 ZIP 的独立解压验收：CRC/清单校验、Debug 284、Release 274、文档测试 2 项均通过；报告另附 ZIP SHA-256，且提取前后源码指纹一致。
- 用户已确认 0.17.0-rc.1 在其实际 Windows 环境可用。这是用户报告的功能 smoke，不等于 rc.2 的完整 17 包原生工程验收。

## 浏览器门槛

当前 Linux 容器真实 Chromium 对 LocalHub 与 ServiceDesk 的 `http://127.0.0.1:<port>/` 导航均返回 `ERR_BLOCKED_BY_ADMINISTRATOR`。当前证据见 `rc2/browser-linux.json`。没有尝试绕过浏览器策略，也没有用 mock fetch、DOM 注入或 `set_content` 冒充真实浏览器端到端通过。

新增 `VERIFY_BROWSER_WINDOWS.cmd` 供维护者在实际 Windows Chrome/Edge/Playwright Chromium 环境运行真实导航、搜索、创建、关闭、服务重启恢复与移动视口检查。若通过，报告中的 `source_id` 必须与本文件指纹相同；若管理员策略阻断，状态必须保持 BLOCKED。

## RC 与 Stable 的剩余门槛

RC_READY 需要当前源码绑定的本机工程、最终归档和真实浏览器证据全部通过。当前 Linux 浏览器门槛仍阻塞。

Stable 额外要求：真实 Rust 1.74.0（当前声明）构建和功能测试、macOS 原生运行、当前版本真实学员关键任务观察。不得把用户报告的 Windows 可用性或自动化测试数量扩大解释为这些门槛通过。
