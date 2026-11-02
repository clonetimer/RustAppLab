# Rust Book 对照学习与工程验证 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修复 Windows 失败、完整验证进阶 B，并把指定中文教材与现有课程连接。

**Architecture:** 延用 LabQuest 命令分发、只读导航、固定测试合同和现有发布脚本。独立阅读映射供 CLI 与生成任务卡复用；独立 Cargo 工作区显式进入测试计划。

**Tech Stack:** Rust 2021 / Cargo / Python unittest / PowerShell / GitHub Actions。

**Spec:** `docs/superpowers/specs/2026-11-02-book-guided-quality-design.md`

## Global Constraints

- 最低 Rust 1.74.0；Windows x64 MSVC / Linux x86_64。
- 不覆盖已有学员工程；阅读导航只读。
- 每项修复先复现；测试失败必须返回非零，不以零测试或旧报告宣称通过。

## Review Focus

- Windows 默认 cp1252 + 中文 UTF-8 诊断（任务 1）。
- list / strict-titles 损坏仍被误判 B3 通过（任务 2）。
- 同名但无标记的目录、中文空格路径、阅读产生副作用（任务 3）。
- 独立工作区或嵌入讲义变化未改变源码指纹（任务 4）。
- 独立测试未执行、零测试、ignored 或 doctest 失败（任务 4）。

## Task 1: Windows 编译实验

文件：`scripts/check_error_labs.py`、`scripts/tests/test_error_labs.py`。

- [x] 从真实 job 日志确认解码根因，先运行 cp1252 和错误码误匹配的失败回归。
- [x] 明确 UTF-8，匹配诊断行；验证全部错误例和修复例。

## Task 2: 进阶 B 固定合同

文件：`apps/labquest/src/advanced_b.rs`、对应集成测试、`projects/advanced-b/`。

- [x] 先用破坏 list、feature、报告与早期步骤的参考实现复现漏检。
- [x] 修复 B3 合同、交付 feature 测试，修复 B1..B0 文案，补公开 API doctest。
- [x] 通过真实生成器和累计检查回归；审查后增加 Report v2，旧 v1 要求重新检查，保留工程与历史记录。

## Task 3: 对照阅读

文件：`book_reading.rs`、`learning.rs`、`guide.rs`、`lib.rs`、`book_reading_e2e.rs`。

- [x] `learn reading [工程]`：识别已有工程类型，返回主题链接、思考实验和返回检查的命令。
- [x] 共用映射供生成的 Markdown / HTML 任务卡使用，注明版本差异。
- [x] 验证全部作品映射、未知目录拒绝、中文路径和只读行为。

## Task 4: 完整工程验证与交付

文件：`scripts/release_check.py`、`msrv_check.py`、`verify_release_windows.ps1`、相关测试和文档。

- [x] 复现进阶源码/讲义不影响指纹，增加默认/严格 feature、Debug/Release、doctest 的独立工作区检查。
- [x] 更新 Python / PowerShell 的证据规则和快速检查命令，验证零测试、ignored 与失败不能通过。
- [x] 更新课程/入口/借鉴记录和当前已知问题，重新生成清单。
- [x] 完整 Linux 测试与独立审查后提交 feature 分支、建立 PR，检查 Windows/Linux/MSRV/浏览器结果。首轮 Linux/MSRV/浏览器通过；Windows 暴露的测试目录名问题已修复并提交同一 PR，最终状态以该 PR 当前提交的 CI 为准。
