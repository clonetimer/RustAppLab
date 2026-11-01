# 07 · 打包交付：从 cargo run 到真正能用的程序

**本章中心**：发布不是给别人发一堆源码，而是让用户在其操作系统上得到明确、可验证的可执行文件与使用说明。

## 先做一次 release 构建

```sh
cargo build --release
cargo test
```

构建产物默认在：

- Linux/macOS：`target/release/filequest`
- Windows：`target/release/filequest.exe`

无需再通过 `cargo run`：例如在 Linux/macOS 直接运行：

```sh
./target/release/filequest help
./target/release/filequest scan .
```

Windows PowerShell：

```powershell
.\target\release\filequest.exe help
.\target\release\filequest.exe scan .
```

## 你要交付什么

- 对应目标操作系统和架构的二进制文件；
- 至少覆盖 `help`、`scan`、`find`、`report` 的使用说明；
- 一份在实际项目上生成的 HTML 报告（注意报告包含本地目录路径，分享前检查隐私）；
- 运行 `cargo test` / `cargo fmt` / `cargo clippy` 的真实结果；
- 清楚列出的限制，例如不支持全文搜索或符号链接遍历。

## 必做验收场景

| 场景 | 预期 |
|---|---|
| 有中文文件名的目录 | 正常显示文件名，成功生成报告 |
| 不存在的目录 | 非零退出码，说明路径错误 |
| 对已有报告执行 `report` | 默认拒绝覆盖 |
| `report --force` 覆盖普通文件 | 成功替换报告 |
| 目录包含符号链接 | 不跟随、不无限递归 |
| 包含 `.git` / `target` | 默认跳过；`--all` 可选扫描 |
| 文件名中有 `<`、`&` 等特殊字符 | HTML 正确转义 |

## 跨平台提醒

Rust 编译出的二进制与目标平台相关，不是一次构建同时兼容 Windows/macOS/Linux。需要为对应平台构建并测试。`scripts/verify.sh` 和 `scripts/verify.ps1` 提供相同的基础检查入口。CI 配置只代表计划的检查动作；需要在实际 GitHub Actions 中执行并有成功记录，才能声称对应平台测试通过。

## 交付思考

- `cargo build` 与 `cargo build --release` 使用不同的优化配置；为什么日常开发不用总是 release？
- 在发布报告前要不要提供文件列表脱敏模式？当前没有，怎样向用户解释？
- 下一款产品 LogLens 应该与 FileQuest 共用哪些抽象？最好先有两份真实业务需求，再决定是否抽象。

完成这章就可以把 FileQuest 作为自己写的第一款软件使用，而不是把它留在“代码练习”目录中。
