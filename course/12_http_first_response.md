# 第 12 关 · 点亮控制台：让浏览器访问 Rust

**中心问题**：命令行已能分析数据，但使用者必须打开终端。如何让同一份分析结果在网页中被阅读？

## 先玩，再学

从项目根目录开始：

```sh
cargo run -p localhub -- demo my-localhub-12
cargo run -p localhub -- serve --root my-localhub-12/files --log my-localhub-12/logs/candidate.log
```

在浏览器打开 `http://127.0.0.1:8765/`；同时开一个终端运行：

```sh
curl -i http://127.0.0.1:8765/api/health
```

**观察任务**：在响应中找出 `HTTP/1.1 200 OK`、`Content-Type`、`Content-Length`，以及空行之后的 JSON 正文。尝试向 `http://127.0.0.1:8765/unknown` 发请求，比较 404 与 200 的区别。

## 只读这三处源码

1. `apps/localhub/src/main.rs`：用户参数从哪里进入？
2. `apps/localhub/src/lib.rs`：为什么必须传 `--root`，为什么不能绑定 `0.0.0.0`？
3. `apps/localhub/src/http.rs`：`TcpListener::bind` → `accept` → 读取请求头 → 路由 → 写回响应。

理解：TCP 只搬运字节，HTTP 再定义请求行、头、正文和状态码。Rust 标准库提供 TCP，不会自动给你一个 HTTP 框架。这一版故意只实现少量 HTTP/1.1 GET 路由，不把教学服务器声称为完整规范实现。

## 动手实验

用 `--port 0 --max-requests 2` 启动服务，记下打印的随机端口。使用 curl 分别访问 `/api/health` 和 `/api/files?limit=1`，两次请求结束后服务应自行退出。

**验收**：两次请求都是 200、`Content-Type` 为 JSON，文件接口至少包含 `stats` 和 `files` 字段；第三次请求因服务已退出而连接失败。运行：

```sh
cargo test -p localhub --test cli_smoke
cargo test -p localhub --test acceptance real_tcp_http_round_trip_and_graceful_limit -- --exact
```

**复盘问题**：如果客户端连接后迟迟不发请求，会发生什么？找到读取超时设置；如果多个客户端同时连接，串行服务器的处理顺序是什么？这些问题是下一阶段引入 async 的真实动机。

下一关：[`13_files_api.md`](13_files_api.md)。
