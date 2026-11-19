# NetFetch · 从本机 HTTP 响应保存一份文件

**这次要做成什么：**解析一个本机 URL，检查真实 HTTP 响应，把内容保存到新文件。你会得到一次实际下载结果，并能区分地址错误、响应错误和写入失败。

**开始条件：**完成 [MiniKV](../minikv/README.md)，熟悉 `Result`、字节、借用与文件写入；本课不用先学异步框架。

本目录是已实现的参考软件；你的练习代码放在下面新建的 `my-netfetch/` 中。

## 1. 看目标，创建自己的工程

在 **RustAppLab 根目录**（能看到根 `Cargo.toml`）运行：

```powershell
cargo run --offline -p netfetch -- demo reference-download
cargo run --offline -p labquest -- studio start netfetch my-netfetch
cargo run --offline -p labquest -- learn lesson my-netfetch 1
cargo run --offline -p labquest -- learn check my-netfetch 1
```

会生成 `reference-download/files/intro.txt` 与 `notes.txt`，前者内容是 `你好，Rust 网络！`。演示使用真实回环 TCP，服务处理完即退出；不要把演示清单里的临时端口当作长期服务。

学员首次检查因 TODO 未实现而 FAIL 是正常的。先看第一条失败；若是编译器或链接器找不到，则应先修好环境。

打开 `my-netfetch/START.html` 或 `TASK_CARDS.html`；只编辑 **`my-netfetch/src/lib.rs`**，每次一个 TODO。命令行外壳 `src/main.rs` 已提供，先不改。目录已存在就继续使用；新建另一份练习时换名称。

## 2. 三个检查点

| 步骤与函数 | 要完成的行为 | 本次概念 |
|---|---|---|
| TODO(1) · `parse_url` | 只接受允许的回环 HTTP 地址，正确处理端口与路径 | 字符串解析、结构体、先验证再连接 |
| TODO(2) · `parse_response` | 检查状态行、头部与正文长度，返回正文原始字节 | `&[u8]`、协议边界、长度与错误 |
| TODO(3) · `download` | 获取响应后新建文件；拒绝覆盖，处理写入失败 | I/O、`Result`、`create_new`、副作用顺序 |

每步先写一组输入和预测，再修改、保存。第一步通过后，在 **平台根目录**继续：

```powershell
cargo run --offline -p labquest -- learn lesson my-netfetch 2
cargo run --offline -p labquest -- learn check my-netfetch 2
cargo run --offline -p labquest -- learn lesson my-netfetch 3
cargo run --offline -p labquest -- learn check my-netfetch 3
```

后一步会累计回归前面的步骤。提示用 `cargo run --offline -p labquest -- learn hint my-netfetch 1`；进度用 `cargo run --offline -p labquest -- learn next my-netfetch`。详细解释见 [netfetch 讲义](../../course/studio/netfetch.md)。

## 3. 用自己的输入完成一次作品

三步通过后，在 `my-netfetch/samples/my-note.txt` 写一行自己的学习目标。为了下载属于你的文件清单，在 **平台根目录的终端 A** 启动已有的 LocalHub 参考服务：

```powershell
cargo run --offline -p localhub -- serve --root my-netfetch/samples --port 8765
```

保留终端 A 运行；另开 **终端 B**，先定位到平台根目录，再执行下面的学员命令。

从 **平台根目录进入学员目录**执行：

```powershell
cd my-netfetch
cargo run --offline -- fetch "http://127.0.0.1:8765/api/files?limit=1" my-files.json
cargo run --offline -- fetch "http://127.0.0.1:8765/api/files?limit=1" my-files.json
```

第一次应显示 `saved=某个字节数`，打开 JSON 能看到你的 `my-note.txt` 文件信息；第二次应拒绝覆盖。换一个新输出文件名后再试未知路由，应因 HTTP 状态失败且不产生下载文件。完成后回到终端 A 按 Ctrl+C 关闭服务。

## 4. 交付与解释

在 **学员目录**运行 `cargo test --offline` 回归全部测试。保存下载的 JSON、所用 URL、正常与失败运行记录。把预测、错误和解释写入 `LEARNING_LOG.md`，再补一个自己的边界测试。

用自己的话回答：为什么响应长度按字节而不是中文字数计算？为什么收到响应不等于下载成功？为什么先检查目标路径，再决定是否创建文件？

测试通过只说明已检查的行为符合合同；个人输入、真实产物和解释能帮助你判断学到了什么。

这是同步客户端，只允许 `127.0.0.1/localhost/[::1]`，响应正文最多 2 MiB，头部最多 16 KiB。支持有限的 HTTP 1.0/1.1 成功响应；不支持 HTTPS、chunked、重定向、断点续传或公网下载。`batch` 接受“文件名<TAB>URL”清单，整体成功后提交新目录；不承诺断电持久化。

## 下一款：ServiceDesk

接下来从 HTTP 客户端转到服务端，把自己的业务状态提供给网页。在 **RustAppLab 根目录**运行（若仍在学员目录，先 `cd ..`）：

```powershell
cargo run --offline -p labquest -- studio start servicedesk my-servicedesk
cargo run --offline -p labquest -- learn lesson my-servicedesk 1
```

继续阅读 [ServiceDesk README](../servicedesk/README.md)。
