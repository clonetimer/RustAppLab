//! 第二款独立作品 PageMint：从文本笔记生成可离线打开的 HTML。
//! 与 TaskPocket 不同，重点是输入数据管道、排序、结构化输出和 HTML 转义。
//! 学员程序仅在显式 `app check pagemint` 时编译执行；评测器不是沙箱。
use crate::capstone::Check;
use crate::capstone::{build_named, run_bounded, short, RunOutput, TempWorkspace};
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

const STAMP: &str = "<!-- RustAppLab PageMint Assessment v1 -->";
const MANIFEST: &str = r#"[package]
name = "pagemint"
version = "0.1.0"
edition = "2021"
rust-version = "1.74"

# 独立工程，避免意外加入 RustAppLab 的 Cargo Workspace。
[workspace]
"#;
const STUDENT: &str = r#"//! PageMint：从一组 .note 文件制作可携带的离线网页。
//! 初始工程可以编译，但核心功能有意留给你自己完成。
use std::env;

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.as_slice() == ["help"] {
        println!("PageMint: stats <notes_dir> | build <notes_dir> <output.html>");
        return Ok(());
    }
    // TODO(01): 严格解析 stats/build 和参数数量；验证输入目录。
    // TODO(02): 只读取直接子目录中的 .note 普通文件，按文件名排序。
    // TODO(03): 第一行是标题，后续非空行是段落；计算 stats。
    // TODO(04): HTML 转义标题、段落和文件名，生成离线 HTML。
    // TODO(05): 遇到非法输入不得创建或覆盖输出文件。
    Err("PageMint 尚未完成：先阅读 CONTRACT.md".into())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("PageMint: {error}");
        std::process::exit(2);
    }
}
"#;
const CONTRACT: &str = r#"# PageMint · 第二款独立软件需求合同（v1）

## 一句话说明

把你真实使用的 `.note` 笔记目录转换为**可离线打开的单文件 HTML 笔记集**。相较 TaskPocket 的任务增删与状态保存，本项目练习**读取多文件 → 排序 → 聚合 → 安全渲染 → 交付网页**，不需要联网或外部 Rust crate。

## 输入合同

- 传入的目录必须存在；只读取其**直接子级**、扩展名严格等于 `.note` 的**普通文件**；忽略 `.txt`、`.md` 和子目录，不递归。
- `.note` 为 UTF-8 文本；**第一行**去除前后空白作为标题，标题不能为空；后续逐行处理，去掉前后空白，**非空行**视为一个独立段落。CRLF 输入也要正确处理。
- 读取失败、UTF-8 无效、无标题或扩展名为 `.note` 的符号链接等不安全输入：操作失败、退出码非零，并且不能产生部分 HTML。
- 按 `.note` **文件名升序**展示，不依赖 `read_dir` 的随机遍历顺序。文件名可能有空格或 Unicode。

## 命令与固定行为

在 PageMint 工程目录执行：

| 命令 | 成功时 stdout | 副作用 |
|---|---|---|
| `cargo run -- stats <notes_dir>` | `notes=N paragraphs=M` 后换行 | 只统计，不创建文件 |
| `cargo run -- build <notes_dir> <output.html>` | `built=N` 后换行 | 输出一份完整 HTML |
| `cargo run -- help` | 任意简短帮助 | 无 |

`N` 是有效 `.note` 文件数，`M` 是所有笔记正文非空行数。错误信息在 stderr，退出码非零。命令或参数数量错误不得返回成功。

### HTML 输出最低结构（评测器仅检查关键结构与内容，不限制 CSS 美化）

```html
<!doctype html>
<html lang="zh-CN">
<head><meta charset="utf-8"><title>PageMint</title></head>
<body><main>
<article data-file="01-first.note"><h2>标题</h2><p>第一段</p><p>第二段</p></article>
</main></body></html>
```

