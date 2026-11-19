# 0.19 之前的验证记录

本目录归档 0.3–0.18 的工程证据与阶段说明。为保留原有报告、日志和截图之间的关系，它们放在同一目录，0.17 候选线的 `rc/`、`rc2/` 仍与对应说明相邻。

**这里的“当前”“本轮”“通过”“未通过”都属于记录所写的历史版本。** 旧平台要求、候选资格和功能范围不作为现在的规则；记录的工程结果也不代表学员已经理解或独立完成课程。当前要求见 [发布规则](../../../RELEASE_POLICY.md)，近期记录见 [验证索引](../../README.md)。

## 按版本查找

| 历史范围 | 主要入口 |
|---|---|
| 0.3–0.11 | [早期 Linux 日志](RUST_1_99_1_LINUX_TEST_EVIDENCE.txt)、[各阶段质量门槛](QUALITY_GATE_HISTORY.md)、[0.11.1 修复证据](RUST_1_99_1_0_11_1_EVIDENCE.txt) |
| 0.5–0.6 SafeSync 实验 | [0.5 基准报告](SAFESYNC_0_5_0_BENCHMARK.json)、[大文件](SAFESYNC_0_5_0_BENCHMARK_LARGE_FILES.json)、[小文件](SAFESYNC_0_5_0_BENCHMARK_SMALL_FILES.json)、[0.6 基准报告](SAFESYNC_0_6_0_BENCHMARK.json) |
| 0.12 | [检查结果](V0_12_0_RESULTS.json)、[实际执行日志](RUST_1_99_1_0_12_0_EVIDENCE.txt) |
| 0.13 | [交接状态](RELEASE_0_13_STATUS.md)、[数据库检查](STATE_DATABASE_0_13_SMOKE.json)、[实际执行日志](RUST_1_99_1_0_13_0_EVIDENCE.txt) |
| 0.14 | [检查结果](V0_14_0_RESULTS.json)、[实际证据](V0_14_0_EVIDENCE.txt) |
| 0.15 | [检查结果](V0_15_0_RESULTS.json)、[归档解压检查](FRESH_ARCHIVE_0_15_0_CHECK.txt) |
| 0.16 | [实际证据](V0_16_0_EVIDENCE.txt) |
| 0.17 候选线 | [rc.1 验收记录](RC_QUALIFICATION.md)、[rc.2 验收记录](RC2_QUALIFICATION.md)、[Windows 原生证据审计](WINDOWS_NATIVE_20261030_AUDIT.md)、[当时的门槛入口](QUALITY_GATE.md) |
| 0.18 dev.1–dev.3 | [dev.1 记录](DEV_0_18_QUALIFICATION.md)、[dev.2 记录](TEACHING_DEV2.md)、[dev.3 记录](TEACHING_DEV3.md) |
| 0.18 dev.4–dev.6 | [dev.4 范围](DEV4_SCOPE.md)、[dev.5 范围](DEV5_VALIDATION.md)、[dev.6 范围](DEV6_SCOPE.md) |

其余同名版本前缀的 JSON 与 TXT 文件是相应批次的详细结果和日志。原始失败、被中断的运行和未完成的检查仍保留，不与成功记录合并。

## 截图和浏览器记录

[LocalHub DOM 检查记录](LOCALHUB_UI_DOM_TEST_EVIDENCE.txt) 与 [桌面截图](LOCALHUB_UI_DOM_DESKTOP.png)、[移动截图](LOCALHUB_UI_DOM_MOBILE.png) 属于原有组件检查；其范围应按原文理解，不能把截图当成当前浏览器端到端验收。

[Studio 桌面](STUDIO_DESKTOP.png)、[Studio 移动](STUDIO_MOBILE.png)、[MiniKV 讲义](MINIKV_LESSON_MOBILE.png) 和 [MiniKV 工坊](MINIKV_WORKSHOP_DESKTOP.png) 同样只反映当时保存的界面。

## 归档说明

归档时只调整 Markdown 的本地导航链接，原始 `.json`、`.txt`、`.png` 文件内容保持不变。记录中的旧路径或命令保留执行时的上下文；查找实际文件时，以本目录和上述链接为准，不要求新学员复跑这些历史检查。

返回 [全部历史记录](../README.md) 或 [近期验证索引](../../README.md)。
