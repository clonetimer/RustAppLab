//! 零基础第一步：生成独立 Rust 小工程，运行多输入验收。
//! 不修改已有文件，也不自动执行不可信代码；学员显式运行 check 时才编译其源码。
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::Duration;

const CARGO_TOML: &str = r#"[package]
name = "first-rust-card"
version = "0.1.0"
edition = "2021"

# 让嵌套在 RustAppLab 根目录内的练习项目保持独立。
[workspace]
"#;

const MAIN_RS: &str = r#"fn main() {
    // args() 是启动软件时传入的文字；nth(1) 取名字，nth(2) 取任务。
    let name = std::env::args().nth(1).unwrap_or_else(|| "朋友".to_string());
    let task = std::env::args().nth(2).unwrap_or_else(|| "学 Rust".to_string());

    // TODO：只修改下面这一行，让输出变成：今日任务卡 | 姓名 | 任务
    // 提示：println! 可以把变量放在 {name} 和 {task} 的位置。
    println!("等待完成第一张任务卡");
}
"#;

const PROJECT_README: &str = r#"# FirstCard · 从一行输出认识 Rust

这次练习要让你输入的名字和任务出现在任务卡中。你会修改一行代码，保存、运行、比较结果，再把自己的任务卡存成文本文件。按 [跟着做.md](跟着做.md) 从第 1 步开始。

## 先认清学习文件

| 文件 | 这次用它做什么 |
|---|---|
| [src/main.rs](src/main.rs) | 你的 Rust 源码；只修改 `println!` 那一行 |
| [Cargo.toml](Cargo.toml) | Cargo 用来识别这个工程的说明；本次不用修改 |
| [跟着做.md](跟着做.md) | 每步操作、预期输出、报错处理和完成后的自测 |
| [LEARNING_LOG.md](LEARNING_LOG.md) | 动手前的预测、自己的输入、用过的提示和换题后的解释 |

本文件所在的文件夹是**学员工程目录**。完整 RustAppLab 平台根目录还有 `apps/`、`course/` 和自己的 `Cargo.toml`，两者不是同一目录。

在本学员目录的终端运行：

```text
cargo run --offline -- "小林" "写 Rust"
```

`cargo run` 会编译并运行；`--` 后面两段文字是给任务卡的输入。初始输出“等待完成第一张任务卡”是正常起点。读讲义、修改、Ctrl+S 保存后再运行。

## 做到什么，才可以进入下一站

1. 当前源码通过 `beginner check` 的四组输入检查；只有学员包时先按讲义逐组手动比较，再用完整平台复验。
2. 自己再选两组名字和任务，先写预期再运行；找到并打开保存的 TXT。
3. 能指出哪一行改变输出，并用自己的话解释：`name` 和直接写死的“小林”有什么区别？为什么换输入后不用再改源码？

这一步练习的是“输入 → 变量 → 格式化输出”及“编辑 → 保存 → 编译运行 → 比较”的循环。读取命令行参数和默认值的代码已经提供；一次检查通过只说明当次程序满足四组固定案例，不能证明你已独立写出整个程序、掌握参数解析或学会全部 Rust。用过提示也可以继续，把仍不理解的地方记下来。
"#;

pub(crate) const UNITLAB_HANDOFF: &str = r#"
## 下一站：UnitLab，从显示文字到返回计算结果

完成自己的任务卡和上面的解释后，回到**完整 RustAppLab 平台根目录**（有 `apps/labquest` 的目录）。下面逐条执行，不要一次粘贴后就等待自动完成：

```text
cargo run --locked --offline -p labquest -- studio start unitlab "my-unitlab"
cargo run --locked --offline -p labquest -- learn lesson "my-unitlab" 1
cargo run --locked --offline -p labquest -- learn check "my-unitlab" 1
```

`start` 只创建全新目录。若 `my-unitlab` 已存在，跳过第一条，直接看任务卡；也可用 `learn next` 查看已有工程。自定义目录时，把每条命令中的 `my-unitlab` 换成同一个实际路径。

