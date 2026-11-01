# 第 24 关 · CASE 01：一次需要交付证据的软件故障调查

**中心任务**：不是阅读一份“故障已修复”的故事，而是在真实 Rust 文件、日志、命令和备份之间找出因果关系，最后交付一份可以重新验收的调查包。

**会使用的软件**：LabQuest（任务进度）、FileQuest（文件地图）、LogLens（事故证据）、SafeSync（备份封存）。LocalHub 是选做的浏览器观察工具。

## A. 开局：你的第一张案情板

在 `RustAppLab/` 根目录执行：

```sh
cargo run -p labquest -- start labquest-playground
cargo run -p labquest -- next labquest-playground
cargo run -p labquest -- board labquest-playground
```

用浏览器打开 `labquest-playground/deliverables/CASE_BOARD.html`。这时应该是 **0/6**，并且包含一条可采取行动的提示。打开 `labquest-playground/BRIEFING.md` 阅读事故背景。

观察 `project/`、`logs/`、`deliverables/` 与 `backups/`，预测各自负责什么。`labquest-playground/project/` 是可独立编译的 Rust 应用，不是几段伪代码。

> 如果目录已存在，`start` 会报错并保留原文件；请换一个目录名，把后续命令的 `labquest-playground` 一并替换。所有文件修改和损坏实验仅限新建案例，不触碰真实资料。

## B. 找文件：先看现场，不急着猜根因

```sh
cargo run -p filequest -- report labquest-playground/project -o labquest-playground/deliverables/inventory.html
cargo run -p labquest -- status labquest-playground
```

打开 `inventory.html`，寻找 `src/main.rs` 和 `config/service.conf`。问自己：配置和程序实现之间的关系是什么？发生过改动后，旧文件地图可能失效，验收会据此提示重新生成。

## C. 取证：理解统计口径

```sh
cargo run -p loglens -- compare labquest-playground/logs/baseline.log labquest-playground/logs/incident.log
cargo run -p loglens -- report labquest-playground/logs/incident.log -o labquest-playground/deliverables/incident.md
cargo run -p labquest -- status labquest-playground
```

基线 12 条日志、0 条 ERROR；事故日志 12 条、11 条 ERROR（91.67%）。这些是**人工生成的日志记录统计**，不代表任何线上服务的真实请求失败率。解释为什么 91.67% 不能被直接写成“系统可用率下降 91.67%”。

## D. 修复并验证：先想办法，再看下一条线索

用文本编辑器打开 `labquest-playground/project/config/service.conf`。注意 `timeout_ms` 与日志里的 `latency_ms`：哪一个是判断 ERROR 的临界值？先尝试自己的改法；需要提示时运行：

```sh
cargo run -p labquest -- hint 3 1
cargo run -p labquest -- hint 3 2
cargo run -p labquest -- hint 3 3
```

改动配置后，再运行：

```sh
cargo run -p labquest -- simulate labquest-playground
cargo run -p loglens -- report labquest-playground/logs/current.log -o labquest-playground/deliverables/current.md
cargo run -p labquest -- status labquest-playground
```

当你再次修改配置、使已有报告过期时，LogLens 报告命令需要在确认输出路径正确后加 `--force`。**只改 `current.md` 的文字不会计入修复成功**，系统会重新计算日志与配置。

## E. 代码任务：真正改变 Rust 应用

你现在可能已通过前四项，但还没有亲手编写新功能。打开 [`第 25 关：亲手补全 Rust 程序`](25_labquest_rust_code.md)。在不阅读完整答案的前提下，使用原来的 Rust app 新增动态 SUMMARY 输出，再执行：

```sh
cargo run -p labquest -- test-app labquest-playground
```

0.8.0 新案例还会验证 150/400/750ms 三种输入，不能写死统计数。可先用 `labquest practice` 和 `labquest predict` 检查边界规则，详细参考[第 26 关](26_transfer_verification.md)。

修改 `main.rs` 后原来的 FileQuest 文件地图已经过时，必须重新生成：

```sh
cargo run -p filequest -- report labquest-playground/project -o labquest-playground/deliverables/inventory.html --force
```

`test-app` 显式调用真实 Cargo，并将构建缓存放在 `labquest-playground/build`，**不**会把编译缓存混入你的源码快照。它不是在线判题服务，不会上传你的代码。

## F. 封存并交付

```sh
cargo run -p safesync -- backup labquest-playground/project labquest-playground/backups/project-snapshot
cargo run -p safesync -- verify labquest-playground/backups/project-snapshot
cargo run -p labquest -- status labquest-playground
cargo run -p labquest -- board labquest-playground
cargo run -p labquest -- finish labquest-playground
```

现在打开 `labquest-playground/deliverables/CASE_BOARD.html` 和 `CASE_REPORT.md`。**完成标准**：六步全通过、产生程序运行证据、SafeSync 校验通过、生成最终报告。请保留 `project/`：它本身就是一件可以独立修改和发布的小型 Rust 软件作品。再次执行 `finish` 应失败，而非覆盖原报告。

## 反事实实验（只在新案例上做）

完成首次通关后，创建 `labquest-second-case`，尝试修改 `config/service.conf` 但不重新运行 `simulate`，以及修改 `main.rs` 但不重新运行 `test-app`。观察是哪一关从 OK 变成待行动，解释**数据、报告和证据为何需要与当前代码保持一致**。

## 选做：浏览器观察站

```sh
cargo run -p localhub -- serve --root labquest-playground/project --log labquest-playground/logs/incident.log
```

在本机打开 `http://127.0.0.1:8765/`，观察文件统计与日志数据。它是可选的本机只读观察工具，不能代替 LabQuest 的六项正式验收；改变日志输入需要停止并重启服务器。
