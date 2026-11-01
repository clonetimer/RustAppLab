## 0.11.1 零基础第一课

还不会编程，不要从 CASE 01 或 TaskPocket 开始。先打开项目根目录 `START_HERE.html`，Windows 可以双击 `START_WINDOWS.cmd`，或在项目根目录运行：

```sh
cargo run -p labquest -- beginner start my-first-card
# 修改 my-first-card/src/main.rs 中唯一一行 println! 并保存
cargo run -p labquest -- beginner check my-first-card
cargo run -p labquest -- beginner hint 1
```

四种输入通过后生成 `my-first-card/FIRST_WIN.md`。编译与功能验收只有你显式运行时才进行；请只检查自己信任的代码。

## 0.10.0 新增：PageMint 独立迁移作品

执行 `cargo run -p labquest -- app catalog` 查看现有两款独立作品。`app init pagemint <新目录>` 创建一个可编译但未实现的离线笔记网站工程；`app check pagemint <目录>` 使用真实编译、多个临时输入目录及 HTML 检查给出反馈；`app hint pagemint 1` 提供逐层提示。

与 TaskPocket 的任务持久化不同，PageMint 要求把多个 UTF-8 笔记文件转为不引用远程资源的 HTML 文档，必须正确进行 HTML 转义、排序和错误处理。详见根目录 `course/30_second_independent_product.md` 和 `course/31_transfer_release.md`。不依赖第三方 crate，且不会默认修改用户笔记。旧 `app init/check/hint` 仍兼容 TaskPocket。

---

## 0.9.0 · TaskPocket 独立作品路线（新增）

第 24–27 关沿用原 CASE 01 调查任务；第 28–29 关升级为学员自己从零实现的一款真实 CLI 程序。**学员代码是单独的 Cargo 工程，不再修改 LabQuest 的案例源码。**

在 `RustAppLab/` 根目录：

```sh
cargo run -p labquest -- app init my-taskpocket
cargo run -p labquest -- app check my-taskpocket    # 初始应该失败
cargo run -p labquest -- app hint 1
```

真正需要实现的行为、TSV 格式与异常协议见自动生成的 `my-taskpocket/CONTRACT.md`；每次 `check` 会编译并运行学员编写的程序，输出九组实际运行结果与 `APP_ASSESSMENT.md`。该执行不是沙箱，不可用于不可信代码。`init` 只创建全新目录，拒绝覆盖；`check` 不修改学员的源文件和任务数据库，构建缓存保存在 `.applab-target/`，测试数据放在系统临时目录。

详细教程：[第 28 关](../../course/28_independent_product.md) · [第 29 关](../../course/29_product_release.md)。

---

# LabQuest · 第一个可通关的 Rust 软件调查案例

LabQuest 不是另一套抽象语法题，也不是安全沙箱。它是 Rust AppLab 从 0.7.0 起提供的 **案例式学习控制台**，让 FileQuest、LogLens、SafeSync 在一次完整交付中协作，并要求你实际修改、编译和运行 Rust 小程序。

## 一分钟上手

在 `RustAppLab/` 根目录、已安装 Rust/Cargo 的终端执行：

```sh
cargo run -p labquest -- start labquest-playground
cargo run -p labquest -- next labquest-playground
cargo run -p labquest -- board labquest-playground
```

打开 `labquest-playground/deliverables/CASE_BOARD.html`。你会看到六个未完成目标和一条线索；它是本地静态 HTML，不需要服务器。`start` 只允许全新目录，第二次请换名称，例如 `labquest-second-case`。

完整通关步骤：[第 24 关](../../course/24_labquest_case.md)；真正动手改源码：[第 25 关](../../course/25_labquest_rust_code.md)；[多输入迁移验收](../../course/26_transfer_verification.md)与[下一步引导](../../course/27_guided_learning_loop.md)。

## 六个可观察目标

| 步骤 | 真实输出 | 为什么有用 |
|---|---|---|
| 1. 文件现场地图 | `deliverables/inventory.html` | 知道代码和配置在哪里，改动后报告必须重新生成 |
| 2. 事故取证 | `deliverables/incident.md` | 不是根据剧情描述猜测错误率，而是用 LogLens 重新计算 |
| 3. 配置修复 | `logs/current.log` | 通过配置和模拟规则重新产生日志，不能简单写一个“已恢复”文本 |
| 4. 独立复验 | `deliverables/current.md` | 使用第二款软件再验证当前日志 |
| 5. 亲手写 Rust | `logs/program.log`、`deliverables/PROGRAM_PROOF.txt` | 修改独立 Rust 应用，`test-app` 真实 Cargo 编译和运行后才计入 |
| 6. 作品封存 | `backups/project-snapshot/` | 使用 SafeSync 验证字节与当前源码、配置一致 |

新建案例会使用 **四种阈值输入**验收 Rust 程序：当前服务配置以及 150/400/750ms。固定写死 ERROR 数量的代码不能通过，且所有测试不会改动练习案例的配置或日志。0.7.0 创建的旧案例继续保持原单配置验收，便于已有学员完成进度，但旧证据不被冒充为新多输入验收。

六步都通过后，`labquest finish <案例目录>` 生成 `deliverables/CASE_REPORT.md`。完整交付还包括静态进度板、分析报告和可重新编译的 `project/`。

## 命令及边界

```text
start <新目录>         创建案例（不覆盖）
status <案例目录>     只读检查六项证据，不执行学员程序
next <案例目录>       根据实际进度给下一步建议、相关 Rust 知识和命令
practice              查看三道判断边界预测题
predict <阈值> <数量>    检查你预测的 ERROR 数（无磁盘写入、不计分）
board <案例目录>      生成/刷新离线 HTML 进度板
simulate <案例目录>   根据配置重新生成案例内 logs/current.log
test-app <案例目录>   使用 Cargo 编译并在多阈值条件下运行案例内 project/（仅新案例）
hint <1-6> [1-3]    逐层提示，第三层包含可执行命令或代码线索
finish <案例目录>     六项完整后生成 CASE_REPORT.md，拒绝覆盖已有报告
```

- **对新手更友好的反馈**：`next` 只读取进度与文件，告诉你为什么还没通过以及如何开始下一步；`predict` 适合先思考再看答案，不能代替 Rust 代码编写。
- **构建与执行上限**：新版 `test-app` 编译一次，顺序运行当前配置与三个覆盖输入。编译最多等待 120 秒，每种输入的运行最多等待 5 秒；超时会尝试终止直接子进程并给出诊断。该限制不能替代进程树隔离或安全沙箱。
- **学习案例的源代码会被执行**：`test-app` 只有在你主动调用时才编译运行；只应对你自己创建或审查过的案例运行。`status` 和 `board` 不会执行 Rust 代码。
- **证据而非认证**：`PROGRAM_PROOF.txt` 保存整个 `project/` 文件树和程序输出的 SHA-256 摘要以发现普通改动；不包含签名、不防恶意伪造，也不是生产级代码审计。
- **不触碰真实业务数据**：模拟日志和时延固定，故障/修复结论仅适用于 12 条教学样例。变大超时时间并不等于修复了真实系统的性能根因。
- **保护已有文件**：`start`、`finish` 不覆盖；`simulate`、`board` 和 `test-app` 只更新案例受管输出并拒绝符号链接。`build/` 放案例小程序编译缓存，不会出现在备份的 `project/` 中。
- **工具链要求**：LabQuest 案例中的 `project/` 是一个完整 Rust crate；因此即使使用打包的 Linux `labquest` 二进制，执行 `test-app` 仍需要本机安装 Cargo 与 Rust 编译器。
