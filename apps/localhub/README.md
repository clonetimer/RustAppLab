# LocalHub 0.3.0 · Rust 本地工作台

这是 Rust AppLab 的第三款教学软件。它复用 FileQuest 的目录扫描与 LogLens 的日志分析库，通过标准库 `TcpListener` 提供本机只读网页与 JSON API。

## 5 分钟启动

在仓库根目录运行（首次使用新目录名，避免覆盖已有文件）：

```sh
cargo run -p localhub -- demo
cargo run -p localhub -- serve --root localhub-playground/files --log localhub-playground/logs/candidate.log
```

打开 <http://127.0.0.1:8765/>。可以看到文件清单、文件类型分布、日志各级别统计，以及演示日志的 **37.50% 记录级错误占比**；对比基线日志时，该数值为 12.50%。按 `Ctrl+C` 停止。

不用演示数据时，可以用真实目录及显式选定的日志文件：

```sh
cargo run -p localhub -- serve --root . --log ./my.log --port 8766
```

这里的 `--root .` 是用户显式授权；**请优先使用小目录，不推荐以家目录、系统根目录、私密资料总目录作为工作区**。

## HTTP API

| 路由 | 响应及限制 |
|---|---|
| `GET /` | 本机浏览器页面；所有 HTML / CSS / JS 离线可用 |
| `GET /api/health` | `status / app / version / snapshot` JSON |
| `GET /api/files` | 指定根目录的文件数量、体积、扩展名分布、前 100 条文件路径 |
| `GET /api/files?limit=200` | 更改返回条数，整数范围 1–500；未知参数返回 400 |
| `GET /api/logs` | 若配置日志则返回识别记录、级别、错误占比及高频消息；否则 `enabled:false` |
| `GET /assets/app.css`、`GET /assets/app.js` | 网页资源；无 CDN、无 JS 依赖 |

通过 curl 检查：

```sh
curl -i http://127.0.0.1:8765/api/health
curl http://127.0.0.1:8765/api/files?limit=3
curl http://127.0.0.1:8765/api/logs
```

快照在**服务启动时**生成：修改目录或日志后，需重新启动服务才会重新扫描。网页上的检索只筛选**已返回的前 200 个文件**，并非全部文件的实时搜索。用户必须手动指定 `--root`；浏览器不能通过 query 参数改写后端的扫描目录。

## 数据边界与安全

- 服务仅绑定 `127.0.0.1`，拒绝非本机 Host，不返回跨域许可头；只允许 GET，不支持文件上传、执行命令或修改本地文件。
- 解析 HTTP/1.1，8 KiB 请求头上限、单个连接 3 秒读取/写入超时；当前**单线程串行**，慢连接会影响响应，不能当做生产 HTTP 服务。
- 没有身份认证；同机进程仍可访问。不要端口转发、反向代理发布或配置成公共网络服务。浏览器可能加载包含敏感路径、文件名和日志消息的内容。
- 路径快照沿用 FileQuest 忽略构建目录、跳过符号链接的规则；LogLens 只识别两种 UTF-8 单行日志格式。未识别行不参与错误率分母。
- 启动时扫描全部可读的目标目录，并处理所选日志，内存/耗时尚未设置严格上界。只对可信的本机数据使用；后续版本会增加资源预算与并发测试。

## Rust 教学位置

- `src/cli.rs`：`enum`、`Result`、数字范围和路径参数契约。
- `src/data.rs`：复用 Workspace 成员 crate、结构体所有权、JSON 转义与借用。
- `src/http.rs`：`TcpListener`、`TcpStream`、HTTP 报文、路由、错误状态码与超时。
- `src/ui/`：少量 JavaScript 调用真实 Rust API，使用 `textContent` 而非 `innerHTML` 展示文件名。
- `tests/acceptance.rs`：真实套接字、异常请求、Host 验证、纯函数路由测试。
- `tests/challenges.rs`：需要学员亲手完成的 13–15 关，默认忽略。

从 [`../../course/12_http_first_response.md`](../../course/12_http_first_response.md) 开始。

## 质量验收

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
cargo build --workspace --release
```

`--max-requests N` 仅用于可终止的本地自动化复验。`--port 0` 可由操作系统分配空闲端口，输出中的实际 URL 才是访问地址。生产级性能、安全、兼容性不在此教学项目的承诺范围内。