- 必须包含 `<!doctype html>`（大小写不敏感）、`<meta charset="utf-8">` 和 `<title>PageMint</title>`；每份 `.note` 对应一个 `<article data-file="已转义文件名">`，标题为 `<h2>...</h2>`，每个正文非空行对应 `<p>...</p>`。
- **所有外部文本**（文件名、标题、段落）必须做 HTML 文本/属性转义，至少 `& → &amp;`、`< → &lt;`、`> → &gt;`、`" → &quot;`、`' → &#39;`；**先转义 &**，不要出现从笔记输入直接生成的 `<script>`。
- 网页不引用远程字体、脚本或 CDN；你可增加内联 `<style>` 改善排版和响应式布局，但不能破坏合同中的结构。建议把文章放入 `main`；也可以增加本地可用的搜索筛选。
- 仅当目标文件不存在时才能创建；**不能覆盖已有 HTML**。对非法输入先完整检查再写文件。为避免输入/输出相互影响，`output.html` 必须放在输入笔记目录**之外**，且其父目录已存在。

## 推荐的手动实验

```sh
cargo run -- stats samples/notes
cargo run -- build samples/notes samples/my-notes.html
```

第二条成功后浏览器打开 `samples/my-notes.html`。请先查看 `samples/notes` 中的真实文件再动手。因为拒绝覆盖目标，重复 build 请改用新文件名或先人工确认再删除旧产物。

## 迁移验收

在 RustAppLab 根目录运行：

```sh
cargo run -p labquest -- app check pagemint <项目目录>
```

评测器在**多个独立临时目录**中创建动态输入，实际编译你的程序、启动多个独立进程、检查生成的 HTML 和错误路径是否未产生文件；它不是安全沙箱，只评测自己信任的代码。评分只能证明合同覆盖，不等于真人已掌握 Rust。

## 延伸挑战（不计入基础评分）

加入可访问的搜索控件（离线 JavaScript）、标签、`--theme`、图片本地相对引用和快照备份；再用 SafeSync 封存成品。网页安全还包括 CSS/JS、URL、路径和内容安全策略等，本练习的转义要求不等于完整安全审计。
"#;
const JOURNAL: &str = r#"# PageMint · 我的独立作品记录

## 我想整理的真实笔记是什么？

（例如课程笔记、旅行清单、工作会议摘录）

## 我想先验证的三个实验

- [ ] `stats` 能计算两份样例笔记的数量与段落数
- [ ] `build` 可生成浏览器可打开的离线网页
- [ ] 输入加入 `<script>`、中文和带 `&` 的文件名后仍能正确显示为文本

## 设计与调试记录

| 失败现象 | 我的推测 | 改动 | 复验 |
|---|---|---|---|
| | | | |

## 最终交付

- 页面文件位置：
- 我的真实笔记有多少份：
- 我准备如何向别人说明这个软件的使用方式：
- 与 TaskPocket 相比，Rust 哪些知识是我复用的，哪些是新学的：
"#;
const HINTS: [&str; 3] = [
    "第 1 层：先只做 stats：遍历 read_dir，筛出扩展名为 note 的普通文件，按文件名排序。先让两个样例的统计正确，不必一开始做网页或 UI。",
    "第 2 层：定义 Note { file: String, title: String, paragraphs: Vec<String> }。把读取放在 read_notes(dir) -> Result<Vec<Note>, String>，stats 和 build 都复用这一层。用 lines() 读取 UTF-8、trim() 跳过空段落。",
    "第 3 层：写 escape_html(s: &str) -> String，按字符把 & < > 双引号、单引号替换成 HTML 实体，不能直接拼入不可信文本。用 create_new(true) 防止覆盖；先读完整目录并校验，再开始写入 HTML。可用 String::push_str 构造单文件网页。",
];

pub fn hint(tier: usize) -> Result<&'static str, String> {
    HINTS
        .get(tier.checked_sub(1).ok_or("提示层级必须在 1 到 3 之间")?)
        .copied()
        .ok_or("用法：labquest app hint pagemint <1|2|3>".into())
}

pub fn init(root: &Path) -> Result<(), String> {
    fs::create_dir(root)
        .map_err(|err| format!("只能创建不存在的新工程目录 {}：{err}", root.display()))?;
    fs::create_dir(root.join("src")).map_err(|e| e.to_string())?;
    fs::create_dir_all(root.join("samples/notes")).map_err(|e| e.to_string())?;
    let files = [
        ("Cargo.toml", MANIFEST),
        ("src/main.rs", STUDENT),
        ("CONTRACT.md", CONTRACT),
        ("PROJECT_JOURNAL.md", JOURNAL),
        (".gitignore", "/target/\n/.applab-target/\n/samples/*.html\n"),
        (
            "samples/notes/01-first.note",
            "  Rust 实践第一天  \n今天学习了 Cargo 和 Result。\n\n我终于写出第一个真正有用的软件。\n",
        ),
        (
            "samples/notes/02-ideas.note",
            "  下一步的想法  \n支持中文 & 特殊字符\n不要把 <script> 当成 HTML 标签。\n",
        ),
    ];
    for (filename, content) in files {
        let file = root.join(filename);
        let mut out = File::options()
            .write(true)
            .create_new(true)
            .open(&file)
            .map_err(|e| format!("创建 {} 失败：{e}", file.display()))?;
        out.write_all(content.as_bytes())
            .map_err(|e| format!("写入 {} 失败：{e}", file.display()))?;
    }
    crate::independent::write_materials(root, "pagemint")?;
    Ok(())
}

