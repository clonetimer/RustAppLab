# 进阶 B · 从可以运行到可以复用

## B1 数据层（数据契约）

先阅读 `taskcore/src/lib.rs`，并用测试定义数据格式。`Task`、`Summary` 属于业务层，不应由 CLI 决定它们的含义。学员需解析损坏行、重复 ID、非法状态、中文和空数据文件。完成后讨论：什么该返回 `Result`，什么可以用普通值表达。

## B2 业务层（功能变更）

实现 `add`、`complete`，确保错误发生时已有数据不被改变。观察闭包/迭代器写法和手写循环各自的清晰度。设计一条自己想要的规则（例如标题去重），先补测试，再改业务代码。

## B3 工程交付（两程序复用）

同一份 `taskcore` 同时供 CLI 与报告程序使用。实际操作：add 任务、完成任务、新进程 list/stats、Markdown 报告。解释为何 `taskreport` 应依赖业务库而非 CLI。为新的命令增加回归测试，确认坏数据不会被悄悄覆盖。

## Cargo 工具实践

```powershell
cargo metadata --no-deps --format-version 1
cargo tree --workspace
cargo test --offline --workspace
cargo test --locked --offline -p taskcore --test feature --no-default-features
cargo test --locked --offline -p taskcore --test feature --no-default-features --features strict-titles
cargo test --locked --offline -p taskcore --doc
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

`feature.rs` 对比 80/81 个 Unicode 字符的标题边界，并检查解析和新增任务是否共享标题规则。B3 自动验收会分别运行上面的两种 feature 配置。测试目录中的文件是集成测试，`///` 中可编译运行的示例是文档测试；`cargo test --all-targets` 不会代替 `cargo test --doc`。

注意 feature 可能因依赖图而合并；独立运行某个包的行为不等于完整 Workspace feature 组合。

## B4 第三方依赖与只读 JSON 导出

B4 在 `json-export/` 内提供独立 Workspace，使用真实的 Serde / serde_json；它通过 `taskcore = { path = "../taskcore" }` 复用业务规则。根 Workspace 与原有 B1–B3 Workspace 保持纯 path 依赖，初次学习 B1–B3 不需要下载 registry crate。B4 仍使用 Edition 2021 / Rust 1.74，固定 v3 格式 `Cargo.lock`；不要为解题升级工具链或重生成锁文件。

### 创建、迁移与显式准备

在 **Rust AppLab 平台根目录**执行。新工程已包含 JSON 骨架；已有旧工程只运行第二条命令，不必重建前三阶段：

```powershell
cargo run --locked --offline -p labquest -- advanced-b start "my-taskpocket-pro"
# 仅旧工程需要；已有 json-export 时拒绝覆盖
cargo run --locked --offline -p labquest -- advanced-b add-json "my-taskpocket-pro"
# 明确联网下载固定依赖，然后执行固定参考合同；先完成这一步
cargo run --locked --offline -p labquest -- advanced-b prepare-json "my-taskpocket-pro"
```

`cargo run --offline` 只约束启动平台本身；`prepare-json` 是明确授权联网的子命令，它内部执行 `cargo fetch --locked`。准备在临时目录中使用平台固定的 manifest、lock、reference 源码、taskcore 与测试，**不执行学员源码或学员 build.rs**。依赖进入当前用户正常的 Cargo 缓存，不修改全局配置。准备日志保存到学员工程的 `.advanced-b-dependencies/`，失败也保留日志；准备成功只证明依赖环境可用。

切换工具链、机器、`CARGO_HOME` 或删除缓存后可能需要重新准备。没有网络时先继续显式 B3 检查，不要删除锁文件或把“准备成功”当作 B4 成绩。缺缓存、锁文件错误和缺失 Cargo 的实际诊断会保留；根据错误修复环境后再检查。

### 编辑位置与合同

编辑 **自己工程的** `json-export/src/lib.rs` 和 `json-export/src/main.rs`。库接口固定为：

```rust
pub fn to_json(tasks: &[taskcore::Task]) -> Result<String, String>
```

创建本包 DTO（面向输出的数据结构），使用 `#[derive(serde::Serialize)]`，只包含 `id: u64`、`done: bool`、`title: &str` 或 `String`。保持基础 `Task` 无 Serde 依赖。先验证快照，再把 DTO 按 id 升序排列，最后交给 `serde_json::to_string` 编码。不要手拼 JSON，也不要把 u64 经由 f64 转换。

JSON 顶层为数组，空输入为 `[]`。拒绝零/重复 id、非法标题和需要 trim 才合法的快照标题；启用 `strict-titles` 后沿用 80 个 Unicode 字符上限。库函数只借用输入，不改变原数组顺序。注意：CLI 的 TSV 解析沿用 taskcore 的规范化语义；库接口对直接传入的快照禁止静默 trim。