第 1 次检查通常失败：你还没有实现换算公式。新工程先读 `my-unitlab/README.md`；旧工程没有这份文件时，直接使用上面的 `learn lesson` 或原有 `START.html`。打开自己的 `my-unitlab/src/lib.rs`，只改 `celsius_to_fahrenheit` 中的 TODO(1)，保存后重复最后一条。第一步练习把数值交回调用者；后两步才连接单位选择、错误处理和完整命令行。第 1 步通过后看第 2 步任务卡，不要继续重复第 1 步，也不要先运行尚未完成的完整换算命令。

Windows 也可返回平台文件夹双击 `CONTINUE_WINDOWS.cmd`，按 Enter 选 UnitLab；已有工程用 `LEARN_NEXT_WINDOWS.cmd`。如果使用 learner-kit，直接打开包内已有的 `Studio/unitlab/README.md`，在 `Studio/unitlab` 目录执行 `cargo test --offline --test milestones step1_`，不必再次创建工程。
"#;

const PROJECT_GUIDE: &str = r#"# 我的第一个 Rust 软件：今日任务卡

目标：输入名字和任务，让输出随输入改变。我们先练习一行代码的效果，读取参数的代码已经替你写好。本次只修改 **src/main.rs 中的 println! 行**。

## 第 1 步：先运行未完成的程序

在本学员工程目录（本文件与 Cargo.toml 所在的文件夹）打开 PowerShell 或终端。Windows 可在资源管理器地址栏输入 `powershell` 后回车。执行：

```text
cargo run --offline -- "小林" "学习 Rust"
```

第一次应该显示 **等待完成第一张任务卡**。这说明程序已运行，但还没有输出输入内容。`Compiling` 是编译过程信息；初始代码可能出现变量尚未使用的 warning，它与阻止编译的 error 不同。

`cargo run` 负责把 `.rs` 源码编译后运行；`--` 把 Cargo 的选项与程序的输入分开。这里的两段带引号文字分别是名字和任务。引号能把“学习 Rust”中的空格保留在同一个参数内。

## 第 2 步：预测，再修改一行

用记事本或 VS Code 打开 `src/main.rs`。先认识你眼前的内容：

- `fn main()` 是程序开始执行的位置。
- 两行 `let` 给输入的数据起名，分别叫 `name` 和 `task`。没有提供参数时，已给出的代码会使用“朋友”和“学 Rust”。本次先使用这段代码，不要求独立写出它。
- `println!` 把文字显示到终端；格式文本里的 `{name}` 会插入变量 `name` 的值，`{task}` 同理。

先写下：如果把名字改成“阿青”，任务改成“整理笔记”，输出中应该改变哪两处？然后找到 `println!("等待完成第一张任务卡");`，让它使用这两个变量。输入“小林”“学习 Rust”时，目标格式是：

```text
今日任务卡 | 小林 | 学习 Rust
```

只改这一行，保留括号、双引号、分号，按 **Ctrl+S** 保存。不要把“小林”和“学习 Rust”写死在代码里，否则下一组输入就不正确了。完整答案需要时再看第 3 层提示。

## 第 3 步：用变化的输入验证

仍在本学员目录，先重复第 1 步命令，再运行：

```text
cargo run --offline -- "阿青" "整理笔记"
cargo run --offline
```

预期分别是 `今日任务卡 | 阿青 | 整理笔记` 和 `今日任务卡 | 朋友 | 学 Rust`。第二条没有传入名字和任务，因此使用已提供代码的默认值。再自选两组不同输入，先预测，再比较实际输出。这里不需要每换一次名字就修改源码。

## 第 4 步：自动检查当前源码

回到**完整 RustAppLab 平台根目录**（有 `apps/` 和 `course/`）。假设学员工程是该目录下的 `my-first-card`，运行：

```text
cargo run --locked --offline -p labquest -- beginner check "my-first-card"
```

如果工程在其它位置，把 `my-first-card` 换成真实路径并保留引号。不要在本学员目录运行带 `-p labquest` 的命令；学员工程并不包含平台程序。

检查先编译当前源码，再用四组固定输入真实运行：

| 输入的名字 / 任务 | 预期输出 |
|---|---|
| 小林 / 写 Rust | `今日任务卡 \| 小林 \| 写 Rust` |
| Alex / 整理房间 | `今日任务卡 \| Alex \| 整理房间` |
| 林 小 / A&B <test> | `今日任务卡 \| 林 小 \| A&B <test>` |
| 不传参数 | `今日任务卡 \| 朋友 \| 学 Rust` |

