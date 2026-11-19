# ServiceDesk · 用浏览器操作自己的工单

**这次要做成什么：**在本机网页新增、关闭工单，重启后数据还在，并能输出供客户端读取的 JSON。你负责业务规则，HTTP 与日志外壳已提供。

**开始条件：**完成 [NetFetch](../netfetch/README.md)，理解客户端、HTTP 响应和错误；还应能解释 TermGarden 的状态更新。

本目录是已实现的参考软件；你的练习代码放在下面新建的 `my-servicedesk/` 中。

## 1. 看目标，创建自己的工程

在 **RustAppLab 根目录**（能看到根 `Cargo.toml`）运行：

```powershell
cargo run --offline -p servicedesk -- demo reference-service
cargo run --offline -p servicedesk -- list reference-service/tickets.log
cargo run --offline -p labquest -- studio start servicedesk my-servicedesk
cargo run --offline -p labquest -- learn lesson my-servicedesk 1
cargo run --offline -p labquest -- learn check my-servicedesk 1
```

参考演示创建两张工单，第一张已关闭；JSON 应有 `count:2`。`tickets.json` 是演示当时的快照，后续修改不会自动更新该文件。

学员首次检查因 TODO 未实现而 FAIL 是正常的。先看第一条失败；若是编译器或链接器找不到，则应先修好环境。

打开 `my-servicedesk/START.html` 或 `TASK_CARDS.html`；只编辑 **`my-servicedesk/src/lib.rs`**，每次一个 TODO。命令行外壳 `src/main.rs` 已提供，先不改。目录已存在就继续使用；新建另一份练习时换名称。

## 2. 三个检查点

| 步骤与函数 | 要完成的行为 | 本次概念 |
|---|---|---|
| TODO(1) · `validate_title` | 拒绝控制字符，去首尾空白，返回合法标题 | 纯函数、`String`、输入校验 |
| TODO(2) · `apply` | 添加或关闭工单，错误不污染已有状态 | `&mut Vec<Ticket>`、枚举、ID 与状态 |
| TODO(3) · `render_json` | 输出 tickets 数组与 count，正确转义文字 | 只读切片、JSON 字符串、数据与协议 |

每步先写一组输入和预测，再修改、保存。第一步通过后，在 **平台根目录**继续：

```powershell
cargo run --offline -p labquest -- learn lesson my-servicedesk 2
cargo run --offline -p labquest -- learn check my-servicedesk 2
cargo run --offline -p labquest -- learn lesson my-servicedesk 3
cargo run --offline -p labquest -- learn check my-servicedesk 3
```

后一步会累计回归前面的步骤。提示用 `cargo run --offline -p labquest -- learn hint my-servicedesk 1`；进度用 `cargo run --offline -p labquest -- learn next my-servicedesk`。详细解释见 [servicedesk 讲义](../../course/studio/servicedesk.md)。

## 3. 用自己的输入完成一次作品

三步通过后，用全新的个人日志启动自己的服务。

从 **平台根目录进入学员目录**执行：

```powershell
cd my-servicedesk
cargo run --offline -- init my-tickets.log
cargo run --offline -- serve my-tickets.log 8766
```

浏览器打开 <http://127.0.0.1:8766/>，新增“复习借用”“整理作品”“写一条测试”三张工单，再关闭第一张。按 Ctrl+C 停服务，在同一学员目录执行 `cargo run --offline -- list my-tickets.log`：应有 `"count":3`，第一条 `"done":true`。再次启动同一日志，页面应保留这些状态；另试空白标题，观察错误而不是新增空工单。

## 4. 交付与解释

在 **学员目录**运行 `cargo test --offline` 回归全部测试。保存日志副本、三张个人工单和重启后的实际输出。把预测、错误和解释写入 `LEARNING_LOG.md`，再补一个自己的边界测试。

用自己的话回答：哪些代码决定业务状态，哪些处理 HTTP，哪些负责磁盘？为什么标题中的引号不能原样拼入 JSON？返回成功前为什么要确认持久化结果？

测试通过只说明已检查的行为符合合同；个人输入、真实产物和解释能帮助你判断学到了什么。

标题 1–120 个字符，最多 1000 张工单，日志最多 2 MiB。仅绑定 127.0.0.1，同步串行处理；没有认证、HTTPS、多人并发控制或公网部署能力。日志损坏会被拒绝，不自动修复。`init` 不覆盖已有日志。

`GET /api/tickets` 可供 NetFetch 下载；网页使用 `POST /api/tickets` 新增、`POST /api/close` 关闭。完整组合实验见 [系统路线](../../course/SYSTEMS_PATH.md)。

## 下一款：StoreDesk

接下来把单条状态修改扩展为数据库中的整批操作，学习参数绑定和事务回滚。在 **RustAppLab 根目录**运行（若仍在学员目录，先 `cd ..`）：

```powershell
cargo run --offline -p labquest -- studio start storedesk my-storedesk
cargo run --offline -p labquest -- learn lesson my-storedesk 1
```

继续阅读 [StoreDesk README](../storedesk/README.md)。