struct Harness {
    executable: PathBuf,
    tmp: TempWorkspace,
}
impl Harness {
    fn folder(&self, name: &str) -> Result<PathBuf, String> {
        let dir = self.tmp.0.join(name);
        fs::create_dir(&dir).map_err(|e| e.to_string())?;
        Ok(dir)
    }
    fn command(&self, args: &[OsString]) -> Result<RunOutput, String> {
        let mut cmd = Command::new(&self.executable);
        cmd.args(args).current_dir(&self.tmp.0);
        run_bounded(
            &mut cmd,
            Duration::from_secs(5),
            &self.tmp.0,
            "PageMint 程序",
        )
    }
    fn invoke(
        &self,
        command: &str,
        input: &Path,
        output: Option<&Path>,
    ) -> Result<RunOutput, String> {
        let mut args = vec![OsString::from(command), input.as_os_str().to_owned()];
        if let Some(target) = output {
            args.push(target.as_os_str().to_owned());
        }
        self.command(&args)
    }
    fn success(
        &self,
        command: &str,
        input: &Path,
        output: Option<&Path>,
        expected: &str,
    ) -> Result<(), String> {
        let actual = self.invoke(command, input, output)?;
        if !actual.success {
            return Err(format!("{command} 应成功：{}", short(&actual.stderr, 250)));
        }
        if actual.stdout != expected {
            return Err(format!(
                "{command} 标准输出不符：预期 {expected:?}，实际 {:?}",
                short(&actual.stdout, 200)
            ));
        }
        Ok(())
    }
    fn reject(&self, command: &str, input: &Path, output: Option<&Path>) -> Result<(), String> {
        let actual = self.invoke(command, input, output)?;
        if actual.success {
            return Err(format!("{command} 对非法输入返回成功退出码"));
        }
        Ok(())
    }
}

fn write_note(input: &Path, name: &str, text: &str) -> Result<(), String> {
    fs::write(input.join(name), text).map_err(|e| e.to_string())
}
fn html(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("没有生成 HTML {}：{e}", path.display()))
}
fn structure(output: &str, count: usize) -> Result<(), String> {
    let lowered = output.to_ascii_lowercase();
    if !lowered.contains("<!doctype html>")
        || !lowered.contains("<meta charset=\"utf-8\">")
        || !output.contains("<title>PageMint</title>")
        || output.matches("<article data-file=\"").count() != count
    {
        Err("页面缺少 DOCTYPE / UTF-8 / PageMint 标题，或 article 数量不符".into())
    } else {
        Ok(())
    }
}
fn stats_only(h: &Harness) -> Result<(), String> {
    let dir = h.folder("只读 notes")?;
    h.success("stats", &dir, None, "notes=0 paragraphs=0\n")?;
    write_note(
        &dir,
        "02-next.note",
        "  笔记 B  \r\n第一段\r\n\r\n第二段\r\n",
    )?;
    write_note(&dir, "01-first.note", "第一天\n学习文件\n")?;
    write_note(&dir, "ignore.txt", "不是笔记")?;
    let sub = dir.join("nested");
    fs::create_dir(&sub).map_err(|e| e.to_string())?;
    write_note(&sub, "skip.note", "子目录不读取\n跳过\n")?;
    h.success("stats", &dir, None, "notes=2 paragraphs=3\n")?;
    write_note(&dir, "01-first.note", "第一天\n学习文件\n新增一段\n")?;
    let before = fs::read(dir.join("01-first.note")).map_err(|e| e.to_string())?;
    h.success("stats", &dir, None, "notes=2 paragraphs=4\n")?;
    let after = fs::read(dir.join("01-first.note")).map_err(|e| e.to_string())?;
    if before != after || fs::read_dir(&dir).map_err(|e| e.to_string())?.count() != 4 {
        return Err("stats 不能修改源笔记或创建额外产物".into());
    }
    Ok(())
}

