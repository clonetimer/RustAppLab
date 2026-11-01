# 04 · 数据分析小队：struct、Vec、BTreeMap 与迭代器

**本章中心**：一份文件清单只有路径和大小，经过结构化聚合才能成为用户需要的报告。

## 观察

```sh
cargo run -- scan filequest-playground
```

输出包含文件总数、总体积、扩展名分组、体积最大文件。它们都来自 `scanner.rs` 中的同一个 `ScanSummary`，不是四次独立的文件遍历。

关键类型：

```rust
pub struct FileRecord {
    pub relative_path: PathBuf,
    pub bytes: u64,
    pub extension: String,
}
```

`Vec<FileRecord>` 持有所有普通文件的记录。`BTreeMap<String, usize>` 让扩展名统计具有确定性，然后可以按数量排序。`largest_files` 先按体积降序，再以路径作为稳定的第二排序键；这对自动测试和结果对比有价值。

## 本章产品扩展

目前只会输出每类文件有多少个。请在 `ScanSummary` 上增加一个按扩展名累加字节的函数，例如：

```rust
pub fn extension_bytes(&self) -> BTreeMap<String, u64> {
    // 这是需要你补充的实现设计，不要直接照抄现有 count 方法。
    todo!()
}
```

**提醒**：以上是思路片段，请不要将 `todo!()` 直接放入正在执行的路径而不实现。对累加运算考虑 `saturating_add`，避免极端输入时溢出。

在 `scan` 输出中增加如 `log：12 B` 的各类型体积信息。

## 测试

```sh
cargo test --test challenges mission_04_extension_size_summary -- --ignored --exact
cargo test
```

测试给出两个 `.log` 文件，大小分别为 5 B 和 7 B，要求显示总计 12 B。这是可重复的真实文件 I/O，而不是“手动猜出输出”。

## 思考

- 对文件列表先做 `clone()` 再排序是否必要？如何让 `largest_files` 使用引用避免复制每个文件记录？
- `HashMap` 和 `BTreeMap` 在排序、性能和确定性上有什么区别？
- 本版本会把所有文件元数据放进内存；面对百万文件应怎样设计流式统计？暂时只列设计，不提前复杂化课程。

本章验收物：**能从文件清单得到可验证的体积分类统计**。
