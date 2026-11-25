# 我的 Rust 工具诊断练习

复制本目录到个人新目录，安装固定工具链的 rustfmt 与 clippy，不修改平台原件。

1. cargo fmt -- --check：应拒绝当前格式。阅读 diff，运行 cargo fmt，再执行检查。
2. cargo clippy --locked --offline --all-targets -- -D warnings：应拒绝 needless_return；不要通过 allow 或删除检查解决。
3. 修改 classify，保留返回类型、去掉不必要的 return，运行 cargo fmt 和同一 Clippy 命令。
4. cargo test --locked --offline：边界行为仍正确。把阈值附近输入改为自己的迁移测试。

格式化通过不是代码正确，Clippy 通过也不是证明无 UB 或无漏洞。记录工具版本、实际诊断、修复与行为回归。
