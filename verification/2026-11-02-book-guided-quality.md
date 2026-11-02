# RustAppLab 对照阅读与工程验证记录

日期：2026-11-02。基线：`d31f9fca5de8bfa5fb564039f72ed6557c31524e`。

本记录区分基线、实现集成检查和审查后的追加回归。各平台的最终状态以对应提交的 [GitHub Actions](https://github.com/clonetimer/RustAppLab/actions) 为准，不用其他平台或旧提交的通过记录代替。

## 已复现的问题

- Windows CI 的 rustc UTF-8 管道输出被 Python 按 cp1252 解码，导致错误实验步骤失败。
- 进阶 B 的错误 `list` 输出和缺失 strict-titles 校验可被旧 B3 检查误判通过。
- 初始 B1 失败时导航显示不存在的 B1..B0 已通过区间。
- 独立参考 Workspace 没有进入根 Workspace 的直接测试计划，源码指纹未覆盖 `projects/advanced-b`。
- 旧 Report v1 的源码指纹即使有效，也不能证明它通过了加强后的 B3 合同；导航现在要求重新累计检查，新报告使用 v2，工程标记仍保持 v1。

本地首次默认沙箱检查因 localhost socket 权限限制失败；该记录用于确认执行环境限制，未计作产品通过证据。实际网络回归在允许本机端口的执行环境及 CI 中运行。

## 已执行的本地验证

环境：Linux x86_64，Rust 1.99.1，Python 3.12.14；SQLite 开发库可用。

| 检查范围 | 结果 |
| --- | --- |
| 原始基线完整工程检查 | 17/17 步骤通过；Debug 369、Release 359、doctest 9 |
| 本轮实现集成检查 | 27/27 步骤通过；Debug 378、Release 368、根 Workspace doctest 9 |
| 独立进阶 B 参考 Workspace | 默认/严格 feature、Debug/Release、无默认 feature：每组 13 个普通测试通过；默认/严格 doctest：每组 1 个通过 |
| Python 工具回归 | 52 个测试通过，覆盖 UTF-8 解码、诊断行匹配、源码指纹和测试证据规则 |
| 真实错误实验 | 4 个指定编译错误及其 4 个修复例通过；另验证 cp1252 输出环境 |
| 阅读导航 | 5 个端到端测试通过，覆盖 17 类工程、中文与空格路径、HTML 转义及只读行为 |
| 审查后 Report v2 回归 | 9 个进阶 B 端到端测试通过，含有效源码指纹的旧 v1 记录仍须重新检查 |
| 阅读外链 | 本轮检查的 22 个链接均返回 HTTP 200；这是检查时状态，不保证网站后续不变 |

Debug/Release 均保留原有 15 个有意忽略的学员挑战测试；普通测试的通过数不包含它们。进阶 B 固定合同要求零 ignored。Debug 与 Release 数量不同来自既有按构建模式启用的测试。

27 步实现集成检查对应源码指纹 `77ca52102115624a4921949da615c1c58d575a61442e7bc4aaaf91383073ad18`，发生在最后的 Report v2 兼容修复之前；该修复由追加端到端测试与独立复审覆盖，最终提交还须由 CI 执行完整矩阵。独立复审发现的 1 项重要问题和 2 项小问题均已解决。

Rust 1.74.0 的首次本地全量运行在 328 个测试通过后遇到生成的测试二进制缺少执行位（EACCES），没有记为通过；保留失败证据并在独立副本清理重建后复验。MSRV、Windows 原生 PowerShell 和浏览器的最终判断请查看本次提交的相应 CI job。

## 可复现命令

```sh
python scripts/check_error_labs.py
python -m unittest discover -s scripts/tests
python scripts/release_check.py --output native-evidence
# 必须在实际安装的 Rust 1.74.0 下运行：
RUSTUP_TOOLCHAIN=1.74.0 python scripts/msrv_check.py --output msrv-evidence
cargo test --locked --offline -p labquest --test advanced_b_e2e --test book_reading_e2e
```

输出目录须尚不存在。Windows PowerShell 门禁另运行 `./scripts/verify_release_windows.ps1 -OutputDirectory windows-evidence`；浏览器门禁使用已安装 Chromium 的 `scripts/browser_e2e.py`，由 CI 保留产物。

## 本轮范围

修复以上问题，补充对照阅读导航、任务卡参考段、feature 与 doctest 检查，以及当前源码/教材输入的指纹记录。保留开发版定位，B4 和真实异步课程仍待完成。
