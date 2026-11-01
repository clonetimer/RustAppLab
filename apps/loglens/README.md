# LogLens — 本地日志分析器

Rust AppLab 的第二款独立应用。**当前是教学开发版源码，不等于已验证的 Stable 二进制。**

## 真实用途

在部署前后比较日志，快速回答“新版本 ERROR/FATAL 的记录占比是否变化了”，并将调查证据保存为 Markdown。它不会读取服务器 API、不会上传日志，也不会试图自动推断真实故障原因。

```sh
# 从 RustAppLab 根目录执行
cargo run -p loglens -- demo
cargo run -p loglens -- analyze loglens-playground/baseline.log
cargo run -p loglens -- analyze loglens-playground/candidate.log
cargo run -p loglens -- compare loglens-playground/baseline.log loglens-playground/candidate.log
cargo run -p loglens -- report loglens-playground/candidate.log -o my-log-report.md
```

**例子说明**：演示旧日志有 8 条有效记录与 1 条 ERROR；新日志有 8 条有效记录、3 条 ERROR、1 条无法识别的堆栈文本。错误占比从 12.5% 变为 37.5%，增加 25 个百分点；由于样本与业务量未严格对齐，这不等于服务实际故障率的变化。

## 输入契约

当前识别 UTF-8 单行文本：

```text
INFO service started
[WARN] disk is slow
2026-10-28T10:20:30Z ERROR timeout reading cache
warning: deprecated API
```

不支持跨行堆栈语义、JSON Lines、任意日志框架格式或本地时区推算。默认非严格模式会跳过不支持的非空行并显示未识别数量；`--strict` 第一个非法行即退出。时间窗口仅对识别到的 UTC `YYYY-MM-DDTHH:MM:SSZ` 时间戳求最小/最大值。

**统计口径**：记录级错误率 = `(ERROR + FATAL) / 已识别日志记录数`。未识别行不进入分母；不是请求失败率、CPU/内存故障率或事件因果分析。

## 质量与风险边界

- 文件输入逐行读取，不把原始文件一次性载入内存；但不同日志消息的聚合映射没有容量上限，高基数日志仍可能消耗大量内存。
- 日志中的控制字符在终端展示时转成替换字符，Markdown 表格单元格对 `|`、尖括号等进行转义。
- 写报告默认拒绝覆盖，`--force` 只允许普通文件，并拒绝将报告写回输入日志路径；**当前强制覆盖不保证原子更新**，操作系统级并发修改仍存在竞争条件。
- 日志可能含路径、口令或用户数据。工具本身不上传数据，但导出的报告需要在分享前人工审查。
- 本版本实现不依赖第三方 crate，可在离线 Rust 环境构建；平台 CI 仍需真实执行结果。

教学入口：[`course/08_first_log.md`](../../course/08_first_log.md) 至 11 章。`missions` 命令可查看任务路径。
