# D9 · Rust 1.75–1.100 稳定增量审计（2026-11-25）

逐版查阅 Rust 官方 GitHub Releases 的 Language、Libraries、Stabilized APIs、Cargo 段落；下表 26 个 minor 版本均有代表性主题和课程归属，链接固定到该版本发布说明。安全、目标架构、编译器内部优化与兼容性修复不自动变成必修课；按教学价值选题。**这是发布级代表性审计，不是数百项 API 的穷尽覆盖。** A/B/D8 代表实验均需对应提交实际 CI；未实现内容保持延期，不能计入已验证范围。

| 版本 | 官方 Release Notes / 代表增量 | 教学映射与保留缺口 | 分类 |
|---|---|---|---|
| 1.75.0 | [发布说明](https://github.com/rust-lang/rust/releases/tag/1.75.0)；async fn/RPITIT；char/u32 ABI | D9-A step4；RPITIT 单独的捕获规则仍待专题 | 代表实验 A；其余延后 |
| 1.76.0 | [发布说明](https://github.com/rust-lang/rust/releases/tag/1.76.0)；Result/Option inspect；Arc/Rc unwrap_or_clone | D9-B step1；智能指针增量可后续迁移 | 代表实验 B |
| 1.77.0 | [发布说明](https://github.com/rust-lang/rust/releases/tag/1.77.0)；C-string literals；array each_ref/each_mut；cargo:: build directives | D9-B step2；FFI literals/build.rs 归 D11 | 代表实验 B |
| 1.78.0 | [发布说明](https://github.com/rust-lang/rust/releases/tag/1.78.0)；自定义 trait 诊断；Cargo lockfile v4 | Cargo 练习真实生成 v4；诊断属性待高级工具 | 代表实验 B |
| 1.79.0 | [发布说明](https://github.com/rust-lang/rust/releases/tag/1.79.0)；inline const；MSRV-aware cargo add；unchecked 算术 | D9-B step3；unchecked 与 cargo add 依赖解析另列 | 代表实验 B |
| 1.80.0 | [发布说明](https://github.com/rust-lang/rust/releases/tag/1.80.0)；LazyLock；独占范围模式；check-cfg | D9-B step4；cfg 与硬件目标另列 | 代表实验 B |
| 1.81.0 | [发布说明](https://github.com/rust-lang/rust/releases/tag/1.81.0)；expect lint；Duration abs_diff；core::error | D9-B step5；工具与 no_std 另列 | 代表实验 B |
| 1.82.0 | [发布说明](https://github.com/rust-lang/rust/releases/tag/1.82.0)；精确 RPIT 捕获；raw references；Option is_none_or | D9-A step2；裸指针归 D11 | 代表实验 A |
| 1.83.0 | [发布说明](https://github.com/rust-lang/rust/releases/tag/1.83.0)；BufRead skip_until；const mut；autolib | D9-B step6；const 及 manifest 自动发现后续 | 代表实验 B |
| 1.84.0 | [发布说明](https://github.com/rust-lang/rust/releases/tag/1.84.0)；Strict Provenance；CString FromStr；resolver v3 | D9-B step7；D8 resolver；unsafe 指针专题待补 | 代表实验 B/D8 |
| 1.85.0 | [发布说明](https://github.com/rust-lang/rust/releases/tag/1.85.0)；Edition 2024；async closures；AsyncFn 系列 | D8 和 D9-A step5；其他 API 单独选择 | 代表实验 A/D8 |
| 1.86.0 | [发布说明](https://github.com/rust-lang/rust/releases/tag/1.86.0)；trait upcasting；get_disjoint_mut | D9-A step3；D9-B step8b | 代表实验 A/B |
| 1.87.0 | [发布说明](https://github.com/rust-lang/rust/releases/tag/1.87.0)；Vec extract_if；precise capturing in traits；匿名 pipe | D9-B step8；RPITIT capture、pipe 后续 | 代表实验 B |
| 1.88.0 | [发布说明](https://github.com/rust-lang/rust/releases/tag/1.88.0)；let chains；HashMap extract_if；Cell update | D8 let chains；集合增量可迁移到 B8 | 代表实验 D8 |
| 1.89.0 | [发布说明](https://github.com/rust-lang/rust/releases/tag/1.89.0)；const 参数显式推断；文件锁；AVX512 | D9-B step3；文件锁跨进程与 SIMD 待 E1/D11 | 代表实验 B |
| 1.90.0 | [发布说明](https://github.com/rust-lang/rust/releases/tag/1.90.0)；unsigned checked_sub_signed；multi-package publish | D9-B step9；不执行真实 registry 发布 | 代表实验 B |
| 1.91.0 | [发布说明](https://github.com/rust-lang/rust/releases/tag/1.91.0)；Path file_prefix；build-dir；host-tuple | D9-B step10；构建路径和平台目标归 D11 | 代表实验 B |
| 1.92.0 | [发布说明](https://github.com/rust-lang/rust/releases/tag/1.92.0)；RwLockWriteGuard downgrade；zeroed allocation | D9-B step11；未初始化/zeroed 安全归 D11 | 代表实验 B |
| 1.93.0 | [发布说明](https://github.com/rust-lang/rust/releases/tag/1.93.0)；slice as_array；Vec/String into_raw_parts；VecDeque conditional pop | D9-B step12；所有权原始部件归 D11 | 代表实验 B |
| 1.94.0 | [发布说明](https://github.com/rust-lang/rust/releases/tag/1.94.0)；slice array_windows；Cargo config include；registry pubtime | D9-B step13、Cargo 配置与冷却 | 代表实验 B |
| 1.95.0 | [发布说明](https://github.com/rust-lang/rust/releases/tag/1.95.0)；match if-let guards；MaybeUninit / Cell arrays | D9-A step6；unsafe 初始化归 D11 | 代表实验 A |
| 1.96.0 | [发布说明](https://github.com/rust-lang/rust/releases/tag/1.96.0)；assert_matches / debug_assert_matches；NonZero ranges | D9-B step14 合同断言；范围与宏细节待扩展 | 代表实验 B |
| 1.97.0 | [发布说明](https://github.com/rust-lang/rust/releases/tag/1.97.0)；integer bit_width/isolate_lowest_one；Cargo build.warnings | D9-B step14 与 Cargo lint 对照 | 代表实验 B |
| 1.98.0 | [发布说明](https://github.com/rust-lang/rust/releases/tag/1.98.0)；str substr_range；slice subslice_range；NumBuffer | D9-B step15；低分配格式化后续 | 代表实验 B |
| 1.99.0 | [发布说明](https://github.com/rust-lang/rust/releases/tag/1.99.0)；Box<[T;N]> IntoIterator；debug profile；C variadic | D9-B step16 与 Cargo debug profile；FFI 归 E4 | 代表实验 B |
| 1.100.0 | [发布说明](https://github.com/rust-lang/rust/releases/tag/1.100.0)；Never、Allocator、DropGuard、ZST；Cargo lints/cooldown | D9-A steps7–9；D9-B Cargo lints 与回环冷却；ZST/ABI 归 D11 | 代表实验 A/B |

D9-A [九项语言实验](BOOK_D9_STABLE_FEATURES.md)；D9-B [标准库与 Cargo](BOOK_D9_STDLIB_CARGO.md)；D8 [Edition 2024](BOOK_D8_EDITION_2024.md)。GAT 在 1.65 稳定，是前置知识；不是本区间新增。1.100 的 Allocator 只纳入已稳定的核心接口，allocator_ext 仍按实际稳定标签处理。异步运行时、Pin/Waker/Stream 属 D10；Unsafe、Miri、跨语言、安全工具属 D11/E4。版本号记录首次稳定的相关能力，不声称在每个历史编译器上复测，工程统一固定 1.100.0。

官方参考：[Rust 版本公告](https://blog.rust-lang.org/releases/) · [详细 release notes](https://doc.rust-lang.org/releases.html) · [Cargo changelog](https://doc.rust-lang.org/cargo/CHANGELOG.html) · [1.100.0](https://blog.rust-lang.org/releases/1.100.0)。
