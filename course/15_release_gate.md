# 第 15 关 · 交付一个负责任的本地 Web 软件

**中心问题**：页面能打开，不代表它是安全、可靠、可交付的 Rust 应用。怎样证明软件行为和边界？

## 验证三种 HTTP 异常

确保本地服务运行后，分别发送：

```sh
curl -i http://127.0.0.1:8765/not-found
curl -i -X POST http://127.0.0.1:8765/api/files
curl -i 'http://127.0.0.1:8765/api/files?root=/'
```

依次应为 404、405、400。观察 `Connection: close`、`Content-Length`、`Content-Security-Policy`。从 `apps/localhub/tests/acceptance.rs` 找到 Host 校验、真实 TCP 收发、请求头上限的测试。

## 新能力挑战：自定义健康检查的服务名

允许 `serve --name MyWorkshop`，使 `/api/health` 的 `app` 字段变成 `MyWorkshop`；未传时继续返回 `localhub`。需要同时修改：CLI 解析、启动配置、路由状态和单元/集成测试，不能只在网页上写一个名字。

```sh
cargo test -p localhub --test challenges mission_15_custom_service_name -- --ignored --exact
```

这个测试会真正启动服务并通过 TCP 请求 `/api/health`。最容易遗漏的是兼容旧 `serve` 命令和参数边界。

## 软件发布卡

运行所有基础验收：

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
cargo build --workspace --release
```

发布时还需检查 Windows、macOS 上的实际结果，不把 Linux 通过当作跨平台通过。教学挑战默认忽略，**未完成挑战不是基础功能失败，也不能宣称已经完成挑战**。

## 产品限制要能向使用者说明

LocalHub 只监听 `127.0.0.1`，不支持认证、TLS、多线程、生产级 HTTP 兼容性或用户授权管理；同机应用仍可能访问本服务。文件和日志默认被读取一次并生成快照。未来如要允许远程访问，必须先独立设计鉴权、访问权限与威胁模型，而不是直接把地址改为 `0.0.0.0`。

**交付物**：构建好的 `localhub` 可执行文件、正常/异常 HTTP 测试记录、一个浏览器截图（本机手工验证）、已知限制与版本说明。下一阶段是可靠备份或网络并发性能优化，不是堆叠无关页面。