fn empty_notes(h: &Harness) -> Result<(), String> {
    stats_only(h)?;
    let input = h.folder("empty-notes")?;
    let target = h.tmp.0.join("empty.html");
    h.success("stats", &input, None, "notes=0 paragraphs=0\n")?;
    h.success("build", &input, Some(&target), "built=0\n")?;
    structure(&html(&target)?, 0)
}
fn actual_content(h: &Harness) -> Result<(), String> {
    let input = h.folder("actual-notes")?;
    write_note(
        &input,
        "02-next.note",
        "  Rust 进阶  \n文件与所有权\n\n错误处理\n",
    )?;
    write_note(&input, "01-first.note", "  第一天  \n认识 Cargo\n")?;
    let target = h.tmp.0.join("actual.html");
    h.success("stats", &input, None, "notes=2 paragraphs=3\n")?;
    h.success("build", &input, Some(&target), "built=2\n")?;
    let output = html(&target)?;
    structure(&output, 2)?;
    for required in [
        "<article data-file=\"01-first.note\">",
        "<h2>第一天</h2>",
        "<p>认识 Cargo</p>",
        "<article data-file=\"02-next.note\">",
        "<h2>Rust 进阶</h2>",
        "<p>文件与所有权</p>",
        "<p>错误处理</p>",
    ] {
        if !output.contains(required) {
            return Err(format!("HTML 缺少 {required}"));
        }
    }
    if output.find("01-first.note") >= output.find("02-next.note") {
        return Err("文章没有按照文件名顺序展示".into());
    }
    Ok(())
}
fn variable_input(h: &Harness) -> Result<(), String> {
    let input = h.folder("dynamic-notes")?;
    let marker = format!(
        "session-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("grade")
    );
    write_note(&input, "three.note", &format!("{marker}\nAlpha\nBeta\n"))?;
    let target1 = h.tmp.0.join("dynamic-first.html");
    h.success("stats", &input, None, "notes=1 paragraphs=2\n")?;
    h.success("build", &input, Some(&target1), "built=1\n")?;
    if !html(&target1)?.contains(&format!("<h2>{marker}</h2>")) {
        return Err("HTML 似乎使用固定模板而非读取真实源文件".into());
    }
    write_note(&input, "another.note", "Extra\nOne\nTwo\nThree\n")?;
    let target2 = h.tmp.0.join("dynamic-second.html");
    h.success("stats", &input, None, "notes=2 paragraphs=5\n")?;
    h.success("build", &input, Some(&target2), "built=2\n")?;
    structure(&html(&target2)?, 2)?;
    Ok(())
}
fn encoding_and_escaping(h: &Harness) -> Result<(), String> {
    let input = h.folder("utf8-notes")?;
    write_note(
        &input,
        "蓝色 & 想法.note",
        " <计划> \r\nA & B\r\n\"hello\" and 'world'\r\n<script>alert(1)</script>\r\n",
    )?;
    let target = h.tmp.0.join("safe.html");
    h.success("stats", &input, None, "notes=1 paragraphs=3\n")?;
    h.success("build", &input, Some(&target), "built=1\n")?;
    let output = html(&target)?;
    structure(&output, 1)?;
    for required in [
        "data-file=\"蓝色 &amp; 想法.note\"",
        "<h2>&lt;计划&gt;</h2>",
        "<p>A &amp; B</p>",
        "<p>&quot;hello&quot; and &#39;world&#39;</p>",
        "<p>&lt;script&gt;alert(1)&lt;/script&gt;</p>",
    ] {
        if !output.contains(required) {
            return Err(format!("转义不足或丢失 UTF-8：{required}"));
        }
    }
    if output.contains("<script>alert(1)</script>") {
        return Err("笔记源内容被当作可执行 HTML 插入".into());
    }
    Ok(())
}
fn ignore_others(h: &Harness) -> Result<(), String> {
    let input = h.folder("mixed-input")?;
    write_note(&input, "keep.note", "Keep\nOne\n")?;
    write_note(&input, "skip.md", "Skip\nMore\n")?;
    write_note(&input, "skip.NOTE", "Skip uppercase\n")?;
    let sub = input.join("inner");
    fs::create_dir(&sub).map_err(|e| e.to_string())?;
    write_note(&sub, "nested.note", "Nested\nWrong\n")?;
    let target = h.tmp.0.join("mixed.html");
    h.success("stats", &input, None, "notes=1 paragraphs=1\n")?;
    h.success("build", &input, Some(&target), "built=1\n")?;
    let output = html(&target)?;
    structure(&output, 1)?;
    if output.contains("Nested") || output.contains("skip.md") {
        return Err("程序递归或错误处理了非 .note 文件".into());
    }
    Ok(())
}
fn malformed_input(h: &Harness) -> Result<(), String> {
    for (name, contents) in [
        ("blank", b"   \nhello\n".as_slice()),
        ("utf8", &[0xff, 0xfe, 0xfd][..]),
    ] {
        let input = h.folder(name)?;
        fs::write(input.join("broken.note"), contents).map_err(|e| e.to_string())?;
        let target = h.tmp.0.join(format!("{name}.html"));
        h.reject("stats", &input, None)?;
        h.reject("build", &input, Some(&target))?;
        if target.exists() {
            return Err("读取失败却产生了部分 HTML 文件".into());
        }
    }
    Ok(())
}
fn refuse_overwrite(h: &Harness) -> Result<(), String> {
    let input = h.folder("overwrite-notes")?;
    write_note(&input, "keep.note", "Keep\nHello\n")?;
    let target = h.tmp.0.join("important.html");
    fs::write(&target, b"THIS IS MY DOCUMENT").map_err(|e| e.to_string())?;
    h.reject("build", &input, Some(&target))?;
    if fs::read(&target).map_err(|e| e.to_string())? != b"THIS IS MY DOCUMENT" {
        return Err("原有网页被覆盖或截断".into());
    }
    let inside = input.join("inside.html");
    h.reject("build", &input, Some(&inside))?;
    if inside.exists() {
        return Err("不应在原始笔记目录里生成输出网页".into());
    }
    Ok(())
}
fn bad_paths_and_cli(h: &Harness) -> Result<(), String> {
    let missing = h.tmp.0.join("no-such-notes");
    let target = h.tmp.0.join("bad.html");
    h.reject("stats", &missing, None)?;
    h.reject("build", &missing, Some(&target))?;
    let input = h.folder("bad-cli")?;
    for args in [
        vec![OsString::from("stats")],
        vec![OsString::from("unknown"), input.as_os_str().to_os_string()],
        vec![OsString::from("build"), input.as_os_str().to_os_string()],
    ] {
        if h.command(&args)?.success {
            return Err("非法参数应非零退出".into());
        }
    }
    if target.exists() {
        return Err("非法输入不应产生网页".into());
    }
    Ok(())
}
#[cfg(unix)]
fn symlink_input(h: &Harness) -> Result<(), String> {
    let input = h.folder("links-notes")?;
    let outside = h.tmp.0.join("outside.note");
    write_note(&h.tmp.0, "outside.note", "SECRET\ncontent\n")?;
    std::os::unix::fs::symlink(&outside, input.join("escape.note")).map_err(|e| e.to_string())?;
    let target = h.tmp.0.join("linked.html");
    h.reject("build", &input, Some(&target))?;
    if target.exists() {
        return Err("不应导出符号链接目标".into());
    }
    h.reject("stats", &input, None)
}

