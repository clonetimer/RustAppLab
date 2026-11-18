# rc.3 核心概念与实操 · 本地验证记录

日期：2026-11-18。基线为 rc.2 / `ed255fe31ec1be2faba04ffb720660d13938e6a6`；以下记录来自当前 rc.3 开发源码在 Linux 上的实际执行。远端最终提交的 Windows/Linux 完整门禁与发布身份由同提交 CI 单独确认。本记录不表示实际 IDE 操作、用户 Windows 机器体验或学员理解已经通过。

## 基础参考工作区

| 工具链 | Debug | Release | Doctest | 失败 / 忽略 |
|---|---:|---:|---:|---:|
| Rust 1.74.0 | 48 | 48 | 76 | 0 / 0 |
| Rust 1.85.0 | 48 | 48 | 76 | 0 / 0 |

[基础实验原始记录](foundation/report.json) 保存完整命令、日志哈希及所测 Rust 源码/manifest/lock 的哈希。成功例与预期编译失败例均计入 doctest；后者用于验证规则边界。该隔离工作区的 stable `cargo fmt --all -- --check` 与 `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` 也已通过。

Python 门禁回归由 `python -m unittest discover -s scripts/tests -v` 实际执行。首次概念批次为 **131 / 131 通过**，原始结果保留在 [首次 Python 日志](python-tests.txt)。后续 CI 同名产物修复增加选择器 20 项、下载器 18 项回归，完整套件再次执行为 **169 / 169 通过**，见 [产物修复后的完整 Python 日志](python-tests-artifact-fix.txt)。

## CI 重跑证据与产物选择

[Run 226](https://github.com/clonetimer/RustAppLab/actions/runs/44250858396) 的 Windows 交付首次超时，同提交、同预算复验通过；候选组装随后读到了旧的同名失败产物。两个原始产物均保留，[元数据快照](ci-retry-artifacts.json) 包含当时 API 返回的 19 条记录。新选择器按带时区的创建时间选择每个名称的最新记录，真实快照回放得到 18 个生产者产物，Windows 对应 `14404059996`；下载器按明确 ID 下载，核对 SHA-256 后解包，并保存完整选择清单。

本地集成检查还读取了该 Windows 成功产物的实际 ZIP，确认 API 摘要匹配、32 个文件可安全提取，两个交付报告均为 PASS，选择清单按原始字节保留。下载边界以已留存字节替代网络调用；真实 GitHub 权限、下载及所有候选报告组合仍由新提交的完整 CI 验收。Run 226 不作为修复后的通过记录。详细经过与证据边界见 [开发经验第 31–32 节](../DEVELOPMENT_EXPERIENCE_D6.md)。

## 工具、迁移与 Nightly

| 模式 | 命令步骤 | 实际结果 | 证据 |
|---|---:|---|---|
| D 工具 | 15 | unused_mut 失败/修复，格式、Clippy、Debug 11、Doc 3、Release 11、文档生成 | [report](tools/report.json) |
| E 迁移 | 14 | 原始 11、探针 12；迁移后的 Debug 12、Doc 3、Release 12 | [report](edition/report.json) |
| G Nightly | 6 | 固定日期测试 1；Stable 有 gate / 无 gate 两次拒绝 | [report](nightly/report.json) |

D/E 使用 Rust 1.85.0，G 使用 nightly-2026-11-17 并以 1.85.0 为 Stable 对照。所有报告均为 PASS，原仓库输入快照保持不变。Nightly 是独立选修，不代替 Stable/MSRV 验收。

D/E 运行使用 `--keep-workspace`，报告中包含当次执行保留副本的直接入口；这些本机临时路径在别人的机器上不能复用，应重新运行以生成自己的副本。相对 `lab` 的配置字节另存为 [tools 配置](tools/ide-workspace.json) 与 [edition 配置](edition/ide-workspace.json)，并被证据哈希保护。可编辑的 `IDE_RECORD.md` 留在当次临时副本，初始 `NOT_RUN`，不进入不可变命令证据。

复现方式见 [实操指南](../../course/BOOK_D6_PRACTICAL.md)。每次使用新的输出目录；`tools` / `edition` 带 `--keep-workspace` 可保留 IDE 入口。当前候选包校验器已接受这两份实际 Linux 报告，确认命令、输入快照、配置内容、路径关系和证据哈希一致。CLI 和 JSON 结构验证不证明 VS Code 跳转或诊断体验。

旧 rc.2 原始记录保留于 [d6-practical-linux](../d6-practical-linux/README.md)，不改写其版本或运行结果。
