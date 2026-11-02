# 当前学习路线（0.19.0-dev.2）

[Rust Book 对照学习](BOOK_GUIDE.md) · [学习导航](LEARNING_NAVIGATOR.md) · [进阶 A](ADVANCED_A.md) · [进阶 B](ADVANCED_B.md)。已有学员可用 `learn reading <工程路径>` 按当前作品查找相关概念。

完成 FirstCard 后先去 [作品工坊](../WORKSHOP.html)，从 [UnitLab](studio/unitlab.md) 开始。原有 00–31 讲义保留，但不再要求零基础从复杂工具源码直接入手。

新增进阶作品：[TermGarden](studio/termgarden.md) · [StoreDesk](studio/storedesk.md) · [SQLite环境说明](SQLITE_SETUP.md)。完成基础再进入，不取代UnitLab入口。

[20 款课程路线](CURRICULUM.md) · [Studio 使用说明](STUDIO_GUIDE.md)

---

# Rust AppLab · 以应用为中心的教学地图

**新学员只需要先打开根目录的 [`START_HERE.html`](../START_HERE.html)，并做完 [`零基础第 0 阶段`](BEGINNER_WINDOWS_GUIDE.md)。不要在第一天就打开下表 32 章。**

**使用规则**：先运行应用，再理解一小段源码；最后交付一项能被验证的新能力。不要把所有关卡一次读完。

| 关卡 | 应用 | 学习成果 | 教学验证 |
|---|---|---|---|
| 00 | FileQuest | [五分钟得到第一份文件分析报告](00_first_five_minutes.md) | Demo / Scan / Find / Report |
| 01 | FileQuest | [CLI 版本命令](01_cli_contract.md) | `mission_01_version_command` |
| 02 | FileQuest | [按扩展名扫描](02_filesystem.md) | `mission_02_scan_extension_filter` |
| 03 | FileQuest | [错误处理](03_error_handling.md) | 非零退出码与局部异常 |
| 04 | FileQuest | [文件体积分组](04_collections.md) | `mission_04_extension_size_summary` |
| 05 | FileQuest | [只按文件名检索](05_ownership_and_search.md) | `mission_05_find_basename_only` |
| 06 | FileQuest | [报告标题安全处理](06_report_and_tests.md) | `mission_06_custom_report_title_is_escaped` |
| 07 | FileQuest | [可交付的本机二进制](07_ship.md) | release 编译与实机验收 |
| 08 | LogLens | [从真实日志流统计事件](08_first_log.md) | `mission_08_analyze_top_count` |
| 09 | LogLens | [消息过滤与异常行契约](09_parsing_and_errors.md) | `mission_09_filter_message` |
| 10 | LogLens | [比较版本错误率](10_compare_and_metrics.md) | `mission_10_fail_on_regression` |
| 11 | LogLens | [生成 Markdown 诊断报告](11_log_report.md) | `mission_11_report_custom_title` |
| 12 | LocalHub | [通过真实 HTTP 请求访问 Rust](12_http_first_response.md) | TCP 端到端验证 |
| 13 | LocalHub | [按扩展名过滤 JSON 文件列表](13_files_api.md) | `mission_13_extension_filter` |
| 14 | LocalHub | [在浏览器展示日志统计](14_log_data_api.md) | `mission_14_logs_min_level` |
| 15 | LocalHub | [服务安全边界与身份定制](15_release_gate.md) | `mission_15_custom_service_name` |
| 16 | SafeSync | [第一份可验证备份与真实恢复](16_backup_first_snapshot.md) | 备份→验证→恢复闭环 |
| 17 | SafeSync | [改动一个字节，识别损坏](17_integrity_hash.md) | `mission_17_verify_quiet` |
| 18 | SafeSync | [安全提交与拒绝覆盖](18_safe_restore.md) | `mission_18_restore_dry_run` |
| 19 | SafeSync | [备份幂等与自动化交付](19_release_recovery.md) | `mission_19_verify_json` |
| 20 | SafeSync | [异常退出、完整暂存检查与恢复](20_interrupted_backup.md) | 真实子进程故障注入与 `recover` 验证 |
| 21 | SafeSync | [性能基准、外部 SHA-256 与发布证据](21_benchmark_release.md) | 真实字节比较和多负载实验 |
| 22 | SafeSync | [源文件变化与一致性检查](22_source_consistency_lab.md) | 确定性修改时机、复制摘要检查、拒绝提交 |
| 23 | SafeSync | [暂存身份谜题与旧版兼容](23_stage_provenance.md) | 备份/恢复用途隔离、LEGACY 明确处理 |
| 24 | LabQuest | [网关超时事故调查：六步综合案例](24_labquest_case.md) | 文件、日志、源码修改和备份的一体化真实交付 |
| 25 | LabQuest | [亲手改一个 Rust 程序的输出能力](25_labquest_rust_code.md) | 动态计数、编译运行、源码证据与过期失效 |
| 26 | LabQuest | [多输入迁移验收](26_transfer_verification.md) | 固定答案与比较符错误无法通过真实运行测试 |
| 27 | LabQuest | [下一步引导与学习循环](27_guided_learning_loop.md) | 预测→实现→检验→解释→交付 |
| 28 | LabQuest Capstone | [独立开发 TaskPocket](28_independent_product.md) | 全新 Cargo 工程、文件持久化、亲手写 Rust |
| 29 | LabQuest Capstone | [多场景验证与软件交付](29_product_release.md) | 9 组黑盒测试、实际使用、Release 交付 |
| 30 | LabQuest Transfer | [PageMint：第二次跨领域独立开发](30_second_independent_product.md) | 多笔记解析、排序、HTML 生成与转义 |
| 31 | LabQuest Transfer | [用真实笔记交付网页与反思迁移能力](31_transfer_release.md) | 独立作品、跨领域复盘、平台差异记录 |

如果希望先体验任务式学习，可直接从 `cargo run -p labquest -- start labquest-playground` 开始；传统基础路线则

开始前先读仓库根目录的 [`README_START_HERE.md`](../README_START_HERE.md)。

**挑战与基础功能严格分离**：`cargo test --workspace` 运行当前已经实现的功能测试；尚未完成的 RED→GREEN 挑战标记为 `#[ignore]`，只有指定 `--ignored` 才会执行。学员不能把忽略的挑战理解成“已经通过”。

举例：

```sh
cargo test -p filequest --test challenges mission_01_version_command -- --ignored --exact
cargo test -p loglens --test challenges mission_08_analyze_top_count -- --ignored --exact
cargo test -p localhub --test challenges mission_13_extension_filter -- --ignored --exact
cargo test -p safesync --test challenges mission_17_verify_quiet -- --ignored --exact
```

卡住后再按顺序阅读 [`HINTS.md`](HINTS.md) 或 [`HINTS_08_11.md`](HINTS_08_11.md) 或 [`HINTS_12_15.md`](HINTS_12_15.md) 或 [`HINTS_16_19.md`](HINTS_16_19.md)。关卡记录模板在 [`LEARNING_LOG.md`](LEARNING_LOG.md)。
