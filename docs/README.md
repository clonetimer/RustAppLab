# 维护文档与源码地图

正在学习 Rust，请从 [开始指南](../README_START_HERE.md) 或 [网页工作台](../course/WORKBENCH.md) 进入。本目录只供维护者使用。

| 当前任务 | 入口 |
|---|---|
| 课程范围与支持环境 | [范围](../ROADMAP.md)、[支持平台](../SUPPORTED_PLATFORMS.md)、[已知边界](../KNOWN_ISSUES.md) |
| 网页工作台实现 | [维护说明](WORKBENCH_DEVELOPMENT.md) |
| 验证与打包 | [发布规则](../RELEASE_POLICY.md)、[验证入口](../verification/README.md) |

`course/` 保存当前讲义与学习记录模板，`apps/` 保存 17 个根工作区包，`projects/` 保存独立教学与参考工程，`examples/` 保存示例数据，`scripts/` 保存启动和维护工具。`verification/` 仅保留当前校验实际引用的审计数据和验证入口。用户自己的 `my-...` 工程、数据、学习记录不属于历史文档清理范围。

目录、文档或打包规则改动后，在平台根目录执行：

```text
python scripts/package.py
python scripts/structural_check.py
```

打包会重新生成源码校验清单。还需按现有 CI 执行同提交双平台、浏览器和发布验收；结构检查不代替真实编译与运行。历史文档和原始证据仍可通过对应 Git 提交、发布资产及 CI 记录追溯，无需复制到当前源码目录。
