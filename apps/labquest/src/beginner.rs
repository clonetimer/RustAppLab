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

const PROJECT_GUIDE: &str = r#"# 我的第一个 Rust 软件：今日任务卡

本任务只需要修改 **src/main.rs 中唯一一个 println! 行**，不要一开始研究其它源码。

## 第 1 步：先运行未完成的程序

打开 PowerShell，在本工程目录（能看到 Cargo.toml 的文件夹）执行：

```powershell
cargo run --offline -- "小林" "学习 Rust"
```

第一次应该显示 **等待完成第一张任务卡**。这不是错误，只表示你还没完成它。

## 第 2 步：修改一行 Rust 代码

用记事本或 VS Code 打开 `src/main.rs`，找到 `println!("等待完成第一张任务卡");`。
修改该行，使程序使用 `name` 和 `task` 两个变量，输出以下格式：

```text
今日任务卡 | 小林 | 学习 Rust
```

只改一行并保存（Ctrl+S）。这里的 `println!` 是打印；`{name}` 是把变量的值放进文本。

## 第 3 步：自己运行一次

```powershell
cargo run --offline -- "小林" "学习 Rust"
```

正确时会显示任务卡。试着改成你的名字和真实任务，再运行一次。

## 第 4 步：回到 RustAppLab 根目录自动检查

```powershell
cargo run --offline -p labquest -- beginner check my-first-card
```

上面的相对路径适用于你把本项目命名为 `my-first-card`，并放在 RustAppLab 根目录。
如果你使用其它名字，请替换为自己的目录名。
检查会先编译你的程序，再用中文、英文、带空格的输入进行测试；不能只写死一个答案。

## 第 5 步：把自己的软件变成作品

在本工程目录运行：

```powershell
cargo run --offline -- "我的名字" "完成 Rust 入门" | Out-File -Encoding utf8 "我的任务卡.txt"
```

然后打开 `我的任务卡.txt`。如果你已经有同名文件，请换个名称，不要覆盖自己的文件。

## 如果卡住

- `cargo 不是命令`：Rust 尚未安装或需要重新启动 PowerShell，先看 RustAppLab 根目录 `START_HERE.html`
- `找不到 Cargo.toml`：先查看终端的当前目录；每条命令必须在说明的位置执行
- `error: expected ...`：通常是引号、括号、分号不成对，先比较编辑前后是否只改了一行
- 输出还是“等待完成”：可能忘了按 Ctrl+S 保存文件
- 提示多个测试失败：不要写死“小林”，程序要使用 `name` 和 `task` 变量

提示分为三层：在 RustAppLab 根目录执行 `cargo run -p labquest -- beginner hint 1`；实在卡住再看 2、3。
"#;

const HINTS: [&str; 3] = [
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
    write_new(&root.join("跟着做.md"), PROJECT_GUIDE)?;
    println!("[1/3] 已创建：{}", root.display());
    println!("[2/3] 现在用记事本打开 {}/src/main.rs", root.display());
    println!("[3/3] 只修改一处 println! 并保存，再运行：");
    println!(
        "  cargo run -p labquest -- beginner check \"{}\"",
        root.display()
    );
    println!("不会 Rust 也没关系：详细步骤在工程中的 跟着做.md；提示用 beginner hint 1。此时 check 失败是正常的。");
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
        println!("先检查 println! 那一行的双引号、括号、分号是否完整；然后再运行 beginner check。需要时用 beginner hint 1。\nWindows 如果出现 link.exe 找不到，请按官方 rustup 指引安装 MSVC C++ 构建工具。");
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
                    println!("你可能仍在输出占位文本，或把某个名字写死了。只修改 println!，让它使用 name 和 task 两个变量。提示：beginner hint 1。\n这不是安装失败，可以继续修改代码并重试。");
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
    let content = "# 我完成了第一款 Rust 小软件！\n\n我亲手修改了 src/main.rs，使用变量生成今日任务卡。程序在 4 组输入下通过自动检查（包含中文、英文和默认值）。\n\n这只是本地练习记录，不是职业认证。\n\n## 我的真实作品\n\n在本目录 PowerShell 中运行：\n\n```powershell\ncargo run --offline -- \"我的名字\" \"今天要做的事\" | Out-File -Encoding utf8 \"我的任务卡.txt\"\n```\n\n打开 TXT 阅读自己的作品；再尝试修改任务内容。下一关返回 RustAppLab 根目录，双击 CONTINUE_WINDOWS.cmd，按 Enter 从 UnitLab 的第 1 步开始；不是先研究完整平台源码。\n";
    if !badge.exists() {
        write_new(&badge, content)?;
    }
    println!("\n四项都通过了！你的第一个 Rust 作品已经能处理不同输入。");
    println!("学习记录：{}（已有记录不会覆盖）", badge.display());
    println!("下一步：在你的工程目录运行 cargo run --offline -- \"我的名字\" \"今天的任务\"，也可以把输出保存为 TXT。");
    Ok(true)
}
