# AsyncBoard · 分层提示（不要一开始全看）

## 第 1 层：方向

1. `input.lines()` 遍历每行，用 `split('\t')` 分成字段。
2. `render_report` 从固定表头开始，使用 `push_str` 或 `format!` 追加。
3. `complete` 的关键是 `Delay` 实现了 `Future<Output = ()>`，因此可以在 `async fn` 里 `.await`。

## 第 2 层：思路

1. 用 `HashSet<String>` 检查重复；`parse::<u64>()` 解析数字并用 `map_err` 提供中文错误；始终先检查再 `push`。
2. `for item in completions` 的 item 是借用值，不必移动或 clone 整个集合。
3. `delay.await;` 必须发生在创建 Completion 之前。先 await 后构造结果；不要用 `thread::sleep`。

## 第 3 层：局部示例（不是完整答案）

```rust
let delay_ms: u64 = delay_text.parse().map_err(|_| "等待时间不是整数")?;
if delay_ms > MAX_DELAY_MS { return Err("等待时间过长".into()); }
```

```rust
text.push_str(&format!("{}\t{}\t{}\n", item.id, item.delay_ms, item.note));
```

```rust
delay.await; // 只有在 async fn 内才能使用
```

如果依然失败，把**第一条**编译或测试错误与当前修改的函数一起分析，不要一次重写整个文件。
