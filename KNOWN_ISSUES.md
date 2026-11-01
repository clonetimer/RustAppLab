# 0.18.0-dev.3 当前边界

新版 Windows 持续学习菜单已进行静态检查，尚无原生执行证据。当前源码也尚未在 Rust 1.74 原生重验；旧 RC 的通过证据只适用于旧冻结源码。基础微任务卡目前只覆盖 UnitLab、QuizDeck、TextScope，其余 Studio 使用既有讲义。没有新增真人教学效果数据。

与 dev.1 不同，本轮已取得单次全仓 Debug/Release 全部通过结果。下方历史条目中的超时或旧版本状态不能作为当前测试结论。

---

# 0.18.0-dev.1 已知限制与教学改进事项

| ID | 范围与现状 | 是否影响 Stable |
|---|---|---|
| PLATFORMS | 仅声明 Windows x64 MSVC、Linux x86_64。macOS 不声明支持，也不是 Stable 门槛 | **否** |
| WINDOWS-NEW | 当前版本新增 `LEARN_NEXT_WINDOWS.cmd` 和 `learn` 命令，尚未取得此开发版的 Windows 原生复验 | **是**，需要测试 |
| LEARNERS | 尚无足够的零基础学员连续独立完成 FirstCard/UnitLab/独立软件的观察数据 | **是** |
| ASSESSMENT | 旧 `.studio-reports` 可由用户修改，`learn next` 只用于导航，不产生新通过证明 | 已有约束，不能当能力证据 |
| ERROR-HELP | `learn diagnose` 仅解释十种常见 Rust 错误码，不能自动定位或修复任意错误 | 不阻塞当前教学范围 |
| STORAGE | SafeSync/MiniKV 为教学存储，不承诺物理断电安全；StoreDesk 需系统 SQLite | 已知边界 |
| NETWORK | 同步 HTTP 与 AsyncBoard Future 不构成 Tokio/Reqwest/Axum 非阻塞网络课程 | 后续扩展范围 |
| NEW-DOMAINS | ChatDock、DeskNotes GUI、WasmBoard 仍未完成 | 后续课程，不伪报已完成 |

当前 0.17.0-rc.2 冻结版本的 RC_READY 有独立记录。当前开发版不得以旧版证据发布 Stable。
