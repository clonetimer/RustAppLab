# B4 · Serde JSON 导出设计

日期：2026-11-02。基线：合并 PR #1 后的 `57af6ae`；其文件树与已通过 Windows/Linux/MSRV/浏览器 CI 的 `14bf462` 相同。

## 目标与范围

完成路线中既定的 B4：使用真实 serde / serde_json 将 TaskPocket 的 TSV 任务导出为 JSON，让学员练习第三方依赖、Cargo.lock、derive、feature 和公开 API 测试。保留现有 TSV 数据契约、业务库、CLI 和 Markdown 报告。JSON 导入、替换持久化格式和异步课程 C 不属于本轮。

用户要求继续推进，已有偏好为 feature 分支、验证、合并主干和阶段标签；本设计中的细节由实施者根据现有项目确定。

## 方案选择

1. 直接给现有 taskcore 加 optional Serde：改动较少，但 optional 依赖仍参与锁文件解析，不能保护全新缓存下 B1–B3 的离线体验。
2. 保留 legacy 清单、另建包含同一批源码的升级清单：可行，但形成两套业务包配置和升级规则。
3. **采用独立 JSON 适配工作区**：在现有工程内新增 `json-export/`，用相对路径复用 taskcore；原工作区的清单与锁文件保持纯 path 依赖。依赖边界和教学目的都清楚，代价是多一个 JSON 导出程序。

本轮采用方案 3。B4 通过显式准备命令填充 Cargo 缓存，后续检查使用 locked/offline。源码包交付固定锁文件，B4 首次准备需要网络；本轮不交付 vendor 离线介质。

## 固定接口与行为

`projects/advanced-b/{reference,learner}/json-export/` 是同构的独立单包工作区：

- package / binary：`taskpocket-json`，library：`taskpocket_json`，version `0.1.0`。
- Edition 2021，MSRV `1.74`，resolver `2`，Cargo.lock 格式 v3。
- `taskcore = { path = "../taskcore" }`；`serde = { version = "=1.0.228", features = ["derive"] }`；`serde_json = "=1.0.145"`。
- `strict-titles = ["taskcore/strict-titles"]`；default features 为空。
- `pub fn to_json(tasks: &[taskcore::Task]) -> Result<String, String>`。
- 使用本包的 DTO 和 `#[derive(serde::Serialize)]`，不为基础 Task 引入 Serde 依赖。
- 输出 JSON 数组，按 id 升序；每项仅有 `id: u64`、`done: bool`、`title: string`；空数组为 `[]`。
- 使用 serde_json 编码中文、引号、反斜杠和合法控制字符；不手工拼 JSON，不把整数转为浮点。
- 复用 taskcore 的快照校验规则：拒绝零/重复 ID、非法标题、会被静默 trim 的标题；strict 模式仍执行 80 字符规则；不修改输入。
- `taskpocket-json export-json <db>` 将 JSON 和一个结尾换行写 stdout；错误写 stderr、退出码非零、stdout 为空。文件不存在沿用空库语义，不创建文件。任何导出路径均不写入数据文件。

快照 API 的 silent trim 拒绝不改变 CLI 的现有 load/parse_tsv 语义：TSV 标题经基础解析器规范化后再导出。合法控制字符用标题内部的 NUL/退格覆盖；tab/CR/LF 继续按基础业务规则拒绝。

reference 提供完成实现、至少 10 个普通行为测试及 1 个 doctest；learner 提供可编译的 B4 TODO、相同可见测试和三级提示。固定平台合同来自受控模板，不接受学员自行修改的 tests 或 Cargo 清单。

## 学员流程和兼容

