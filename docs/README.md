# 维护文档与源码地图

**正在学习 Rust？** 从 [开始指南](../README_START_HERE.md)、[FirstCard 完成后的下一步](../course/AFTER_FIRSTCARD.md) 或 [课程目录](../course/README.md) 进入。这一目录用于理解和维护项目，初学者无需依次阅读。

## 按任务找文档

| 当前任务 | 入口 |
|---|---|
| 查当前教学范围、近期修订 | [路线与范围](../ROADMAP.md)、[当前变更](../RELEASE_NOTES.md) |
| 了解系统、工具链和已知问题 | [支持环境](../SUPPORTED_PLATFORMS.md)、[已知问题](../KNOWN_ISSUES.md) |
| 贡献源码或检查课程实现 | 下方源码地图、各应用和工程的 README |
| 维护网页学习工作台 | [操作说明](../course/WORKBENCH.md)、[设计与边界](superpowers/specs/2026-11-23-browser-learning-workbench-design.md)、[实现经验](WORKBENCH_DEVELOPMENT.md) |
| 验证或打包交付 | [发布规则](../RELEASE_POLICY.md)、[验证记录索引](../verification/README.md) |
| 追溯旧设计、旧版本或旧故障 | [历史文档](archive/README.md)、[历史验证](../verification/archive/README.md) |
| 查看实现方案和当时的工作清单 | [开发计划记录](superpowers/README.md)；计划的勾选状态不代表当前验收结果 |

## 源码目录分别做什么

| 目录 | 内容与用途 |
|---|---|
| `course/` | 当前讲义、课程地图、提示和学习记录模板；按作品 README 指向的章节阅读 |
| `apps/` | 17 个根工作区包；应用参考实现和 LabQuest 教学平台。`apps/labquest/` 还包含课程模板、固定检查和提示 |
| `projects/` | 进阶 B/C 的学员模板与参考实现、Rust Book 独立实验工作区；以各目录 README 区分角色 |
| `examples/` | 演练输入、产物样例和空白学习记录；示例的通过记录不代表个人成绩 |
| `my-.../` | 使用者自己的练习工程，可能放在仓库内或其他目录；继续编辑已有工程，不按缓存清理 |
| `scripts/` | 教学启动、维护验证、打包与发布工具；Python/浏览器维护脚本不是 FirstCard 前置条件 |
| `verification/` | 指定版本和范围的证据、当前教学审计与开发经验；从索引进入，历史放在 `archive/` |
| `docs/` | 维护导航、历史设计与实现计划；完整源码包包含这些 Markdown 文档 |

课程、样例和测试夹具都是源码的一部分。`target/`、Python 字节码和维护工具生成的输出才是可再生成内容。

## 维护时怎样验证

先按改动范围运行相关测试，再使用现有 CI 检查支持平台。涉及源码目录、文档移动或打包规则时，在平台根目录执行：

```text
python scripts/package.py
python scripts/structural_check.py
```

`package.py` 会重新生成 `SOURCE_MANIFEST.sha256`，并在仓库的上级目录生成带版本号的 source.zip。检查最终源码清单差异应只反映本次改动；还要从新目录解压，运行其中的结构检查并打开相对链接。此检查只证明交付结构和链接，不代替 Rust 编译、运行或真人学习记录。

完整交付要求、固定工具链及 CI 入口只在 [发布规则](../RELEASE_POLICY.md) 维护，避免多个文档各自给出“当前门槛”。

## 生成文件与归档约定

- 个人重复运行的维护报告放在 `artifacts/` 等已忽略的输出目录；旧的 DOM 冒烟脚本现在输出到 `artifacts/browser-dom/`。
- CI 仍使用各工作流声明的输出目录和产物名；当前提交的结果从对应 PR/运行查看。
- 只有需要保留并解释范围的证据才收进 `verification/`。新增记录应写明日期、版本或提交、实际命令、结果和未验证范围。
- 历史原始日志、JSON、截图及内嵌校验值原样保留；搬迁只调整文档导航。原日志中的旧工作目录是当时现场信息。
- 已完成或被替代的设计保留在历史索引，当前状态通过当前课程、源码和同提交证据判断。
- 学员工程、数据库、练习输入和个人记录由使用者保留；本轮整理不删除这些内容。
