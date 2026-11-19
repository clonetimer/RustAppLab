# 事务与等待的练习清单

这里是 StoreDesk / AsyncBoard 的**可选输入样例和学习记录模板**，不含数据库、不含学员业务答案，也不预填成功记录。完整步骤见 [事务与等待路线](../../course/RELIABILITY_PATH.md)。

## 前置与文件含义

StoreDesk 先修 LedgerLite 和 `Result`；AsyncBoard 先理解 BatchMeter 的线程与 MiniKV 的借用。刚完成 FirstCard 时先走 [基础后续指南](../../course/AFTER_FIRSTCARD.md)。StoreDesk 还需要原生 SQLite，按 [环境说明](../../course/SQLITE_SETUP.md) 配置；不要求为这些样例安装 Python。

| 文件 | 输入与要观察的行为 |
|---|---|
| `rollback.tsv` | PEN -2、BOOK -99；第二项失败时整批及流水回滚 |
| `success.tsv` | PEN -2、BOOK +1；两项与流水一起提交 |
| `repeated-sku.tsv` | PEN -1、PEN -2；顺序应用于事务内最新值 |
| `jobs.tsv` | 三条不同等待配置的提醒 |
| `equal-delay.tsv` | 两条同延时提醒，不要求固定完成顺序 |
| `duplicate-jobs.tsv` | 重复 ID；应拒绝且不创建报告 |
| `LEARNING_LOG_TEMPLATE.md` | 填写自己的预测、实际结果与解释 |

TSV 使用真正的 Tab 和 UTF-8，修改内容时保留列分隔。`.db` 是运行后创建的 SQLite 文件，不用文本编辑器修改。

只有学员包、没有平台 `apps/` 时，先在 `Studio/storedesk` / `Studio/asyncboard` 完成自己的工程，再按末节说明替换参考命令；不需要为使用样例重建工程。

## 在新目录观察参考行为

以下均在 **RustAppLab 源码根目录**运行，先复制样例并准备参考数据库：

```powershell
if (Test-Path -LiteralPath "my-reliability") { throw "目录已存在，请换一个新名字。" }
Copy-Item -Recurse -LiteralPath "examples/reliability" -Destination "my-reliability"
cargo run --locked --offline -p storedesk -- doctor
cargo run --locked --offline -p storedesk -- init my-reliability/inventory.db
cargo run --locked --offline -p storedesk -- add my-reliability/inventory.db PEN "Alice's 钢笔" 10
cargo run --locked --offline -p storedesk -- add my-reliability/inventory.db BOOK "我的 Rust 练习本" 3
cargo run --locked --offline -p storedesk -- apply my-reliability/inventory.db my-reliability/rollback.tsv
cargo run --locked --offline -p storedesk -- list my-reliability/inventory.db
cargo run --locked --offline -p storedesk -- history my-reliability/inventory.db
```

`apply rollback.tsv` 应失败；继续看 list / history，仍应为 PEN=10、BOOK=3、流水为空。失败退出码本身不足以证明回滚。doctor 如果报告链接环境问题，先按 SQLite 讲义修复。

再依次执行：

```powershell
cargo run --locked --offline -p storedesk -- apply my-reliability/inventory.db my-reliability/success.tsv
cargo run --locked --offline -p storedesk -- list my-reliability/inventory.db
cargo run --locked --offline -p storedesk -- history my-reliability/inventory.db
cargo run --locked --offline -p storedesk -- apply my-reliability/inventory.db my-reliability/repeated-sku.tsv
cargo run --locked --offline -p storedesk -- list my-reliability/inventory.db
cargo run --locked --offline -p storedesk -- history my-reliability/inventory.db
```

成功批次后为 PEN=8、BOOK=4、流水 2 条；重复 SKU 批次后为 PEN=5、BOOK=4、流水总数 4 条。判断回滚依据是逻辑库存和流水，不是数据库文件的哈希是否不变。

## 再观察等待与失败输出

仍在 **平台根目录**，这些报告路径必须尚不存在：

```powershell
cargo run --locked --offline -p asyncboard -- preview my-reliability/jobs.tsv
cargo run --locked --offline -p asyncboard -- run my-reliability/jobs.tsv my-reliability/RESULTS.tsv
cargo run --locked --offline -p asyncboard -- run my-reliability/equal-delay.tsv my-reliability/EQUAL.tsv
cargo run --locked --offline -p asyncboard -- run my-reliability/duplicate-jobs.tsv my-reliability/SHOULD_NOT_EXIST.tsv
```

最后一条应失败，且不产生 `SHOULD_NOT_EXIST.tsv`。正常报告每个 ID 出现一次，说明和配置保留；同延时不要求顺序。`wait_ms` 是配置，不是实测耗时，不能用其总和证明性能。

## 换成你的实现与个人数据

上述命令运行完整参考程序。按 [StoreDesk](../../course/studio/storedesk.md) / [AsyncBoard](../../course/studio/asyncboard.md) 创建学员工程、编辑自己的 `src/lib.rs` 并完成三个检查点后，在学员目录用 `cargo run --offline --` 替换相应 `-p` 参考命令；数据与输出采用你的完整路径，避免误读其他目录。

把两个物品、三项提醒换成自己的内容。先预测一份“第一条有效、第二条失败”的批次，再核对库存和流水；为此补一条回归测试。保存正常报告、同延时报告、失败记录和填写后的学习日志。

你应能解释事务为什么必须包含流水，以及 `.await` 等待与阻塞线程的区别。AsyncBoard 是教学执行器，本实验没有真实网络提醒或跨软件事务。

## 下一步

完成自己的软件、数据、异常实验和解释后，按 [独立作品路线](../../course/INDEPENDENT_PATH.md) 开发 TaskPocket / PageMint，练习自行拆分需求和接口。参考清单运行成功不替代这一步独立设计。
