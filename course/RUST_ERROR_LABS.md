# Rust 编译错误练习：错误本身也是教学材料

这份材料**不要求学员记住错误编号**。每个实验都有：一个真实的软件需求、一段会失败的程序、修复方向，以及要自己解释的问题。建议先用 `UnitLab`、`QuizDeck`、`TextScope` 的实际业务代码练习，再来读抽象机制。

以下是可以单独复制到自己的**临时** `src/main.rs` 里试的最小程序。不要覆盖已经完成的作品；先用 `cargo new rust-error-practice` 新建实验目录。

## E0382：字符串被移动以后为什么不能继续用？

业务：给任务管理器保存标题，同时打印成功提示。

```rust,compile_fail
fn main() {
    let title = String::from("写完 Rust 单元测试");
    let mut tasks = Vec::new();
    tasks.push(title);
    println!("新增任务：{title}"); // E0382：title 已移动到 tasks
}
```

修复方向：如果无需所有权，可以先打印，再执行 `push`；如果两处都需要拥有数据，仔细权衡是否要 `clone`，不要把 `clone` 当通用解法。

```rust
fn main() {
    let title = String::from("写完 Rust 单元测试");
    println!("新增任务：{title}");
    let mut tasks = Vec::new();
    tasks.push(title);
    assert_eq!(tasks.len(), 1);
}
```

**解释题**：为什么 `tasks.push(title)` 之后 `tasks` 能使用字符串，而局部变量 `title` 不能？

## E0502：正在查看任务时，为什么不能同时修改它？

```rust,compile_fail
fn main() {
    let mut tasks = vec![String::from("写需求"), String::from("写测试")];
    let first = &tasks[0];
    tasks.push(String::from("写文档"));
    println!("{first}"); // E0502：Vec 可能重新分配内存
}
```

在修改之前结束借用：

```rust
fn main() {
    let mut tasks = vec![String::from("写需求"), String::from("写测试")];
    println!("{}", &tasks[0]);
    tasks.push(String::from("写文档"));
    assert_eq!(tasks.len(), 3);
}
```

**解释题**：为什么 `push` 可能让之前的引用不再安全？

## E0308：为什么打印数值不等于返回数值？

```rust,compile_fail
fn double(x: i32) -> i32 {
    println!("{}", x * 2);
}
fn main() { let _ = double(5); }
```

```rust
fn double(x: i32) -> i32 { x * 2 }
fn main() { assert_eq!(double(5), 10); }
```

**解释题**：`println!` 的结果是 `()`，为什么不能用于需要 `i32` 的计算？

## E0597：借用的文字能否比拥有它的字符串活得更久？

```rust,compile_fail
fn main() {
    let borrowed: &str;
    {
        let message = String::from("已完成备份");
        borrowed = &message;
    }
    println!("{borrowed}");
}
```

```rust
fn main() {
    let owned: String;
    {
        let message = String::from("已完成备份");
        owned = message;
    }
    assert!(owned.contains("备份"));
}
```

**解释题**：上面为什么选择传递 `String` 所有权，而不是返回一个生命周期不足的 `&str`？

## 真实作品迁移任务

完成后返回 `TaskPocket`，试着用自己的话解释：`load` 读取的任务为什么要拥有 `String` 标题？`list` 如果只是展示，为什么不需要夺走任务的所有权？再返回 `TextScope`，解释为什么搜索结果可以只借用原文。

教师可用错误诊断器为学员提供第一层解释：

```powershell
cargo run --offline -p labquest -- learn diagnose errors.txt
```

诊断器不会自动修复代码。请要求学员写下“错误原因 → 我修改了什么 → 换什么输入重新验证”，不要让它成为复制参考答案的入口。