type Probe = fn(&Harness) -> Result<(), String>;
pub(crate) fn assess_stage(root: &Path, stage: usize) -> Vec<Check> {
    let tmp = match TempWorkspace::new_named("pagemint") {
        Ok(dir) => dir,
        Err(err) => {
            return vec![Check {
                name: "环境检查",
                passed: false,
                feedback: err,
            }]
        }
    };
    let exe = match build_named(root, &tmp, "pagemint") {
        Ok(file) => file,
        Err(err) => {
            return vec![Check {
                name: "Cargo 编译",
                passed: false,
                feedback: err,
            }]
        }
    };
    let h = Harness {
        executable: exe,
        tmp,
    };
    let probes: Vec<(&str, Probe, &str)> = vec![
        (
            "01 空目录可生成网页",
            empty_notes,
            "先处理空 Vec，不要把无笔记视为错误",
        ),
        (
            "02 真实文本变成有序文章",
            actual_content,
            "检查标题、段落和文件名排序",
        ),
        (
            "03 修改文件后结果动态变化",
            variable_input,
            "不要把文件数量、标题或输出写死",
        ),
        (
            "04 Unicode 与 HTML 转义",
            encoding_and_escaping,
            "将文件名/标题/段落的五类 HTML 特殊字符转义",
        ),
        (
            "05 忽略非 note 文件和子目录",
            ignore_others,
            "只读取输入目录直接子级、扩展名精确为 note 的文件",
        ),
        (
            "06 损坏的源内容拒绝写入",
            malformed_input,
            "先加载并验证所有笔记，再 create_new 创建目标",
        ),
        (
            "07 拒绝覆盖和目录内输出",
            refuse_overwrite,
            "输出必须在输入目录之外；已有目标要原样保留",
        ),
        (
            "08 非法 CLI 和不存在的目录",
            bad_paths_and_cli,
            "检查参数数量、命令名和目录存在性",
        ),
    ];
    // 只有 Unix 会追加符号链接测试；Windows 构建不需要可变变量。
    #[cfg(unix)]
    let mut probes = probes;
    #[cfg(unix)]
    probes.push((
        "09 拒绝符号链接源",
        symlink_input,
        "检查 symlink_metadata 与 FileType，拒绝跟随链接",
    ));
    let probes: Vec<(&str, Probe, &str)> = if stage == 1 {
        vec![(
            "01 只读统计与动态输入",
            stats_only,
            "先完成 stats，尚不要求 build；仅读取直接子级 note 文件",
        )]
    } else if stage == 2 {
        probes.into_iter().take(5).collect()
    } else {
        probes
    };
    probes
        .into_iter()
        .map(|(name, probe, hint)| match probe(&h) {
            Ok(()) => Check {
                name,
                passed: true,
                feedback: "通过：真实命令及产物符合需求".into(),
            },
            Err(err) => Check {
                name,
                passed: false,
                feedback: format!("{}。建议：{hint}", short(&err, 450)),
            },
        })
        .collect()
}
pub(crate) fn report(root: &Path, checks: &[Check]) -> Result<PathBuf, String> {
    let path = root.join("APP_ASSESSMENT.md");
    match fs::symlink_metadata(&path) {
        Ok(meta) => {
            if !meta.file_type().is_file() || meta.file_type().is_symlink() {
                return Err("拒绝覆盖非普通文件类型的评测报告".into());
            }
            let existing = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            if !existing.starts_with(STAMP) {
                return Err("APP_ASSESSMENT.md 是你自己的文件，拒绝覆盖，请先备份".into());
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.to_string()),
    }
    let passed = checks.iter().filter(|check| check.passed).count();
    let mut body = format!("{STAMP}\n# PageMint · 独立作品验收报告\n\n{passed}/{} 项通过；结果来自实际 Cargo 编译、独立 CLI 进程和网页文件检查。不是学习成效认证。\n\n", checks.len());
    for check in checks {
        body.push_str(&format!(
            "## {} {}\n\n{}\n\n",
            if check.passed { "PASS" } else { "FAIL" },
            check.name,
            check.feedback.replace('\n', " ")
        ));
    }
    body.push_str("## 下一步\n\n先修复第一个 FAIL，再重新评测；全通过后用**自己的真实笔记**生成网页、打开查看并填写 PROJECT_JOURNAL.md。\n\n注意：评测器不是沙箱，不要运行不可信 Rust 源码。\n");
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)
        .map_err(|e| e.to_string())?;
    file.write_all(body.as_bytes()).map_err(|e| e.to_string())?;
    Ok(path)
}
pub fn check(root: &Path) -> Result<bool, String> {
    crate::independent::check(root, "pagemint", 3)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn starter_is_not_completed() {
        assert!(STUDENT.contains("TODO(04)"));
        assert!(!STUDENT.contains("create_new"));
        assert!(CONTRACT.contains("data-file"));
        assert!(CONTRACT.contains("HTML"));
    }
    #[test]
    fn hints_are_scaffolded() {
        assert!(hint(1).unwrap().contains("stats"));
        assert!(hint(3).unwrap().contains("escape_html"));
        assert!(hint(0).is_err());
        assert!(hint(4).is_err());
    }
}