看到四项都通过，才说明这些案例满足要求。编译成功只说明代码能够编译。输出不同就先看“期待 / 实际”，修改并保存后重复检查。通过后生成 `FIRST_WIN.md`；旧记录不会覆盖，修改过源码后仍应重新检查。

**只有 learner-kit？** 本包中的 FirstCard 可直接 `cargo run --offline`，但没有 LabQuest 检查器。可以按上表逐组手动运行比较；这属于手动练习，不能记成自动 4/4。要自动检查，用完整平台对本工程的实际路径执行上面的命令。

## 第 5 步：保存一份自己的作品

回到本学员目录。PowerShell 使用：

```powershell
cargo run --offline -- "我的名字" "完成 Rust 入门" | Out-File -Encoding utf8 "我的任务卡.txt"
```

Linux 终端使用：

```sh
cargo run --offline -- "我的名字" "完成 Rust 入门" > "我的任务卡.txt"
```

先把名字和任务换成自己的内容，然后打开 `我的任务卡.txt` 核对。若已有同名文件，先换一个输出文件名再执行。保存源码和保存输出是两件事：Ctrl+S 保存 `.rs`，这条命令保存程序生成的文本。

## 第 6 步：用自己的话收尾

在笔记中记录两组自己的“输入、预期、实际”，并回答：

1. 哪一行负责显示任务卡？`{name}` 与直接写“小林”有什么区别？
2. 只换命令里的任务，为什么可以得到新任务卡？如果修改源码却忘了保存，会发生什么？
3. 编译成功、四组检查通过、我能解释并换新输入，各说明了什么？还在哪一步需要提示？

能做这些，就有了进入 UnitLab 的起点。你还没有独立实现命令行解析，也还没有练习函数返回值、分支或错误处理；下一款作品会逐步加入这些内容。

## 如果卡住

- `cargo 不是命令`：安装 Rust 后重新打开终端，安装说明见完整平台的 `START_HERE.html`。
- `找不到 Cargo.toml`：核对当前位置。本工程内运行自己的程序，平台根目录运行 `-p labquest` 的命令。
- `error: expected ...`：先比较引号、括号、分号，确认只改了指定行。
- 输出还是“等待完成”：确认保存的是本工程 `src/main.rs`，然后重新运行。
- 编译成功但检查失败：比较“期待 / 实际”，确认输出使用变量而非固定名字。

提示在完整平台根目录运行，先看 1，尝试后再看 2、3：

```text
cargo run --locked --offline -p labquest -- beginner hint 1
```
"#;

const FIRST_WIN_RECORD: &str = r#"# FirstCard · 一次实际运行的完成记录

生成本文件的那次检查已编译学员源码，并验证四组固定案例：中文输入、英文名字、带空格和符号的输入、未传参数的默认值。它记录的是程序行为；检查器没有验证代码由谁编写，也不能自动证明你已理解 Rust。

本文件只在第一次通过时创建。已有记录会保留，源码后来改变时请重新运行 `beginner check`，以当次结果为准。

## 把这次检查变成自己的学习证据

在本学员目录，用自己的名字和任务运行两次，先记录预期再核对实际。PowerShell 保存输出的命令是：

```powershell
cargo run --offline -- "我的名字" "今天要做的事" | Out-File -Encoding utf8 "我的任务卡.txt"
```

Linux 把管道及后面的 PowerShell 命令换为 `> "我的任务卡.txt"`。如文件已存在，换一个新文件名后再执行。找到并打开文本，确认它是你的这次输入生成的。

| 我的输入 | 我的预期 | 实际输出 | 用过的帮助 / 仍不理解的地方 |
|---|---|---|---|
| 第一组：待填写 | 待填写 | 待填写 | 待填写 |
| 第二组：待填写 | 待填写 | 待填写 | 待填写 |

最后用自己的话解释：哪一行负责显示？`name` 和直接写固定名字有什么区别？修改源码后为什么需要保存并重新运行？

本次练习了变量参与格式化输出，以及编辑、保存、运行、比较结果的循环。参数读取和默认值逻辑已经提供；通过这个热身不等于能从空文件独立开发软件。下一步开始练习函数的输入与返回值。
"#;

