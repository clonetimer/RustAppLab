# 集合与错误传播练习

在同一份成绩输入上练习 Rust Book 8.1、8.2、8.3 和 9.2。每轮先写预期，再跑示例。在临时副本中只改一处，记录实测结果和原因。用你自己的姓名、标签和分数做最后一轮。

## 运行实验

从仓库根目录执行。源码里的每个示例都由 doctest 检查，行为测试同时覆盖两版读取函数。

```bash
cd projects/book-d6-foundations/foundation-lab
cargo test --offline --test collections
cargo test --offline --doc collections
```

打开 [collections.rs](../projects/book-d6-foundations/foundation-lab/src/collections.rs) 和 [tests/collections.rs](../projects/book-d6-foundations/foundation-lab/tests/collections.rs)。标有 `compile_fail` 的示例要编译失败，其旁的成功对照要编译并运行通过。索引触发的 panic 由示例中的局部捕获检查，测试输出可能仍显示 panic 位置。

## Vec 读取与修改

`vec![...]` 和 `Vec::new()` 都会创建向量，示例里的 `push(42_u32)` 让编译器推断元素类型是 `u32`。`get` 返回 `Option<&T>`，`select_label` 把取到的 `&String` 转成 `&str`。直接写 `labels[i]` 要求下标有效，越界会触发 panic。`for label in &labels` 产生共享引用，`for score in &mut scores` 产生可变引用，`*score += 1` 修改原元素。`annotate_labels` 用可变切片接收需要修改的标签。

1. 对两个标签预测下标 `0`、`1`、`2` 的结果，再分别运行 `select_label`。对下标 `2` 的直接索引，查看模块中局部捕获的对应示例。
2. 改成只传入 `&mut labels[..1]` 并追加自己的后缀。检查第二个标签的内容。再加一个 `pop` 断言，说明为什么向量空时得到 `None`。
3. 给 `Cell` 添加一个 `Missing` 变体，预测原 `match` 在补分支前的结果，再补分支并把它放进向量。解释每个元素为什么仍具有同一种类型。

持有元素引用时调用 `push` 的借用反例已在 [编译错误练习](RUST_ERROR_LABS.md) 的 E0502 小节中演示。

## String 追加、拼接与 UTF-8

`push_str(&str)` 追加一段借来的文本，`push(char)` 追加一个 Unicode scalar。`left + right.as_str()` 移动左侧的 `String`，借用右侧的文本。用 `format!("{left}{right}")` 拼接这两个值时，两个输入还可继续使用。

1. 用两个自己的 `String` 运行 `format!` 对照。换成 `+` 后继续打印左侧和右侧，预测哪一行得到 E0382，修复后再次运行。
2. 对 `"A中"` 写出字节长度、`bytes()` 的输出、`chars().nth(1)`、`get(1..4)` 和 `get(1..2)` 的结果。换入含表情或组合音标的标签重做。`len` 和切片范围以字节为单位，`chars` 遍历 Unicode scalar，一个显示字形可能由多个 scalar 组成。
3. 比较 `label[0]` 的 E0277、`&label[1..2]` 的边界 panic 和 `label.get(1..2)` 的 `None`。字符串的第几个字符需要按 UTF-8 解码查找，直接按字节偏移可能落在多字节编码内部。处理外部传入的字节范围时用 `get` 检查边界。

## HashMap 键的所有权与更新

给 `HashMap<String, u32>` 调用 `insert(name, score)` 会移动 `name`。如果 map 储存的是 `&str` 键，原字符串要活到 map 最后一次使用该键。拥有 `String` 键的 map 可以用 `get("Lin")` 按 `&str` 查询，返回类型是 `Option<&u32>`。缺键时得到 `None`。

1. 将同一姓名按 `80`、`91` 的顺序传给 `scores_by_name`，预测最终成绩。函数通过 `insert` 覆盖旧值，`insert` 返回被替换的值。
2. 在示例中把第二次写入换成 `entry(...).or_insert(91)`，观察保留了哪个成绩。对返回的 `&mut u32` 解引用并加一，预测新结果，比较缺键时的行为。
3. 用一个局部 `String` 分别构造拥有键的 map 和借用键的 map。尝试在最后一次查询前释放原字符串，说明每种写法中键的有效期由谁承担。若要输出一份稳定成绩单，先决定排序规则，再排序输出。

[TextScope](studio/textscope.md) 已有字节与字符计量、Entry 计数和稳定排序练习，可以用自己的成绩单检验同样的排序思路。

## 第一行分数与错误传播

`read_score_explicit` 用 `match` 展开出错时的返回，`read_score` 用 `?` 表达相同流程。先从 `BufRead` 读取一行，然后用 `trim` 去掉首尾空白，最后解析为 `u32`。两版都只读第一行。文件结束时读到空字符串，得到 `ScoreError::Parse`。如果读取失败，两版都尽早返回 `ScoreError::Io`，其中保留原始 `io::Error`。

1. 给两版分别输入 `b" 42\r\n9"`、`b"oops\n9"`、`b""` 和 `b"42\xff\n9"`。先写出各自的预期结果，再运行。最后一组输入的第一行含无效 UTF-8 字节，`read_line` 会返回 `InvalidData`。记录是哪一步产生错误，后续步骤是否还能执行。
2. 在 `read_score_explicit` 找出两个 `ScoreError::from` 调用，再标出 `read_score` 中相应的两个 `?`。说明 `From<io::Error>` 和 `From<ParseIntError>` 各负责哪一条路径。
3. 在 `fn main()` 里用 `?` 取分数，记录 E0277。把入口改为 `fn main() -> Result<(), ScoreError>` 并用 `Ok(())` 结束；换入失败输入，确认错误仍然会交给调用者。最后自选一个 `u32` 边界值，为两种写法补相同的断言。

[MiniKV 的日志练习](studio/minikv.md) 已经用 `append(...)?` 演示失败时立即返回，内存因此保持原值。这里可以把同一个读取函数的两种写法逐步对照，并观察两种底层错误如何进入同一返回类型。
