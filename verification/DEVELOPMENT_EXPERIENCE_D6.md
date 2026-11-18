# RustAppLab · D6 开发经验记录

Date: 2026-11-17; reviewed 2026-11-18

Scope: D6-A/B references, practical verification, beginner gaps and immutable RC.2 delivery

## 1. Keep reference labs isolated from learner products

D6-A and D6-B are reference workspaces, not additional learner projects. They remain outside the root Cargo workspace and do not change the frozen 17-package learner set. Each lab declares Edition 2021, Rust 1.74 MSRV, `publish = false`, and no registry dependencies.

**Rule:** a coverage reference lab must be independently buildable and testable without silently changing the learner product inventory. Register every reference workspace in structural validation, release checks, declared-MSRV checks, source packaging, learner-kit packaging, POSIX/PowerShell verification, and regression tests.

## 2. Make interactive examples deterministic at the test boundary

The guessing game uses an injected comparison function and a small time-seeded xorshift helper only for the interactive demonstration. Unit tests exercise fixed secret/guess pairs; they never wait for wall-clock randomness or assert a particular random output.

The time-seeded helper is explicitly documented as non-cryptographic. Do not present a small PRNG as suitable for security-sensitive randomness. When a future lesson requires a production RNG, introduce the `rand` crate only with a pinned lockfile and a reproducible offline dependency strategy.

## 3. Do not mutate process environment in unit tests

The config-path lesson uses `config_dir_from(Option<&OsStr>)`. The CLI reads environment variables, but the core path decision is a pure function. This avoids tests racing over global process environment and makes Windows/POSIX path behavior easier to isolate.

**Rule:** inject environment-derived values into pure functions; reserve direct `std::env` access for the application boundary.

## 4. Demonstrate panic versus Result with an explicit decision boundary

Ordinary invalid input is represented as `Result`; panic is reserved for a broken programmer invariant. A panic test may use `catch_unwind` to observe behavior, but the lesson must not imply that catching arbitrary panics is a general recovery strategy.

## 5. Benchmark equivalent work and never assert a performance winner

Loop and iterator versions receive the same slice and repeat count, and the test checks equal results only. Timing is an opt-in release-mode observation using `Instant` and `black_box`; no CI test asserts that one form is faster.

**Rule:** CI validates semantic equivalence and that the benchmark runs. Performance claims require repeated measurements, release builds, controlled inputs, and a report of variability—not a single wall-clock assertion.

## 6. Preserve I/O errors while using iterator adapters

The buffered line-count example uses `BufRead::lines` with `try_fold`. Each line read can fail, and `?` propagates the error. Do not convert an I/O error into an empty line or silently skip it unless the application contract explicitly permits that behavior.

## 7. Keep Appendix keyword and symbol references edition-aware

D6-A now includes the complete current Strict/Reserved/Weak keyword sets and explains the Edition 2018/2024 changes. The older Chinese book is retained as the audit chapter baseline, while current token classification is linked to the Rust Reference.

**Rule:** never copy a historical keyword table forward without checking the current Rust Reference and Edition Guide. Distinguish current syntax from historical Edition behavior.

## 8. Treat documentation tables and generated lesson text as tested deliverables

A previous programmatic Markdown edit duplicated the document body at the macro-metavariable symbol `$` and broke a backtick pair. The later experience record repeated that corruption. The 2026-11-18 audit reproduced the defect with a unique-title/section test before reconstructing this section.

**Rule:** use literal replacements or structured patches when text contains `$`, backticks or regular-expression replacement tokens. After a generated edit, review the diff and check unique headings, table completeness and links. Keyword presence cannot detect an entire duplicated document.

## 9. Apply the isolated workspace's exact stable rustfmt output

D6-B passed the source/structural/tooling checks and Linux/Rust 1.74 compilation, but the first native CI attempt stopped at the D6-B formatting gate. The diff was limited to rustfmt layout in the library assertion/error constructors and CLI test process handling. The workspace was formatted to current stable rustfmt and the source manifest refreshed before the complete cross-platform rerun.

