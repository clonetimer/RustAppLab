# 实际输出示例

`badgeforge-demo.svg` 由本轮编译的 BadgeForge Release 程序真实生成，可直接用浏览器打开。
它是 Rust 程序的输出，不是外部网站配图或概念设计稿。命令示例：

```powershell
cargo run --offline -p badgeforge -- make "我的 Rust 作品" "从第一行代码到自己的软件" my-badge.svg
```

输出路径必须不存在，拒绝覆盖是产品合同的一部分。
