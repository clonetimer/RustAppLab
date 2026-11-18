# Book D1 · Cargo 可运行教学实验（独立 Workspace）

教材：[Rust Book 第 14 章](https://rust.bootcss.com/ch14-00-more-about-cargo.html)。对应本仓库讲义：[course/BOOK_D1_CARGO.md](../../course/BOOK_D1_CARGO.md)。

此目录与主工作区相互独立，作为参考实验随当前 0.19.0-rc.3 交付；根学员工程仍为 17 个。所有包都设置 `publish=false`，不会发布到 crates.io。零外部 Rust 依赖，Edition 2021 / Rust 1.74 可运行；历史 rc.1 标签与资产保持不变。

在仓库根目录执行：

```powershell
cargo metadata --locked --offline --no-deps --format-version 1 --manifest-path projects/book-d1-cargo/Cargo.toml
cargo test --locked --offline --workspace --all-targets --manifest-path projects/book-d1-cargo/Cargo.toml
cargo test --locked --offline --workspace --doc --manifest-path projects/book-d1-cargo/Cargo.toml
cargo run --locked --offline --manifest-path projects/book-d1-cargo/cargo-applab-map/Cargo.toml -- profiles
```

练习 `cargo install` 前先读取讲义：使用全新专用 `--root` 目录安装子命令，不覆盖用户日常使用的 Cargo 工具；保持离线，不运行 `cargo publish`。参考代码可读，可修改自己的复制副本，不直接修改发行候选版。
