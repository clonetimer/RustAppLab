# Rust AppLab rc.2 Windows 原生证据审计（2026-10-30）

**结论：Windows 原生工程 PASS；真实浏览器端到端 NOT RUN / 环境依赖缺失；不签发 RC_READY 或 Stable。**

- 已核对证据文件：`RustAppLab-native-20261030-004600-e27e33ed.zip`、`RustAppLab-browser-20261030-004526-a297ad16.zip`。
- 基线 `rc.2` 源码指纹：`4a3bcddfc6bf23f5915a1554a40f85ed1d6f77dc1aa540a38f4f66fe8e74e580`。
- 应用 Windows 验收热修复包 2 后的源码指纹：`dc7d325d2ce320dae9958116aeeb000f329eaccbe31c9d7c1f92b4d9f8e325a8`。
- Windows 工程报告首尾源码指纹与补丁版精确一致；不是混合不同 Rust 源码的结果。
- Windows `rustc 1.96.0` / `x86_64-pc-windows-msvc`，14 个工程步骤 PASS，Debug 271、Release 261，失败数均为 0，按设计忽略 15 项学员挑战。17 个 `.exe` 的内部 SHA-256 清单全部匹配。
- Rustfmt、Clippy、Debug/Release 测试、文档测试、Release 构建、入门、SQLite 与 Studio Windows 冒烟均通过。
- 浏览器压缩包仅含 `launcher.json` 和 Playwright 预检日志，结果为 `BLOCKED_ENVIRONMENT`：`ModuleNotFoundError: No module named 'playwright'`。**没有运行 Chrome/Edge 的真实页面交互。**
- 未完成：真实浏览器两个应用 PASS、macOS 原生、声明的最低 Rust 1.74、真实学员关键任务验收。

## 浏览器验收修复步骤（Windows PowerShell）

```powershell
py -3 -m pip install playwright
py -3 -c "from playwright.sync_api import sync_playwright; print('Playwright OK')"
```

如果 `py -3` 不存在，请用与浏览器验收启动脚本相同的 Python 3.10+ 解释器安装。Chrome/Edge 已安装时无需另装 Playwright Chromium；找不到浏览器时再运行 `py -3 -m playwright install chromium`。

随后从这份已合并热修复包的源码目录运行 `VERIFY_BROWSER_WINDOWS.cmd`，并上传新生成的 `RustAppLab-browser-*.zip`。此项目的浏览器测试要求两个页面都为 PASS、源码指纹与上文一致，不能把 preflight PASS 当作端到端 PASS。

## 文件一致性

本源码包在原始 `0.17.0-rc.2-source.zip` 的基础上，仅覆盖上面三个 Windows 验收脚本；Rust 应用代码和 Cargo 依赖未修改。`SOURCE_MANIFEST.sha256` 已更新为匹配实际文件字节。旧版资格判定文档保留为历史记录，不能将旧记录中的 NOT_QUALIFIED 直接改成 PASS。
