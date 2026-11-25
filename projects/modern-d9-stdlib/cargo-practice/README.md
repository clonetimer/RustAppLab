# 我的 Cargo 现代诊断实验

使用个人副本，保留原模板。无需 registry 依赖。先运行 cargo generate-lockfile --offline，观察 Cargo.lock version = 4。运行 cargo check --locked --offline --all-targets，记录 Cargo unused dependency 与 Rust unused variable 两类诊断。不要把不同层级的 lint 混为一谈。

1. 在 Cargo.toml 增加 [lints.cargo] / unused_dependencies = "deny"，再运行相同 check，预测非零退出。不要用 allow 隐藏问题。
2. 修改 main，实际调用 d9_helper::label() 并输出 helper=ready；再次 check，应消除未使用依赖错误，但 Rust 未使用变量警告仍存在。
3. 运行 cargo check --locked --offline --config 'build.warnings="deny"'，观察变量警告成为失败；删除无用变量，再运行，应成功。
4. 运行 cargo run --locked --offline --profile debug，观察 mode=included（来自 .cargo/shared.toml，由 config.toml include）及 helper=ready。
5. 修改个人副本 shared.toml 中 D9_MODE 的值，重新运行，验证配置被实际使用。保留日志；不要修改全局配置。

冷却机制只在完整源码维护者脚本的临时本地注册表中重放；没有连接公网或实际上传 crate。参见 course/BOOK_D9_STDLIB_CARGO.md。