**Rule:** every isolated workspace must be checked with its exact manifest command. A root-workspace `cargo fmt --all -- --check` does not include reference workspaces excluded from the root workspace. Formatting is a real gate, not a cosmetic afterthought.


## 10. Treat D6 as one acceptance batch, and verify the actual archives

The final D6 acceptance must cover more than compiling the reference crate. Run 215 validated Linux and Windows native engineering, browser, C1/C2 regressions, offline portable archives, delivery relocation, and the actual Rust 1.74 MSRV. D6-B's exact evidence is 14 library unit tests + 6 CLI integration tests + 2 doctests, all passing in Debug/Release where applicable.

The assembled candidate was opened and inspected independently:
- The source archive contains 651 manifest-tracked files; every entry exists and every SHA-256 matches.
- The learner kit contains both D6-A and D6-B lesson/reference files.
- All 17 expected learner projects remain present; D6 reference workspaces are not counted as new learner projects.
- All 12 outer candidate file hashes match the release checksum list.

**Rule:** inspect the actual assembled ZIPs, not just the packaging script's success message. Count the learner projects by the published package contract, verify the source manifest against extracted bytes, and check that reference labs are included without changing the learner-project inventory.

## 11. Separate implementation coverage from learner mastery

At the PR #10 starting point, the audit recorded 77 items with related implementations, 3 partial items and 1 reference index across 81 rows. This was a directory-mapping inventory. The 2026-11-18 review found missing beginner objectives underneath some of those mapped rows. Author-side reproducibility and learner mastery must be assessed independently: missing learner submissions alone do not show that a course implementation is missing.

**Rule:** report implementation evidence and learner evidence separately. CI, doctests, package integrity, and browser tests prove engineering contracts; learner mastery still requires the learner to complete exercises and explain the results.

## 12. Audit concepts behind the directory labels

The 81 original identifiers were checked against the actual old Chinese Book directory. A count and a few sample IDs were insufficient: the regression now compares the full ID set. The previous 7.3 mapping said Cargo path dependencies even though that chapter teaches Rust module paths. The review also found missing explicit variable/shadowing and compound-type practice.

**Rule:** retain three separate claims: topic mapping, reproducible reference behavior, and learner mastery. Do not report 77/81 as conceptual coverage, and do not interpret zero rows labeled gap as proof that no knowledge is missing. A detailed remaining-concept count stays unquantified until a full concept audit exists.

## 13. Reproduce setup on a genuinely minimal toolchain

On a newly installed Rust 1.85.0 minimal profile, the old guide's cargo fmt command failed with cargo-fmt not installed. Clippy was also not part of that profile. Installing components onto stable while invoking unqualified commands could select a different directory override.

**Rule:** install every required component explicitly and select the same toolchain in every command. The new guide pins 1.85.0 for tools/Edition practice. The runner records actual compiler/tool versions; invoking an installed executable does not prove a learner completed IDE navigation or diagnostics.

## 14. Fix under the old Edition, then change the manifest

The appendix overview and the practical guide previously contradicted each other: the overview changed edition before cargo fix. The corrected order follows the official Edition Guide: establish a baseline, run fix while the old edition is active, then change edition and the temporary MSRV, format and test.

A temporary gen identifier deliberately needs a rewrite to r#gen. This creates observable migration evidence instead of accepting a no-op command as a convincing exercise. Keep the before/after manifest and source diff. The actual project remains Edition 2021 / Rust 1.74.

Reference: https://doc.rust-lang.org/edition-guide/editions/transitioning-an-existing-project-to-a-new-edition.html

## 15. Temporary work must not erase previous learner evidence

The earlier PowerShell and POSIX examples reused fixed temporary directory names and recursively deleted those paths. A native command could fail without stopping the PowerShell sequence, and the final cleanup could remove the failed workspace before the learner saved evidence.

**Rule:** create one unique workspace per run, refuse an existing evidence destination, stop on a failed command, and retain the failed copy. Save logs and before/after evidence outside the disposable workspace before cleanup. Hash the original reference sources and root manifests before and after; absence of rust-toolchain files is also a state to preserve. The learner explicitly requests retention for IDE practice.

## 16. Verify why a negative example failed

