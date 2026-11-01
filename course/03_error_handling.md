# 03 · 容错的扫描仪：Result 与错误处理

**本章中心**：错误不是编译失败后的附带问题，而是软件行为的一部分。用户需要知道哪些操作失败、是否仍拿到了部分结果。

## 先制造错误

```sh
cargo run -- scan this-directory-should-not-exist
cargo run -- scan Cargo.toml
cargo run -- report apps/filequest -o no-such-parent/report.html
cargo run -- demo
```

第三条因父目录不存在而失败。第四条如果你之前已创建过默认演示目录，会拒绝覆盖已有目录。

在 PowerShell 中，命令执行后可以用 `$LASTEXITCODE` 查看退出码；在 Bash/Zsh 中可用 `echo $?`。预期：成功为 0，参数/运行错误为非零。

## 最小源码路径

`apps/filequest/src/main.rs`：

```rust
if let Err(message) = filequest::run(&arguments) {
    eprintln!("错误：{message}");
    std::process::exit(2);
}
```

`apps/filequest/src/scanner.rs`：

```rust
let root = fs::canonicalize(root)?;
```

`?` 传播当前无法处理的错误；文件系统中部分子目录不可访问时，程序则把错误加入 `problems` 并继续扫描。这两种行为服务于不同的产品语义：**根目录无效必须失败，局部访问异常可以给出部分结果，但要明确提示用户结果可能不完整**。

## 修改实验

1. 把 `demo` 的目标名称改成一个已经存在的目录，观察错误中有没有明确目标路径。
2. 观察 `scan` 如何打印 `problems`，解释为什么扫描有不可访问的子目录时不直接丢弃所有成功结果。
3. 在 `main.rs` 临时把 `eprintln!` 改成 `println!`，并用 shell 把标准输出重定向到文件，比较两种行为；实验后改回去。

## 验收

```sh
cargo test
cargo run -- scan this-directory-should-not-exist
```

第二条必须失败且给出易于定位的错误，而不是 panic 或错误地报“扫描完成”。本关没有强制 RED→GREEN 测试，因为对不同操作系统强制制造权限拒绝不稳定；产品边界验证比凑一个不可靠测试更重要。

**支线**：为 `report` 加 `--strict`，让任何局部访问异常导致非零退出码；注意必须先把行为契约及跨平台测试设计清楚。
