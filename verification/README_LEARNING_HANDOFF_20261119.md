# README 与 FirstCard 学习衔接修复记录 · 2026-11-19

## 本轮解决的问题

用户反馈：各项 README 太简略，零基础完成 FirstCard 后不知道继续做什么，也说不清学到了什么。

基线为主干 `42387c5`。此次保持既有课程与评分合同，补齐教学入口、课内操作、能力复盘和课间衔接。没有增加课程数量或改写学员答案。

## 开发经验

### 1. 入口必须给出唯一的近期方向

- 原始 Windows 初学指南仍把 FirstCard 学员送往 FileQuest，较新的课程首页则推荐 UnitLab。目录里有材料，不等于学员能找到正确下一步。
- 统一近期路线为 FirstCard → UnitLab → QuizDeck → TextScope；新建 `course/AFTER_FIRSTCARD.md`，提供复盘、自测、完整命令、编辑位置和下一课用途。
- Book 专题和原有 00–31 章节保留为按需补课、拓展。Windows 作品菜单按主线顺序排列。

### 2. README 应说明学习任务，不能只是目录索引

- 17 个应用都有 README，其中 5 个此前没有。12 款 Studio 对照实际源码写出目标、前置基础、三个函数、运行命令、预期结果、个人实验与下一站。
- 进阶 B/C1/C2、7 个 Book 参考工作区、5 个样例目录分别说明学员练习、参考实验、数据产物的不同角色。
- FirstCard 与 Studio 原先不生成 README；现在新工程附带可移植 README，课程数据与现有任务卡复用，不嵌入构建机器路径。
- 旧工程不重建、不覆盖；没有新 README 时使用原有 START.html 或平台的 learn lesson。已有 FIRST_WIN.md 保留，成功检查仍打印新的后续指引。

### 3. “通过了检查”与“理解了知识”分别说明

- FirstCard 的参数读取和默认值逻辑已经提供，学员亲手练习的是变量参与格式化输出和编辑、保存、运行、比较的循环。
- 每课要求自己预测新输入、验证错误输入、解释一个函数、记录用过的提示；不由软件自动填写理解程度或代码作者身份。
- 个人新增测试必须先用学员工程的 cargo test 运行；平台固定合同不会运行学员新增的测试文件。衔接页已给出两条不同用途的检查命令。

### 4. 从真实生成物和失败路径检查文档

- 新增回归先在旧行为下失败：FirstCard / Studio 缺少 README；首次失败后的提示只有不可直接运行的子命令。
- 修复后测试从输出提取完整命令，真正创建下一款工程、读取任务卡、运行初始失败和提示；覆盖中文及空格路径，检查只读导航与旧记录保护。
- 实际 ZIP 暴露 course/studio 的 12 个文件被两个收集循环重复写入；保留 course_inputs() 的统一来源，删除重复收集，并在实际归档检查中拒绝重复文件名。
- 仓库里能点击的链接不一定在 learner-kit 中存在。新增衔接页和相应讲义一并收录；Book 参考入口链接到实际随包交付的 course 材料。

## 已运行的验证

Linux，实际 Rust 1.85.0。以下为本轮工程验证，不是真人学习效果统计。

| 检查 | 实际结果 |
|---|---|
| `cargo fmt --all -- --check` | 通过 |
| `cargo test --locked --offline -p labquest --lib --test beginner_e2e --test learning_e2e --test studio_e2e` | 113 项通过：65 单元、10 FirstCard、12 学习导航、26 Studio；0 失败 |
| `python -m unittest scripts.tests.test_learner_kit scripts.tests.test_learning_session scripts.tests.test_closeout -q` | 24 项通过 |
| `python scripts/structural_check.py` | 课程材料、源码清单和 Markdown 本地链接检查通过 |
| `git diff --check` | 通过 |
| FirstCard → UnitLab → QuizDeck 衔接 | 实际 CLI 回归通过；下一款未完成工程正确失败，没有把创建等同于完成 |
| 初次失败后的提示命令 | 先观察两项回归失败，修正完整 Cargo 命令后纳入上述 113 项通过结果 |
| START_HERE.html 链接与锚点 | 12 个目标均有效；未做截图验收 |
| `cargo build --locked --offline --release -p labquest` | 实际 Release 构建通过 |
| `python scripts/package_learner_kit.py` | 实际生成学员包；373 个唯一文件，ZIP 校验通过 |
| 实际学员包入口链接 | 检查首页、FirstCard、12 款 Studio README 和衔接页共 15 份入口，120 个本地链接均存在；没有重复 ZIP 文件名 |
| StoreDesk README 的数据库实验 | 使用实际系统 SQLite：PEN=10，调整到 8，含不存在 BOOK 的批次失败后仍为库存 8、1 条流水 |
| NetFetch README 的双终端实验 | 通过真实 LocalHub 服务下载 JSON；重复路径拒绝覆盖，HTTP 错误不留下输出文件 |

运行数据库回归时，容器已有真实 `libsqlite3.so.0`，但缺开发链接名且不能安装系统包。在 `/tmp/rustapplab-sqlite-linker/` 为该系统库建立仅本次使用的开发链接，并以 `LIBRARY_PATH` 传给 Cargo；未替换 SQLite 实现或修改评分合同。首次缺库导致的链接失败不被记为教学代码缺陷。

## 验证边界与交付状态

- Rust 1.85.0 的 Clippy 检查在未修改的 `apps/safesync/src/sha256.rs:148` 报 precedence，在 `--no-deps` 检查中又在未修改的 `apps/labquest/src/capstone.rs:159` 报 let-and-return。这两个文件与基线相同；本轮没有扩大范围修复它们，也不宣称 Clippy 全绿。
- 本机 Chromium 因进程 socket 权限限制无法启动；HTML 仅完成结构、链接与锚点检查，没有生成浏览器截图。
- 当前环境没有 Windows 原生 PowerShell 运行证据。Windows 菜单顺序、编码与入口静态检查不能替代 Windows CI。
- 自动审批审查拒绝了 GitHub 推送检查，理由为当前请求未明确授权把改动发布到远端。因此本轮截至本记录只有本地改动与校验，没有新 PR、远端 CI、合并或 Release；准备完整可审阅补丁后再确认远端操作。
- 此记录不把已有 RC、历史截图或既往 CI 继承为当前源码的跨平台发布结论。

## 2026-11-19 后续：授权与远端交付

上节记录的是首次整理时的状态。随后用户明确回复“推送”，本轮已将同一源码树通过已连接的 GitHub 通道提交到修复分支，并创建 [PR #12](https://github.com/clonetimer/RustAppLab/pull/12)。远端提交为 `a374ae495659add5f5fa092b44749a6ab5528541`，树 `416e22b969dcf5cc556298dcffd8103f567a1c98` 与原本地提交 `e905e9b` 完全一致。

现有 Windows/Linux CI 已启动；后续文档归档还会产生新的提交，最终结果须查看该 PR 最新提交的检查，不能沿用上节的“尚未推送”状态，也不能把首个提交的检查直接继承给后续修订。文档整理的本地检查见 [整理记录](DOCUMENTATION_HOUSEKEEPING_20261119.md)。本轮未合并主分支、改写版本标签或发布 Release。
