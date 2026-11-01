# 第 30 关 · PageMint：第二次独立开发（更换领域）

**中心任务**：你已经做过 TaskPocket 的 CRUD 和文件持久化，这次换一种需求——把自己的离线笔记变成一个能用浏览器打开的 HTML 网站。不是修改 TaskPocket 的业务规则，也不是继续背 Rust 语法，而是**将已学的语言能力迁移到不同产品**。

## 领取一份真正的软件需求

在 RustAppLab 根目录运行：

```sh
cargo run -p labquest -- app catalog
cargo run -p labquest -- app init pagemint my-pagemint
```

这会创建一个**独立 Cargo 工程**。它已经能编译并显示 `help`，但 `stats` 和 `build` 都还没有实现。你需要先阅读：

- `my-pagemint/CONTRACT.md`：交付需求和边界条件；
- `my-pagemint/samples/notes/`：两份真正的样例笔记；
- `my-pagemint/PROJECT_JOURNAL.md`：记录失败、修改、实际用途；
- `my-pagemint/src/main.rs`：准备由你完成的软件源码。

进入独立工程：

```sh
cd my-pagemint
cargo run -- help
cargo run -- stats samples/notes
```

第二条命令在初始工程中**预期失败**，因为功能留给你自己实现。你最终应能得到 `notes=2 paragraphs=4`，并运行 `build samples/notes samples/my-notes.html`，用浏览器打开网页。

## 用四个可观察里程碑推进

| 里程碑 | 你亲手完成什么 | 可观察结果 | 重点知识 |
|---|---|---|---|
| M1 | 识别目录中的 `.note` 普通文件 | `stats` 能统计文件数与非空正文行数 | `Path`、`read_dir`、模式匹配 |
| M2 | 解析并排序文件 | 中文标题、段落均保留，顺序稳定 | `String`、`Vec`、`sort_by`、迭代器 |
| M3 | 输出完整离线网页 | `build` 产生可浏览的 HTML；特殊字符显示为文本而非标签 | `format!`、字符遍历、HTML 转义 |
| M4 | 处理异常并禁止覆盖 | 损坏文件、符号链接及已有目标不会破坏数据 | `Result`、`?`、`create_new` |

不要一开始设计复杂框架。先用一个结构体表示 Note，然后复用一个 `read_notes` 函数服务两条命令。可以先让 M1 通过，再逐个完成其余里程碑。

## 不看答案的预测实验

已知 `samples/notes` 的第一份笔记中有一行空行，且标题不计为正文段落。请先手算两份文件的 `notes` 和 `paragraphs`，并在记录中写下你的预测，然后再运行程序。

第二个实验：把正文写成 `A & B <script>alert(1)</script>`。在页面里它应作为普通文字显示，而不是被执行。请先写下你认为必须转换的字符，随后打开生成的 HTML 查看。

## 学员自主控制的进度

你可以随时返回 RustAppLab 根目录执行：

```sh
cargo run -p labquest -- app check pagemint my-pagemint
cargo run -p labquest -- app hint pagemint 1
```

`app check` 实际编译你写的程序，创建临时笔记目录、运行程序并检查 HTML，失败时输出第一条可以修复的反馈。按需请求第二层、第三层提示，不会直接覆盖你写的源码。

**成绩限制**：自动评测只证明需求合同的一部分行为通过，不能认定你已经独立掌握 Rust。还需要自己用真实笔记尝试，并向其他人解释设计。

下一关：[第 31 关 · 用真实笔记交付作品与跨领域复盘](31_transfer_release.md)。


## 新版连续指导

[第五段：独立开发](INDEPENDENT_PATH.md) 提供只读→业务→完整合同三个累计行为检查点、设计记录、个人回归测试和发布清单。旧 app check 仍执行完整合同，不必重建已有项目。