The new array examples were checked directly with rustc as well as rustdoc: incorrect length and element type produced E0308; their corrected versions compiled. Variable/privacy examples produced E0384, E0308, E0070, E0603 and E0616. Missing imports must not accidentally make a compile-fail example look valid.

The Nightly exercise uses two Stable controls: one checks E0554 with the feature gate, and one removes that gate and checks the generic constant-expression rejection. A feature that later stabilizes should cause this exercise to need review. Tool installation failure or an arbitrary nonzero exit is not a passing negative test.

## 17. Build directories can leak through extension filters

After running the foundation lab locally, course_inputs included target incremental .lock files. An extension allow-list alone had treated compiler output as course source. A regression created real generated .rs and .lock files in a unique target child and reproduced the leak before the fix.

**Rule:** exclude target, .git and cache directories while enumerating every reference lab. Inspect the actual ZIP member names, contents and local links after building, not only in a clean checkout. Ship the practical runner with its guide in both source and learner archives.

## 18. Keep version identities and publication immutable

Main Run 220 used commit 34e39ebab056e1f893486cc1713e43f9efc33fa4. Its 17 verification/candidate jobs and checkpoint passed, but Publish immutable RC assets failed at Existing release tag points to another commit. Version v0.19.0-rc.1 already points to fed4a2d37a72de88dfb353474544efde9bdd537a and was published on 2026-11-04.

**Rule:** advance the current candidate to rc.2 and update VERSION, LabQuest/Cargo.lock and active download instructions together. Keep historical notes and evidence intact. Never force-move a release tag or overwrite old assets to turn a failed publication green. Engineering checks passing does not imply that a new downloadable release exists.

Evidence: https://github.com/clonetimer/RustAppLab/actions/runs/44107176066

## 19. Bind claims to the actual revision and evidence scope

Run 222 passed for the earlier practical-guide-only revision, but it did not execute migration or Nightly. New D/E Linux and Windows jobs execute the runner, and the candidate waits for those jobs. Nightly remains opt-in and never replaces the actual Rust 1.74 gate.

The runner's source snapshot has a narrow, documented scope: root configuration and the appendix lab inputs. It is not interchangeable with the full release source_id. Preserve exact commands, exit codes and log digests; verify the expected input snapshot when assembling a candidate.

## 20. Keep beginner tasks small and explain the result

Cargo vocabulary is now attached to the actual foundation package, executable and import names. Arrays and tuples use three scores; variables and modules use a small study plan. New exercises ask learners to predict, run, modify and explain rather than just copy a command.

The local foundation batch passed Rust 1.85.0 and actual Rust 1.74.0 Debug/Release with 29 runtime tests and 15 doctests. These are reference checks, not 44 completed learner objectives. Cross-platform and final candidate evidence must come from the current PR/main revision. Personal learning records remain unassessed until submitted.

## 21. Update acceptance floors when adding executable exercises

Independent review found that the native/MSRV gates still accepted the old foundation counts of 20 runtime tests and 2 doctests. The new batch actually contains 29 runtime tests and 15 doctests. Without updated floors, losing the new integration tests could still pass the old threshold.

A new run_step regression then revealed a second issue: the log includes its command line, and an unanchored test-summary regex counted a printed summary inside that command as another suite. A report of 20 tests was counted as 40. The regression failed with the old parser, then passed after matching only summary lines at line start in multiline mode. Floors are now 29 Debug, 29 Release and 15 doctests, with zero ignored tests.

**Rule:** keep acceptance thresholds aligned with the actual exercise set, and verify the evidence parser against command echoes and missing suites. A gate's own regression must prove it rejects the old incomplete batch, not merely compare constants. Exact final CI and artifact checks still decide publication.

## 22. Reject protected output paths before creating directories

A final review reproduced an absent-path edge case: using rust-toolchain.toml/logs as the evidence output created a directory named rust-toolchain.toml before the first snapshot. The run could then describe that newly created directory as unchanged. The same boundary applied to absent VERSION and rust-toolchain paths in a learner tree.

The behavior regression first reproduced all three false successes. The runner now rejects output at, or below, each protected root file path before mkdir and takes its baseline snapshot before writing. The regression then passed; ordinary CI and temporary evidence directories remain usable. All three real experiments were rerun with the final runner hash and their archived evidence refreshed.

