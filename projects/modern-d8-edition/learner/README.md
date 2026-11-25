# 我的 Edition 2024 告警筛选器

先安装 Rust 1.100.0（首次需联网），在当前目录运行 `rustc --version`。随后用编辑器修改自己的 `src/lib.rs`。目标是学会利用新版规则，而不是背诵语法。

| 步骤 | 只修改 | 验证命令 |
|---|---|---|
| 1 | select_alarm | cargo test --locked --offline --test contracts step1_ |
| 2 | ensure_threshold | cargo test --locked --offline --test contracts step2_ |
| 3 | legacy_kind 宏 | cargo test --locked --offline --test contracts step3_ |

初始业务测试预期失败。每一步先预测，再运行，理解错误；只改 TODO，勿改合同测试。最后运行 `cargo test --locked --offline --all-targets` 和 `cargo run --locked --offline --example alarm -- 45 40`。尝试 `missing 40`、`-1 -2`、`bad 40`，比较输出与退出状态。再增加一项自己的边界测试。

在 LEARNING_LOG.md 写下每步的预测、结果、改动、解释、不同输入的迁移结果。公开合同不是独立评审，也不代表个人掌握。

若想知道为什么行为不同，返回完整课程的 `course/BOOK_D8_EDITION_2024.md`（学员包也包含该讲义）。