pub(crate) const HINTS: [&str; 3] = [
    "先找到 println! 那一行。name 与 task 已经从命令行读取好了，不需要写 for 循环。只考虑怎样在输出中显示两个变量。",
    "Rust 的格式化字符串可以在文本中直接写 {name} 和 {task}。保留 println! 的括号、双引号、分号，并用 ` | ` 分隔三段内容。",
    "最后一层提示（完整的一行）：println!(\"今日任务卡 | {name} | {task}\"); 先自己敲入，而不是直接替换整个文件。再换两个不同名字运行，确认输出会变化。",
];

fn write_new(path: &Path, text: &str) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|err| format!("无法新建 {}：{err}", path.display()))?;
    file.write_all(text.as_bytes())
        .map_err(|err| format!("无法写入 {}：{err}", path.display()))
}

pub fn init(root: &Path) -> Result<(), String> {
    if root.exists() {
        return Err(format!(
            "目录已存在，为保护你的学习记录不会覆盖：{}。请继续使用已有目录，或换一个新名称。",
            root.display()
        ));
    }
    // Path::parent() 对单层相对路径（如 `my-first-card`）返回 Some("")，
    // 而空路径的 is_dir() 为 false。它实际代表当前工作目录，不能拒绝创建。
    // Windows 双击入口和直接使用 Cargo 时都会使用这一类路径。
    let parent = root.parent().unwrap_or_else(|| Path::new("."));
    let parent = if parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        parent
    };
    if !parent.is_dir() {
        return Err(format!(
            "无法创建练习目录 {}：父目录 {} 不存在。请先检查路径，或改用已存在目录下的新名称。",
            root.display(),
            parent.display()
        ));
    }

    fs::create_dir(root).map_err(|err| format!("创建学习目录失败 {}：{err}", root.display()))?;
    fs::create_dir(root.join("src")).map_err(|err| format!("创建 src 失败：{err}"))?;
    write_new(&root.join("Cargo.toml"), CARGO_TOML)?;
    write_new(&root.join("src/main.rs"), MAIN_RS)?;
    write_new(
        &root.join("README.md"),
        &format!("{PROJECT_README}\n{UNITLAB_HANDOFF}"),
    )?;
    write_new(
        &root.join("跟着做.md"),
        &format!("{PROJECT_GUIDE}\n{UNITLAB_HANDOFF}"),
    )?;
    write_new(
        &root.join("LEARNING_LOG.md"),
        include_str!("../../../course/LEARNING_LOG_TEMPLATE.md"),
    )?;
    println!("[1/3] 已创建：{}", root.display());
    println!("[2/3] 现在用记事本打开 {}/src/main.rs", root.display());
    println!("[3/3] 只修改一处 println! 并保存，再运行：");
    println!(
        "  cargo run --locked --offline -p labquest -- beginner check \"{}\"",
        root.display()
    );
    println!("先打开工程中的 README.md，再按 跟着做.md 操作。上面的检查命令在完整 RustAppLab 平台根目录执行；初次检查失败是正常的。");
    Ok(())
}

pub fn hint(level: usize) -> Result<&'static str, String> {
    if !(1..=3).contains(&level) {
        return Err("用法：beginner hint <1|2|3>。建议先自己尝试，再逐级查看提示。".into());
    }
    Ok(HINTS[level - 1])
}

// 限制学员程序运行时间，避免错误的无限循环让新手以为电脑卡死。
fn run_bounded(binary: &Path, args: &[&str]) -> Result<Output, String> {
    let temp = crate::capstone::TempWorkspace::new_named("first-card-run")?;
    let mut command = Command::new(binary);
    command.args(args);
    crate::process::capture(&mut command, Duration::from_secs(4), &temp.0, "任务卡程序")
}

fn binary_at(root: &Path) -> PathBuf {
    let exe = if cfg!(windows) {
        "first-rust-card.exe"
    } else {
        "first-rust-card"
    };
    root.join("target").join("debug").join(exe)
}

