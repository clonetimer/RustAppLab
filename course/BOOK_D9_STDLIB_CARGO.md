# D9-B · 现代标准库与 Cargo 实战

[现代化计划](MODERN_RUST_ROADMAP.md) · [逐版增量索引](MODERN_STABLE_FEATURE_INDEX.md) · [工程入口](../projects/modern-d9-stdlib/README.md)

在 D9-A 后继续。将 projects/modern-d9-stdlib/learner 整体复制到个人目录，Rust 1.100.0 / Edition 2024。先看任务，再独立实现；一次只做一小步。在个人目录运行 cargo test --locked --offline --test contracts step1_，按编号替换，初始失败是学习起点。

| 组 | 步骤 | 能力及稳定版本 | 实现与迁移 |
|---|---|---|---|
| A 数据基础 | 1 | 观察解析结果（1.76） | 使用 Result::inspect / inspect_err 观察而不改变 Result；保持合法读数顺序。 |
| A 数据基础 | 2 | 固定数组校正（1.77） | 用 each_mut 借用各元素，饱和加法避免溢出。 |
| A 数据基础 | 3 | 泛型数组与 const 推断（1.79 / 1.89） | inline const 可构造非 Copy 元素数组；调用处可用显式推断 const 参数 _。 |
| A 数据基础 | 4 | 惰性初始化缓存（1.80） | 仅真正查询时 force，空输入不触发初始化；越界为 None。 |
| A 数据基础 | 5 | 计算采样间隔偏差（1.81） | Duration::abs_diff 避免把较小时间减较大时间而 panic。 |
| A 数据基础 | 6 | 读取带前缀的字节记录（1.83） | skip_until 跳过第一个冒号及其前缀；剩余内容原样读取；没有冒号时为空。 |
| A 数据基础 | 7 | 安全的 C 字符串转换（1.84） | FromStr for CString 拒绝内嵌 NUL，TryFrom<CString> 转回 String；不操作裸指针。 |
| B 数据与资源 | 8 | 无别名更新与异常提取（1.86 / 1.87） | extract_if 将匹配项移走且保持顺序；D9-A 已覆盖 1.86 upcast，另做 get_disjoint_mut 合同。 |
| B 数据与资源 | 9 | 无符号读数的有符号校正（1.90） | 正偏差应减去，负偏差相当于增加；超出 u32 范围返回 None。 |
| B 数据与资源 | 10 | 多重扩展名的文件标识（1.91） | file_prefix 区分第一个扩展名与最后一个扩展名；保留 OsStr，不强转 UTF-8。 |
| B 数据与资源 | 11 | 写锁原子降级为读锁（1.92） | 先写入，原子 downgrade，再在读锁保护下观察；不要先 drop 写锁再重新读取。 |
| B 数据与资源 | 12 | 动态切片转换为固定形状（1.93） | 长度必须严格等于 2；返回借用而非复制。 |
| B 数据与资源 | 13 | 相邻采样点变化量（1.94） | array_windows 直接给固定长度借用数组，先转 i64 防止 i32 极值差溢出。 |
| C 表示与诊断 | 14 | 位掩码统计与模式断言（1.96 / 1.97） | bit_width 是有效位宽；isolate_lowest_one 提取最低置位；在合同中使用 assert_matches!。 |
| C 表示与诊断 | 15 | UTF-8 分词位置（1.98） | substr_range 定位同一原文中的实际子串而不是搜索文本；返回字节范围。 |
| C 表示与诊断 | 16 | 消费堆上的固定数组（1.99） | Box<[T;N]> 的 IntoIterator 消费拥有型元素，不 clone；核对 String 原始分配。 |

第 8 步包含两个函数：extract_outliers 与 update_two。后者必须在两个下标合法且互不相同后才更新，失败保持原值。第 14 步合同使用 1.96 assert_matches! 验证结果；学员可在另建的迁移测试中使用该宏表达枚举和结构匹配。公开合同不强制语法，仍须解释所用 API，且不得改合同制造通过。

## 为什么这些 API 有实际价值？

inspect/inspect_err 保留原 Result，便于统计与诊断；each_mut、get_disjoint_mut、as_array、array_windows 能将借用边界与数据形状表达给类型系统。LazyLock 初始化一次，不能拿调用计数猜测并发时序。checked_sub_signed 与 Duration::abs_diff 防止简单校正操作的边界错误。RwLockWriteGuard::downgrade 保持原子锁转换；持有读锁期间不要尝试阻塞写锁。substr_range 返回实际子切片在原文中的字节偏移，并不是搜索相同文本，空子串可能产生不唯一位置。Box 数组按值迭代应转移 String 所有权而非复制。CString 的安全转换不能证明任意 C ABI 合同安全。

最后执行 cargo test --locked --offline --all-targets，再运行 cargo run --locked --offline --example telemetry -- 7 bad 120 0 ，先预测 good=3; bad=1; outliers=[120]; changes=[-7]。自己换一批输入、添加迁移测试。报告与个人理解分别记录。

## Cargo 工程诊断（个人独立副本）

复制 [cargo-practice](../projects/modern-d9-stdlib/cargo-practice/README.md)，运行 cargo generate-lockfile --offline。按 README 实际观察 1.78 lockfile v4、1.94 include、1.97 build.warnings、1.99 debug profile、1.100 Cargo 自身 unused_dependencies lint；Cargo lint 与 rustc lint 分开修复。删除依赖不是唯一修复，可真正使用需要的依赖；仅在所有相关构建目标都已检查时才据此判断无用依赖。该练习不修改用户全局 Cargo 配置，也不上传 crate。

## Cargo 1.100 依赖冷却

完整源码维护者运行 python scripts/modern_d9b_check.py --output d9b-evidence（Python 3.11+）。除真实 Rust 合同、RED/GREEN 和 Cargo 诊断外，脚本只在本机临时目录启动 HTTP sparse registry（127.0.0.1 随机端口），生成两个可编译合成 crate 版本和 SHA-256、pubtime。通过真实 Cargo 解析、锁文件和运行输出对照：

- 无冷却时选择新版本 1.1.0；开启 3 天冷却且新建锁文件时选择 1.0.0。
- 已有锁中 1.1.0 不会被自动降级；冷却时间不是漏洞扫描，不覆盖无 pubtime 的版本，也不适用于所有安装路径。
- 仅此回环实验允许 Cargo 联网模式，注册地址和下载地址都固定在本地临时服务；不连接 crates.io、不读取真实 registry token、不下载第三方代码。原始锁文件与日志留证。

普通标准库/诊断课程无需 Python。未安装工具链时先联网安装，之后无第三方依赖。D9-B 不宣称所有稳定 API 都已覆盖；真实版本增量、代表任务、后续安全/异步/硬件专题分别统计；真人观察和网页入口待验收。

官方参考：[Rust Release Notes](https://doc.rust-lang.org/releases.html) · [Cargo lints](https://doc.rust-lang.org/stable/cargo/reference/lints.html) · [Cargo config](https://doc.rust-lang.org/stable/cargo/reference/config.html) · [registry index](https://doc.rust-lang.org/stable/cargo/reference/registry-index.html) · [Rust 1.100 公告](https://blog.rust-lang.org/releases/1.100.0)。
