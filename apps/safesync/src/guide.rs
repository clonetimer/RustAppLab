pub const MISSIONS: &str = r#"SafeSync 课程 16–19：
  16 真实备份与恢复：亲手删除演示源目录，再从备份恢复到全新目标
  17 哈希与损坏检测：修改一字节并观察 verify 非零退出码；挑战 --quiet
  18 安全提交与失败恢复：拒绝已有目标、路径穿越；挑战 restore --dry-run
  19 自动化交付：备份重复执行幂等、清单格式、回归测试；挑战 verify --json
  20 进程异常退出：stages 检查暂存，recover 只提交完整快照
  21 发布基准：Release 真文件读写，Python SHA-256 独立校验
  22 源目录并发变化：复制时哈希和最终二次扫描拒绝不一致快照
  23 暂存类型隔离：恢复暂存不能被误认为可提交的备份；旧版迁移
讲义见 course/16_backup_first_snapshot.md 至 course/23_stage_provenance.md。
挑战默认 #[ignore]，需要手动指定单个测试进入 RED → GREEN 循环。
"#;
