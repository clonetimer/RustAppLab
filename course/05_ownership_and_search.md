# 05 · 失踪文件追踪：借用、所有权与路径检索

**本章中心**：用真正的文件检索理解 `&self`、`&str`、`Vec<&T>`，知道何时需要拥有数据，何时只应借用。

## 观察

```sh
cargo run -- find filequest-playground logs
cargo run -- find filequest-playground rs
```

第一个命令能找出日志目录内的文件，是因为当前搜索对象是**相对路径**。打开 `scanner.rs` 中的：

```rust
pub fn find(&self, query: &str) -> Vec<&FileRecord>
```

三个值得理解的点：

- `&self`：检索只读取 `ScanSummary`，不拿走或修改它。
- `query: &str`：接受文本片段借用，避免不必要的 String 所有权转移。
- `Vec<&FileRecord>`：返回扫描结果中记录的引用集合，不复制路径、扩展名和其他字段。返回引用的有效期与被借用的 `ScanSummary` 关联，由编译器验证。

## 本章修改任务：只按文件名查找

现有 `find` 通过相对路径匹配，因此查询 `logs` 会匹配 `logs/a.txt`。新加 `--name-only` 标志，让用户可以明确表示**只按最后一段文件名匹配**：

```sh
cargo run -- find filequest-playground logs --name-only
```

预期：如果没有文件名本身包含 `logs`，匹配数为 0（即使这些文件都放在 `logs/` 文件夹中）。正常的 `find` 行为保持不变。

建议先在 `ScanSummary` 中实现一个单独的方法，通过 `relative_path.file_name()` 获取末尾文件名，避免把已有功能悄悄改掉；再将新开关接入 `cli.rs` 和 `lib.rs`。对路径文本仍保留大小写不敏感的使用体验。

## 自动验收

```sh
cargo test --test challenges mission_05_find_basename_only -- --ignored --exact
cargo test
```

测试在 `logs/a.txt` 上搜索 `logs`，要求结果为 0。错误场景包括 `--name-only` 被误判为未知选项、使用完整路径替代文件名、把默认查找行为一起破坏。

**支线**：思考如果将 `ScanSummary` 移出作用域后仍持有 `Vec<&FileRecord>`，为什么编译器会拒绝？可以写一个小代码片段验证，不需要改动生产代码。