**Rule:** capture the initial state before any write, and treat a protected file location as protected even when the file does not yet exist. A post-write baseline cannot prove that the source was never changed.

## 23. 记录 rc.2 已完成验收，并保留历史发布身份

2026-11-18 的 rc.2 验收已完成：[PR #10](https://github.com/clonetimer/RustAppLab/pull/10) 合并到 main，合并提交为 [ed255fe31ec1be2faba04ffb720660d13938e6a6](https://github.com/clonetimer/RustAppLab/commit/ed255fe31ec1be2faba04ffb720660d13938e6a6)。该提交的 [main Run 225 / 44243560861](https://github.com/clonetimer/RustAppLab/actions/runs/44243560861) 为 **21/21 成功**。

[v0.19.0-rc.2](https://github.com/clonetimer/RustAppLab/releases/tag/v0.19.0-rc.2) 的 **13 个发布资产**已逐项核对 SHA-256 与字节大小。[v0.19.0-rc.1](https://github.com/clonetimer/RustAppLab/releases/tag/v0.19.0-rc.1) 的标签和资产保持不变。本节补记最终验收结果；前文 Run 220 的发布失败及其修复经过仍作为历史保留。

**经验：**将合并提交、实际运行和发布资产绑定在同一条验收记录中。rc.2 的通过记录只证明该提交和该批资产，不能沿用为后续 rc.3 修改的验收结果。

## 24. 概念抽查须固定分母，不能从目录映射推导全书覆盖

现有 **79/81** 是目录条目的实作映射：79 项已有实作、1 项部分、1 项资料索引。一个目录条目可以关联到真实软件，却仍缺少其内部某个概念的讲解、失败对照或学习者任务。

本批固定抽查 **118 个概念目标**，当前记录如下：

| 状态 | 数量 | 含义 |
|---|---:|---|
| teaching | 87 | 已定位完整教学链 |
| partial | 25 | 已有部分教学，仍有明确子点未完成 |
| related | 2 | 仅定位到关联代码，不能当作完整教学 |
| gap | 4 | 尚未定位到该目标的教学证据 |
| 合计 | 118 | 仅为本轮抽查范围 |

**经验：**保持概念 ID、范围和状态定义稳定，再记录新增证据与剩余子点。不能把 87/118 外推为全书知识点完整率，也不能把目录映射的“缺口 0”解释为所有概念均已教全。工程证据与个人学习仍分开：学员是否完成预测、修改和解释，必须由其自己的记录证明。

## 25. 所有权目标需要失败对照，跨章材料需要先查再补

`Copy` 类型按值调用 `self` 后仍可使用原绑定，容易让正确的业务测试掩盖方法接收者的所有权区别。讲解 `&self`、`&mut self`、`self` 时，应选用含 `String` 的非 `Copy` 模型；读取、修改、消费各有成功行为，**消费方法调用后再次使用原对象**还必须有预期编译失败的对照。仅出现 `self` 签名，或成功调用一次 `into_title`，不足以证明这一教学目标完整。

当前 `into_title()` 消费后再使用原对象，是讲义中要求学习者重排调用的个人修改任务。本批已经自动核对的 E0382 控制是字段移动和结构体更新；不能把它们记为 `into_title()` 消费后再使用这一独立反例已自动执行。

补实验之前先跨章节核查已有材料。本轮 D3 的载荷枚举与模式解构、D4 的元组 newtype、D2 的析构实验都有直接证据，不能因为基础章节没有单独链接就误报为全仓缺失。可以增加基础路线导航和小练习；新增计数应来自实际补足的目标，而不是把旧代码再包装一次。

**经验：**检查“目标中的关键区别是否被教出”，不要只检查 API、关键词或测试总数。编译失败还要核对失败原因，避免缺少导入、错误路径等无关故障冒充所有权反例。

## 26. 旧译本的 API 描述要与实际类型合同核对

旧中文版用于原书章节和概念定位；涉及标准库接口时，仍须核读对应源码或当前官方签名，并在支持的工具链上验证。HashMap 练习应明确：`get` 返回 `Option<&V>`，`Entry::or_insert` 返回 `&mut V`。缺键、有键、已有值不被覆盖，以及通过可变引用修改值，是不同的可验证行为。

**经验：**在示例中写出关键返回类型并实际使用它们。仅统计查找或更新结果正确，可能遗漏“得到的是引用还是拥有值”的教学错误；旧译文中的措辞不能替代源码与编译器的类型合同。

## 27. CLI 组件、IDE 扩展与实际服务器是三个版本证据

`rust-analyzer` 的 Rust 工具链组件版本、编辑器扩展版本、编辑器实际启动的语言服务器版本可能不同。CLI 的 `rust-analyzer --version` 只能证明所调用的可执行程序；扩展可能使用自带服务器，也可能按配置启动另一份程序。记录 IDE 环境时应分别注明组件、扩展，以及实际服务器的版本或可追踪启动信息。

运行器提供 JSON 入口或生成操作记录模板，只证明命令、数据格式和工作区准备完成。GUI 中的跳转、诊断显示和修复结果仍需实际操作与记录；不能把 JSON 校验通过写成 IDE 验证通过。

**经验：**把“工具已安装”“CLI 可运行”“编辑器实际使用哪一份语言服务器”“GUI 操作已验证”拆成不同结论，避免版本号和证据层级互相替代。

## 28. 可编辑 IDE 记录与不可变证据分开，路径按来源平台解析

学习者需要继续填写的 `IDE_RECORD` 应放在带哈希的证据集合之外。另存不可变的 `ide-workspace.json`，固定本次保留工作区的定位信息和证据关联。否则，正常填写记录也会改变证据摘要，或迫使工具通过重算摘要掩盖原始证据被改写。

跨平台校验时，Windows 路径与 POSIX 路径应按各自语法解析，例如分别使用 `PureWindowsPath` 与 `PurePosixPath`。不能在 Linux 上直接用本机 `Path.is_absolute()` 判断 Windows 绝对路径，也不能把来源平台的合法路径误认成本机相对路径。语法解析和在实际运行平台上检查目录存在，是两项不同工作。

**经验：**不可变记录证明当时保存了什么，可编辑表单记录之后做了什么；路径字符串的解释由它所属的平台决定，而非由当前校验器碰巧运行在哪个平台决定。

## 29. 新手命令要包含目录准备，变体练习要同步所有相关断言

指导创建 `foundation-lab/examples/ownership_probe.rs` 前，应先说明创建 `examples` 目录；目录已存在时直接使用。初学者不应依靠猜测补齐命令所依赖的目录层级。反例一次只放一个，再换成对应修复重跑，便于定位诊断。

改变输入后，需要同步所有依赖该输入的断言。例如把陈旧偏移实验的新文本改成 `Rustacean Book` 后，旧偏移仍为 4，预览预期应为 `Some("Rust")`，重新提取首词的预期应为 `"Rustacean"`。只改其中一处，会把正确的概念变体变成无关的测试失败。

**经验：**写修改任务时同时指出输入、预期结果及需要一起更新的位置，让失败反映要学习的行为，而不是漏改测试数据。

## 30. rc.3 提交前检查的证据边界

本批新增概念实验在两套工具链上已实际执行完整 foundation 检查：

| 工具链 | Debug 运行测试 | Release 运行测试 | Doctest |
|---|---:|---:|---:|
| Rust 1.74.0 | 48 通过 | 48 通过 | 76 通过 |
| Rust 1.85.0 | 48 通过 | 48 通过 | 76 通过 |

本节写于 rc.3 首次提交之前，表中记录的是本地参考行为验证。**记录时尚无本批远端 CI 和发布结果。** 后续跨平台 CI、当前提交的交付检查和发布资产核对，应另记实际提交与运行结果；不能引用上一节 rc.2 的 Run 225 或 13 个资产，替代 rc.3 的验收。

**经验：**本地测试通过、远端 CI 通过、交付物可复现和发布完成分别记录。新增实验改变测试集合时，同步维护相应验收门槛，再由当前修订的真实运行决定下一阶段状态。