- 新 `advanced-b start <新目录>` 同时创建 json-export；默认 B1–B3 构建仍不解析它。
- 旧工程用 `advanced-b add-json <学员目录>` 显式添加扩展。只创建新的 json-export，目录已存在即拒绝；不覆盖既有源码或标记。
- `advanced-b prepare-json <学员目录>` 显式联网准备依赖。准备过程在临时目录使用平台固定 manifest、lock、reference 源码和测试，不执行学员源码/build.rs。
- 准备过程先 `cargo fetch --locked`，再在同一固定工程执行离线测试和 doctest。使用用户的正常 Cargo 缓存，不修改全局 Cargo 配置，不自动更新锁文件或工具链。
- 固定准备边界包括相对 path 依赖 taskcore 的 reference 清单和源码，不能指回学员目录或使用 learner TODO。
- 准备状态和每一步日志保存在有独立所有权标记的 `.advanced-b-dependencies/`；任何失败保存日志并返回非零；准备通过不是课程通过。
- `advanced-b check <目录> 1|2|3|4|all` 和 `learn check` 保持一致，all 表示 4；显式 3 保持原合同且在空 registry 缓存下可运行。
- `check 4` 先累计执行原 B1–B3；通过后才进入 B4，复制受控 JSON 清单/锁/测试和学员 JSON src，再离线执行默认与 strict-titles 合同及 doctest。
- B4 前用固定工程的 `cargo metadata --locked --offline` 识别依赖未准备；环境问题单独提示 prepare-json，不诊断为学员 TODO。B4 失败的 passed 为 3。
- metadata 失败须保留实际错误；工具缺失、锁不一致与缺缓存分别解释，不将所有失败都断言为缓存未准备。
- 所有目录名和断言兼容 Windows；源码扫描沿用普通目录/文件、拒绝符号链接、大小与深度限制。

## 报告与只读导航

新报告使用 Report v3，保持 `.applab-advanced-b` 工程标记 v1。`source_sha256` 继续只覆盖三个基础 src；B4 报告另有 `json_source_sha256`，覆盖基础 src 加 JSON src。仅 passed=4 必须有有效的 JSON 指纹。

旧 Report v1 继续提示重新检查 B3；Report v2 的有效 passed=3 仍证明 B1–B3，导航提示 B4 尚未检查。JSON 源码改变使 B4 记录过期，基础源码改变使累计记录过期。沿用“最新检查记录决定导航”的语义；例如 4 后检查 3，最新导航仍显示 B4 待检查。next/lesson/reading 不迁移文件、不下载依赖、不写报告。

## 工程验证与发布

- 根工作区、原 reference 工作区保持零 registry 依赖。
- 需要 Serde 缓存的端到端测试使用 labquest 的 `b4-integration` 测试 feature 和 required-features 测试目标；默认基础测试不依赖缓存。
- 完整门禁显式执行 B4：reference 的默认/strict Debug、Release、doctest、Clippy；平台 B4 E2E 单独执行。检查脚本不自行联网；CI 在门禁前单独 fetch 固定图。
- 必须实际验证 Rust 1.74、Windows MSVC、Linux，并核对源码指纹、普通测试数量和零 ignored。真实变体包括写死 JSON、错误字段类型、CLI 缺失/破坏、旧阶段回归、修改可见 tests、旧报告与源码过期。
- 分发清单收录新增源码/清单/锁/教材。更新 Book 对照入口、B4 中文讲义、修改位置、三级提示、个人数据练习、Windows 指令和当前路线状态。
- 主干全部 CI 成功后，独立低权限边界的标签 job 才拥有 contents:write；只在 push main 运行，创建 `checkpoint/v<VERSION>-<commit前12位>`，不覆盖已有 tag，不发布 Stable。PR 触发时跳过写入。

## 资料

- https://serde.rs/derive.html
- https://docs.rs/crate/serde/1.0.228/source/Cargo.toml
- https://docs.rs/crate/serde_json/1.0.145/source/Cargo.toml
- https://doc.rust-lang.org/cargo/reference/resolver.html
- https://doc.rust-lang.org/cargo/commands/cargo-fetch.html

完整传递版本以 Cargo 生成的 v3 锁文件和真实 Rust 1.74 测试为准，不根据顶层 MSRV 推断整个图。
