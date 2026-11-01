# 学习导航：读任务、修改、检查、理解错误

## 当前入口

Windows 已有工程请双击平台根目录 `LEARN_NEXT_WINDOWS.cmd`。目录可以在旧版本文件夹里，也可以在外部磁盘；导航不会迁移、删除或覆盖源码。没有工程时用 `START_WINDOWS.cmd` 或 `CONTINUE_WINDOWS.cmd` 创建。

| 动作 | Windows 菜单 | 平台根目录命令 |
|---|---|---|
| 查看下一步 | N | `cargo run --offline -p labquest -- learn next "my-unitlab"` |
| 当前微任务卡 | L / Enter | `cargo run --offline -p labquest -- learn lesson "my-unitlab"` |
| 查看一层提示 | 1、2、3 | `cargo run --offline -p labquest -- learn hint "my-unitlab" 1` |
| 真正编译检查 | C | `cargo run --offline -p labquest -- learn check "my-unitlab"` |
| 指定范围 | S | `cargo run --offline -p labquest -- learn check "my-unitlab" 2` |
| 打开自己源码 | E | 用编辑器打开该工程的 `src/lib.rs`，FirstCard 与两款独立作品默认是 `src/main.rs`，也可打开自己拆分的模块 |

`learn lesson`、`next`、`hint`、`route`、`diagnose` 是只读操作。只有明确执行 `learn check` 才编译运行学员代码、生成检查记录。不要检查陌生的不可信源码，平台不是代码安全沙箱。

## 自动选择步骤怎样工作

没有记录，从第一步开始；第 1 步通过，建议做第 2 步；检查第 3 步时若前两步通过、第三步失败，下次仍定位第 3 步。若第一步退化，下一次明确定位第一步。

代码改变后，历史报告不再证明当前行为。导航只把历史用于选择建议任务；检查始终从第一步累计执行到当前步，不凭历史跳过测试。编译失败发生在功能检查之前，因此保留上次尝试范围，先修复编译错误。

全部三步通过后，导航改为提醒用自己的数据交付，不再让学员机械地修改已经完成的 TODO(3)。它不会自动创建下一款软件，也不会自动写入“独立掌握”的标记。

## 提示层级与任务卡范围

十二款 Studio 各有三张微任务卡，每张对应一个函数，并有概念、实现思路、自测输入三层提示。新生成工程内的 `TASK_CARDS.md` 可离线阅读；已有旧工程直接用 `learn lesson` / `learn hint`，无需重新创建。

提示级别放在目录之后，指定步骤放在最后。例如：

```powershell
cargo run --offline -p labquest -- learn hint "my-quizdeck" 2 3
```

这表示 **QuizDeck 第 3 步的第 2 层提示**。只读提示不会替换代码，也不会把“看过提示”记作“独立完成”。TaskPocket、PageMint 则是三个累计行为检查点，并不指定内部函数；同样可按需选择三层提示。

## 看懂两类失败

**编译失败**：程序没有进入功能测试。平台保留原始 error，显示第一条常见诊断的含义和 `文件:行:列`。Studio 在临时目录编译，实际应编辑自己工程的对应源码，不要打开已清理的临时文件。E0308 表示类型不匹配，不一定是分号；诊断只给检查方向。

**功能失败**：程序已经能够编译，但输出或行为不符合合同。查看当前步骤、失败输入和对应函数。`assert_eq!` 的 left/right 是代码里两个表达式的结果，不能在没看断言时一概把左边当实际、右边当预期。

`Compiling` 只是正常构建信息。链接器或系统 SQLite 缺失是环境问题，不能归咎于温度转换公式。不要一看到红字就删除练习、重装 Rust。

## 外部日志也能解释

内置检查已自动显示诊断，不需要学员再手动重定向。确实有外部日志时：

```powershell
cargo run --offline -p labquest -- learn diagnose "error.txt"
```

支持 UTF-8（可带 BOM），以及带 BOM 的 UTF-16 LE/BE 日志；可读取 PowerShell 5.1 常见的 UTF-16 输出。上限 128 KiB，不接受符号链接或无效编码。识别首条错误时会忽略 ANSI 颜色序列，不会因为第一条未知而跳去解释第二条。

更多实作：[基础作品连续练习](FOUNDATION_PATH.md)、[可编译验证的错误实验](RUST_ERROR_LABS.md)。

技术参考：Rust 官方 [E0308](https://doc.rust-lang.org/stable/error_codes/E0308.html)、[错误代码索引](https://doc.rust-lang.org/stable/error_codes/)、[字符串 API](https://doc.rust-lang.org/std/primitive.str.html)。

## 0.18.0-dev.3：跨作品进度与下一阶段

`learn overview <练习父目录>` 可以只读汇总多个 Studio 工程的历史检查状态，区分未检查、过期、阶段失败和三步历史通过。Windows 续学窗口按 P 查看同级练习。该状态不代表真实教学达标；详见 `course/PROGRESS_OVERVIEW.md`。

LedgerLite、BadgeForge、TermGarden 现也提供三阶段 `learn lesson` 与 1/2/3 分层提示；每一步有预测、自有数据实验和反思。详见 `course/FOUNDATION_NEXT_STEPS.md`。


### 0.18.0-dev.5 · 第四段衔接

BatchMeter → MiniKV → NetFetch → ServiceDesk 的系统应用路线继续保留；StoreDesk、AsyncBoard 本轮补齐 6 张微任务卡、18 条分层提示，总覆盖为 12 款/36 张/108 条。完整路径见 [SYSTEMS_PATH.md](SYSTEMS_PATH.md)。新建工程有 TASK_CARDS.md，旧工程可直接使用新版 learn lesson/hint，无需覆盖。StoreDesk/AsyncBoard 的引导已补齐，事务与等待实验见 ../RELIABILITY_PATH.html；成熟异步网络、桌面和 WASM 仍未完成。

本轮具体入口：[第四段：事务与等待](../RELIABILITY_PATH.html)。新手不跳级；StoreDesk 先修 LedgerLite，AsyncBoard 先理解 BatchMeter。后续由 TaskPocket/PageMint 检验独立开发，不把 36 个 TODO 完成自动认定为掌握整个 Rust 生态。


## 独立开发工程（dev.6）

`learn lesson/hint/check/next/overview` 已支持 TaskPocket 与 PageMint；同一 Windows 菜单可以续学。建议先完成只读检查点1，再完成正常业务2，最后完整合同3。详细见 [第五段](INDEPENDENT_PATH.md)。新生成的 START.html、DEVELOPMENT_PLAN.md、TEST_PLAN.md、RELEASE_CHECKLIST.md 与 self_check 测试入口提供从设计到交付的引导，不提供业务答案。

独立作品检查点按范围记录成功/失败；不会像 Studio 那样把一个累计检查点再细分成“已完成几个指定函数”。失败时看该次报告里的第一个 FAIL 并重试当前范围。旧 APP_ASSESSMENT 不自动换算新进度。`.app-reports` 只用于导航，完整行为检查才更新 APP_ASSESSMENT。指纹包括源码、manifest、测试与配置，但不覆盖个人数据和外部依赖。