pub fn check(root: &Path) -> Result<bool, String> {
    let manifest = root.join("Cargo.toml");
    if !manifest.is_file() || !root.join("src/main.rs").is_file() {
        return Err(format!(
            "没有找到学习工程：{}。请先运行 beginner start <新目录>，不能直接检查普通文件夹。",
            root.display()
        ));
    }
    println!("编译准备：正在构建你修改的 Rust 程序（首次编译会慢一点）...");
    std::io::stdout().flush().map_err(|err| err.to_string())?;
    let temp = crate::capstone::TempWorkspace::new_named("first-card-build")?;
    let mut command = Command::new("cargo");
    command
        .args(["build", "--offline", "--quiet", "--manifest-path"])
        .arg(&manifest)
        .arg("--target-dir")
        .arg(root.join("target"));
    crate::process::learner_cargo(&mut command);
    let built = crate::process::capture(
        &mut command,
        Duration::from_secs(120),
        &temp.0,
        "任务卡 Cargo 编译",
    )?;
    if !built.status.success() {
        let diagnostics = String::from_utf8_lossy(&built.stderr);
        let snippet: String = diagnostics.chars().take(2600).collect();
        println!("编译没有通过。先看看下面的错误提示（只显示前面一部分）：\n{snippet}");
        println!("先检查 println! 那一行的双引号、括号、分号是否完整。\n需要提示时，在完整 RustAppLab 平台根目录执行：\ncargo run --locked --offline -p labquest -- beginner hint 1\n保存源码后，仍在平台根目录重新检查：\ncargo run --locked --offline -p labquest -- beginner check \"{}\"\nWindows 如果出现 link.exe 找不到，请按官方 rustup 指引安装 MSVC C++ 构建工具。", root.display());
        return Ok(false);
    }
    let binary = binary_at(root);
    if !binary.is_file() {
        return Err(format!(
            "编译没有产生预期程序 {}。请检查 Cargo.toml 中的软件名称是否被修改。",
            binary.display()
        ));
    }
    let cases: &[(&[&str], &str)] = &[
        (&["小林", "写 Rust"], "今日任务卡 | 小林 | 写 Rust"),
        (&["Alex", "整理房间"], "今日任务卡 | Alex | 整理房间"),
        (&["林 小", "A&B <test>"], "今日任务卡 | 林 小 | A&B <test>"),
        (&[], "今日任务卡 | 朋友 | 学 Rust"),
    ];
    for (i, (args, expected)) in cases.iter().enumerate() {
        match run_bounded(&binary, args) {
            Ok(output) if output.status.success() => {
                let actual = String::from_utf8_lossy(&output.stdout);
                if actual.trim_end_matches(['\r', '\n']) != *expected {
                    println!("检查 {}/4：还没通过。", i + 1);
                    println!("  期待：{expected}");
                    println!("  实际：{}", crate::capstone::short(actual.trim(), 1200));
                    println!("你可能仍在输出占位文本，或把某个名字写死了。只修改 println!，让它使用 name 和 task 两个变量。\n需要提示时，在完整 RustAppLab 平台根目录执行：\ncargo run --locked --offline -p labquest -- beginner hint 1\n这不是安装失败。修改并保存后，仍在平台根目录重新检查：\ncargo run --locked --offline -p labquest -- beginner check \"{}\"", root.display());
                    return Ok(false);
                }
                println!("检查 {}/4：通过 ✓", i + 1);
            }
            Ok(output) => {
                println!(
                    "检查 {}/4：程序异常退出（退出码 {:?}）。\n{}",
                    i + 1,
                    output.status.code(),
                    String::from_utf8_lossy(&output.stderr)
                );
                return Ok(false);
            }
            Err(err) => {
                println!("检查 {}/4：{err}", i + 1);
                return Ok(false);
            }
        }
    }
    let badge = root.join("FIRST_WIN.md");
    if !badge.exists() {
        write_new(&badge, &format!("{FIRST_WIN_RECORD}\n{UNITLAB_HANDOFF}"))?;
    }
    println!("\n四项都通过了！你的第一个 Rust 作品已经能处理不同输入。");
    println!("学习记录：{}（已有记录不会覆盖）", badge.display());
    println!("下一步：在你的工程目录运行 cargo run --offline -- \"我的名字\" \"今天的任务\"，也可以把输出保存为 TXT。");
    println!("四组案例验证的是当次程序行为。再用两组自己的输入，并解释变量怎样改变输出；需要帮助的地方可记在学习记录中。");
    print!("{UNITLAB_HANDOFF}");
    Ok(true)
}