CLI 固定为 `taskpocket-json export-json <db>`：成功只输出 JSON 和一个结尾换行；错误只写 stderr，退出非零且 stdout 为空。数据库不存在时输出 `[]`，不创建数据库。成功、参数错误和坏数据错误均不写回 TSV；这里只做导出，未提供 JSON 导入或存储格式迁移。

### 分级提示

1. **提示 1：先预测。** 画出 TSV → `taskcore::load` → `Vec<Task>` → DTO → JSON 的数据流。给每个步骤写一条坏输入与预期错误；判断哪一层负责排序、转义和打印。
2. **提示 2：缩小实现。** 用集合记录 id，复用 `taskcore::validate_title`，比较验证后的标题是否等于原字符串。DTO 可借用标题，排序 DTO 不需要修改原任务。CLI 在全部成功后再写 stdout。
3. **提示 3：核对类型。** DTO 的 `id` 必须仍是 `u64`、`done` 必须是 `bool`。用 `serde_json::to_string` 序列化 DTO 数组，并把序列化错误转换成 `String`。CLI 用 `args_os` / `PathBuf` 接收路径，向上传递加载、导出和输出错误。

### 本地运行与累计检查

在 **自己的学员工程根目录**执行，初始 TODO 会失败：

```powershell
cargo test --locked --offline --manifest-path json-export/Cargo.toml --all-targets
cargo test --locked --offline --manifest-path json-export/Cargo.toml --all-targets --features strict-titles
cargo test --locked --offline --manifest-path json-export/Cargo.toml --doc
cargo test --locked --offline --manifest-path json-export/Cargo.toml --features strict-titles --doc
cargo run --locked --offline -p taskpocket-pro -- add tasks.tsv '学习 "Serde" 与反斜杠'
cargo run --locked --offline --manifest-path json-export/Cargo.toml -- export-json tasks.tsv
```

回到 **平台根目录**执行：

```powershell
cargo run --locked --offline -p labquest -- advanced-b check "my-taskpocket-pro" 4
cargo run --locked --offline -p labquest -- advanced-b check "my-taskpocket-pro" all
# 只验证原 B1–B3；不需要 Serde 缓存
cargo run --locked --offline -p labquest -- advanced-b check "my-taskpocket-pro" 3
cargo run --locked --offline -p labquest -- learn check "my-taskpocket-pro"
```

`4`、`all` 和 `learn check` 都累计验证到 B4：先回归 B1–B3，再离线运行 B4 默认/严格配置和 doctest。平台固定 manifest、lock 和测试，修改个人 tests 不能绕过合同。B4 失败最多保留 passed=3。旧 v2 的 B3 报告仍只证明 B1–B3；新 v3 的 B4 报告包含 JSON 源码指纹。JSON 源码修改后 B4 记录过期；基础源码修改使累计记录过期。最新一次只检查 3 时，导航仍提示 B4 待检查。

`next`、`lesson`、`learn reading` 只读，不迁移目录、不下载依赖、不写成绩；`add-json`、`prepare-json` 和检查均须显式运行。教学检查用于自己的可信代码，不是恶意程序沙箱。

### 用个人数据验证理解

用虚构的个人任务（避免真实账号、密钥和敏感信息）加入中文、引号、反斜杠、已完成/未完成状态，先画出 JSON，再实际导出。给 API 测试加入 `u64::MAX` 和无序 id，确认 id 数字精确且输出有序；合法 NUL/退格应被编码，tab/CR/LF 继续按基础标题规则拒绝。分别验证 80/81 个中文字符的 strict 边界、缺失数据库及坏 TSV。比较导出前后的文件字节，给出只读证据。

复盘：为什么 DTO 属于适配包？derive 在编译期生成什么？启用适配包 feature 如何传到 taskcore？Cargo.lock 与 Cargo.toml 分别约束什么？只通过准备、只运行可见测试、修改源码后沿用旧报告为什么都不能证明 B4 通过？

阅读：[Serde derive](https://serde.rs/derive.html)、[Cargo fetch](https://doc.rust-lang.org/cargo/commands/cargo-fetch.html)、[Cargo resolver](https://doc.rust-lang.org/cargo/reference/resolver.html)。先用 平台的 `course/BOOK_GUIDE.md` 对照指南 理解模块、trait、错误传播和测试，再回到自己的公开接口验证。

## 质量范围

当前教学型文件保存未承诺断电原子性、多进程并发安全、符号链接竞争安全；真实备份应使用 SafeSync，不应仅靠这份 TaskPocket 数据文件。
