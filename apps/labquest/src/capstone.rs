//! 0.9.0 独立作品：TaskPocket 的初始化、黑盒 CLI 验收与分层反馈。
//!
//! 仅在用户显式执行 `app check` 时编译、运行自己编写的代码。
//! 这不是沙箱；请勿评测来自不可信来源的 Rust 项目。
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const STAMP: &str = "<!-- RustAppLab TaskPocket Assessment v1 -->";
static NONCE: AtomicU64 = AtomicU64::new(0);
const CARGO_MANIFEST: &str = r#"[package]
name = "taskpocket"
version = "0.1.0"
edition = "2021"
rust-version = "1.74"

# 此项目独立于 RustAppLab Workspace，可单独交付。
[workspace]
"#;
const STUDENT_SOURCE: &str = r#"//! TaskPocket：你的第一个独立 Rust 命令行产品。
//! 在阅读 CONTRACT.md 后，把 TODO 替换为你自己的实现。
use std::env;

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("help") {
        println!("TaskPocket: add / list / done / stats，请阅读 CONTRACT.md");
        return Ok(());
    }
    // TODO(01) 解析四种命令及参数。
    // TODO(02) 从指定 TSV 文件读取并验证记录；不存在时当作空表。
    // TODO(03) add 持久化任务；done 更新状态；list/stats 输出固定协议。
    // TODO(04) 非法输入应返回错误，且不能损坏已有记录。
    Err("尚未实现：请从 CONTRACT.md 的第一项功能开始".into())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("TaskPocket: {error}");
        std::process::exit(2);
    }
}
"#;
const CONTRACT: &str = r#"# TaskPocket · 独立开发需求合同（v1）

## 软件解决什么问题

在**没有网络和数据库服务**的电脑上，维护一个可携带的任务列表。数据文件可被文本编辑器打开、备份或通过其他工具读取。这不是编程语法题：最终必须产出能在不同目录、不同任务上重复使用的程序。

## 必须实现的命令

在本目录执行 `cargo run -- <command> <db_path> [value]`：

| 命令 | 行为 | 成功时 stdout（必须严格一致，末尾换行） |
|---|---|---|
| `add <db> <title>` | 添加一条待办 | `added=<id>` |
| `list <db>` | 按 ID 递增输出全部记录 | 每行 `<id>\t<OPEN或DONE>\t<title>`，空表输出空文本 |
| `done <db> <id>` | 将已有任务标为完成；重复完成仍成功 | `done=<id>` |
| `stats <db>` | 输出任务总数、待办与完成数 | `total=N open=N done=N` |

规则：

1. `<db>` 是命令行指定的 UTF-8 TSV 文件路径；**不能写死文件路径**。不同命令、不同进程都要读写同一个文件。若文件不存在，按空列表处理；`list/stats` 不需要创建文件。
2. 文件中**没有表头**，每条记录占一行，格式是 `<正整数id>\t<OPEN或DONE>\t<title>\n`；使用 UTF-8。ID 从 1 开始，新增时按当前最大 ID + 1 编号，不因任务完成而复用。
3. `title` 去除**前后空白**后存储，支持中文及 Unicode；标题为空、包含 `\t`、`\r`、`\n` 时必须拒绝。其他普通字符保留原样。
4. 缺少参数、未知命令、非法 ID、目标 ID 不存在、已存在文件中的格式错误或重复 ID：必须用**非零退出码**提示错误；**不得修改原文件**。对已有 DONE 再执行 done 是成功的幂等操作。
5. 所有成功命令的 stdout 只有上表所述内容，错误信息放 stderr。默认不用网络、账户、外部数据库或非标准库依赖。尽量使用 `Result` 和错误传播，不要只在成功路径写代码。
6. 没有删除命令；不要修改真实任务文件来进行故障注入。验收会在临时目录中创建多组数据库，检查**真实磁盘内容、跨进程持久化、动态任务及失败后数据不变**。

## 人工验收示例

```sh
cargo run -- add work.tsv "修复启动问题"
cargo run -- add work.tsv "编写使用说明"
cargo run -- list work.tsv
cargo run -- done work.tsv 1
cargo run -- stats work.tsv
```

应依次看到：`added=1`、`added=2`、两行 OPEN、`done=1`、`total=2 open=1 done=1`。完成后使用 RustAppLab 根目录的 `cargo run -p labquest -- app check <本项目路径>` 运行迁移验收。

## 进阶（不参与基础自动评分）

编写单元测试、采用临时文件 + rename 减少部分写入风险，增加 `search` 或 `export`，用 `cargo fmt` 和 Clippy 检查。实践中还需要考虑断电、权限、并发写入及路径替换；本教学案例不声称解决这些生产可靠性问题。
"#;
const JOURNAL: &str = r#"# 我的 TaskPocket 开发记录

**我要解决的真实问题：**（举例：追踪个人学习计划）

## 第一次能运行

- [ ] 使用 `cargo run -- help` 运行程序
- [ ] 实现 `list` / `stats` 的空列表行为
- [ ] 第一次 `add` 后重新启动进程，能看到记录

## 逐步完善

- [ ] `done` 能修改持久化数据；重复 done 无害
- [ ] 中文、前后空格以及非法输入都有明确行为
- [ ] 能解释 `Result`、借用/所有权、`Vec`、文件 I/O 的具体用途
- [ ] 在未看完整答案的情况下通过 `labquest app check`

## 我的失败实验和复盘

| 观察到的错误 | 原因推测 | 我做的修改 | 复验结果 |
|---|---|---|---|
| | | | |

## 作品交付

- 可执行程序或源码仓库：
- 我使用它管理了哪些实际任务：
- 我还能独立开发的下一款软件：
"#;
const HINTS: [&str; 3] = [
    "第 1 层：先完成空库的 list/stats，再实现 add。将 CLI 参数作为 Vec<String> 接收，把存储定义为 Vec<Task>；不要先考虑附加功能。先手工打开 TSV 看每行到底有几个字段。",
    "第 2 层：Task 可以包含 id: u64、done: bool、title: String；用 split('\\t') 检查恰好三段，用 parse::<u64>() 验证 ID。存储层分成 load(path) -> Result<Vec<Task>, String>、save(path, tasks) -> Result<(), String>；命令层只做业务逻辑。",
    "第 3 层：成功之前先做验证：add 先 trim 标题并排除换行/制表符；done 先 parse ID 和查找目标；load 遇到损坏行或重复 ID 应返回 Err。只有解析和参数验证全部通过后才保存，避免错误输入破坏文件。用 std::fs::read_to_string / std::fs::write 起步，后续再尝试原子写入。",
];

pub fn hint(tier: usize) -> Result<&'static str, String> {
    if (1..=HINTS.len()).contains(&tier) {
        Ok(HINTS[tier - 1])
    } else {
        Err("用法：labquest app hint <1|2|3>".into())
    }
}

pub fn init(root: &Path) -> Result<(), String> {
    // 唯一新目录，拒绝覆盖；不尝试删除任何已存在的学员文件。
    fs::create_dir(root)
        .map_err(|err| format!("只能初始化不存在的新目录 {}：{err}", root.display()))?;
    let result = (|| {
        fs::create_dir(root.join("src")).map_err(|e| e.to_string())?;
        for (path, contents) in [
            ("Cargo.toml", CARGO_MANIFEST),
            ("src/main.rs", STUDENT_SOURCE),
            ("CONTRACT.md", CONTRACT),
            ("PROJECT_JOURNAL.md", JOURNAL),
            (".gitignore", "/target/\n/.applab-target/\n/*.tsv\n"),
        ] {
            let target = root.join(path);
            let mut file = File::options()
                .write(true)
                .create_new(true)
                .open(&target)
                .map_err(|e| format!("创建 {} 失败：{e}", target.display()))?;
            file.write_all(contents.as_bytes())
                .map_err(|e| format!("写入 {} 失败：{e}", target.display()))?;
        }
        crate::independent::write_materials(root, "taskpocket")?;
        Ok::<(), String>(())
    })();
    // 首次创建失败时不删除目录：并发修改或意外文件应由用户确认。
    result
}

pub(crate) struct TempWorkspace(pub(crate) PathBuf);
impl TempWorkspace {
    fn new() -> Result<Self, String> {
        Self::new_named("taskpocket")
    }
    pub(crate) fn new_named(product: &str) -> Result<Self, String> {
        let count = NONCE.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| err.to_string())?
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "rust-applab-{product}-{}-{nanos}-{count}",
            std::process::id()
        ));
        fs::create_dir(&dir).map_err(|err| format!("创建测试临时目录失败：{err}"))?;
        Ok(Self(dir))
    }
    fn db(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}
impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub(crate) struct RunOutput {
    pub(crate) success: bool,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
}

pub(crate) fn run_bounded(
    command: &mut Command,
    timeout: Duration,
    scratch: &Path,
    label: &str,
) -> Result<RunOutput, String> {
    let output = crate::process::capture(command, timeout, scratch, label)?;
    let text = |bytes: Vec<u8>| {
        String::from_utf8(bytes)
            .map(|s| s.replace("\r\n", "\n"))
            .map_err(|_| "子进程输出不是 UTF-8 文本".to_string())
    };
    Ok(RunOutput {
        success: output.status.success(),
        stdout: text(output.stdout)?,
        stderr: text(output.stderr)?,
    })
}

fn build(root: &Path, tmp: &TempWorkspace) -> Result<PathBuf, String> {
    build_named(root, tmp, "taskpocket")
}

pub(crate) fn build_named(
    root: &Path,
    tmp: &TempWorkspace,
    package: &str,
) -> Result<PathBuf, String> {
    let manifest = root.join("Cargo.toml");
    if !manifest.is_file() || !root.join("src/main.rs").is_file() {
        return Err("找不到 Cargo.toml 或 src/main.rs；先执行 labquest app init <新目录>".into());
    }
    let target = root.join(".applab-target");
    let mut command = Command::new("cargo");
    command
        .arg("build")
        .arg("--offline")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg(&manifest)
        .arg("--target-dir")
        .arg(&target)
        .env_remove("CARGO_BUILD_TARGET");
    crate::process::learner_cargo(&mut command);
    let outcome = run_bounded(
        &mut command,
        Duration::from_secs(120),
        &tmp.0,
        "学生项目 Cargo 编译",
    )?;
    if !outcome.success {
        return Err(format!("Cargo 编译失败：{}", short(&outcome.stderr, 1600)));
    }
    let exe = target
        .join("debug")
        .join(format!("{package}{}", std::env::consts::EXE_SUFFIX));
    if !exe.is_file() {
        return Err(format!(
            "Cargo 构建成功但没有 {package} 可执行程序；请保持包名 {package}"
        ));
    }
    Ok(exe)
}

pub(crate) fn short(input: &str, max: usize) -> String {
    let s: String = input.chars().take(max).collect();
    if input.chars().count() > max {
        format!("{s}…")
    } else {
        s
    }
}

struct Harness {
    exe: PathBuf,
    tmp: TempWorkspace,
}
impl Harness {
    fn run(&self, args: &[OsString]) -> Result<RunOutput, String> {
        let mut command = Command::new(&self.exe);
        command.args(args).current_dir(&self.tmp.0);
        run_bounded(
            &mut command,
            Duration::from_secs(5),
            &self.tmp.0,
            "TaskPocket 命令",
        )
    }
    fn invoke(&self, command: &str, db: &Path, extra: &[&str]) -> Result<RunOutput, String> {
        let mut args = vec![OsString::from(command), db.as_os_str().to_owned()];
        args.extend(extra.iter().map(OsString::from));
        self.run(&args)
    }
    fn ok(&self, command: &str, db: &Path, extra: &[&str], expected: &str) -> Result<(), String> {
        let out = self.invoke(command, db, extra)?;
        if !out.success {
            return Err(format!(
                "{command} 应成功，但退出非零：{}",
                short(&out.stderr, 250)
            ));
        }
        if out.stdout != expected {
            return Err(format!(
                "{command} 输出不符合合同。预期 {:?}，实际 {:?}",
                short(expected, 170),
                short(&out.stdout, 170)
            ));
        }
        Ok(())
    }
    fn rejected(&self, command: &str, db: &Path, extra: &[&str]) -> Result<(), String> {
        let out = self.invoke(command, db, extra)?;
        if out.success {
            return Err(format!("{command} 对非法输入返回了成功退出码"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Check {
    pub name: &'static str,
    pub passed: bool,
    pub feedback: String,
}

fn storage(db: &Path, expected: &str) -> Result<(), String> {
    let actual =
        fs::read_to_string(db).map_err(|err| format!("程序未在指定数据路径写入记录：{err}"))?;
    if actual != expected {
        return Err(format!(
            "TSV 数据与合同不符：预期 {:?}，实际 {:?}",
            short(expected, 160),
            short(&actual, 160)
        ));
    }
    Ok(())
}
fn same(db: &Path, saved: &[u8]) -> Result<(), String> {
    let now = fs::read(db).map_err(|e| e.to_string())?;
    if now == saved {
        Ok(())
    } else {
        Err("失败命令改写了原有数据文件；应先完整验证再写入".into())
    }
}

fn check_empty(h: &Harness) -> Result<(), String> {
    let db = h.tmp.db("empty.tsv");
    h.ok("list", &db, &[], "")?;
    h.ok("stats", &db, &[], "total=0 open=0 done=0\n")?;
    if db.exists() {
        return Err("list/stats 不应为缺失的数据库创建文件".into());
    }
    Ok(())
}
// Checkpoint 1 deliberately requires no mutation commands.
fn check_read_only(h: &Harness) -> Result<(), String> {
    let db = h.tmp.db("只读 task list.tsv");
    let saved = "8\tDONE\t复盘\n3\tOPEN\t写自己的测试\n";
    fs::write(&db, saved).map_err(|e| e.to_string())?;
    h.ok("list", &db, &[], "3\tOPEN\t写自己的测试\n8\tDONE\t复盘\n")?;
    h.ok("stats", &db, &[], "total=2 open=1 done=1\n")?;
    same(&db, saved.as_bytes())
}

fn check_add_and_persist(h: &Harness) -> Result<(), String> {
    let db = h.tmp.db("persist.tsv");
    h.ok("add", &db, &["Review PR"], "added=1\n")?;
    h.ok("add", &db, &["Write docs"], "added=2\n")?;
    storage(&db, "1\tOPEN\tReview PR\n2\tOPEN\tWrite docs\n")?;
    h.ok(
        "list",
        &db,
        &[],
        "1\tOPEN\tReview PR\n2\tOPEN\tWrite docs\n",
    )?;
    h.ok("stats", &db, &[], "total=2 open=2 done=0\n")?;
    check_read_only(h)
}
fn check_done(h: &Harness) -> Result<(), String> {
    let db = h.tmp.db("done.tsv");
    fs::write(&db, "1\tOPEN\tAlpha\n2\tOPEN\tBeta\n").map_err(|e| e.to_string())?;
    h.ok("done", &db, &["1"], "done=1\n")?;
    h.ok("stats", &db, &[], "total=2 open=1 done=1\n")?;
    h.ok("done", &db, &["1"], "done=1\n")?;
    h.ok("list", &db, &[], "1\tDONE\tAlpha\n2\tOPEN\tBeta\n")?;
    storage(&db, "1\tDONE\tAlpha\n2\tOPEN\tBeta\n")
}
fn check_dynamic(h: &Harness) -> Result<(), String> {
    let db = h.tmp.db("sprint plan.tsv");
    for (id, title) in ["Plan", "Ship", "Measure", "Retrospect", "Follow-up"]
        .iter()
        .enumerate()
    {
        h.ok("add", &db, &[*title], &format!("added={}\n", id + 1))?;
    }
    for id in ["2", "4"] {
        h.ok("done", &db, &[id], &format!("done={id}\n"))?;
    }
    h.ok("stats", &db, &[], "total=5 open=3 done=2\n")?;
    h.ok(
        "list",
        &db,
        &[],
        "1\tOPEN\tPlan\n2\tDONE\tShip\n3\tOPEN\tMeasure\n4\tDONE\tRetrospect\n5\tOPEN\tFollow-up\n",
    )
}
fn check_unicode_and_trim(h: &Harness) -> Result<(), String> {
    let db = h.tmp.db("utf8.tsv");
    h.ok("add", &db, &["  修复登录 🐛  "], "added=1\n")?;
    h.ok("add", &db, &["une crème"], "added=2\n")?;
    storage(&db, "1\tOPEN\t修复登录 🐛\n2\tOPEN\tune crème\n")?;
    h.ok(
        "list",
        &db,
        &[],
        "1\tOPEN\t修复登录 🐛\n2\tOPEN\tune crème\n",
    )
}
fn check_invalid_input(h: &Harness) -> Result<(), String> {
    let db = h.tmp.db("invalid.tsv");
    fs::write(&db, "1\tOPEN\tKeep me\n").map_err(|e| e.to_string())?;
    h.ok("list", &db, &[], "1\tOPEN\tKeep me\n")?;
    let bytes = fs::read(&db).map_err(|e| e.to_string())?;
    for title in ["   ", "bad\ttitle", "bad\nline", "bad\rline"] {
        h.rejected("add", &db, &[title])?;
        same(&db, &bytes)?;
    }
    for id in ["0", "abc", "900"] {
        h.rejected("done", &db, &[id])?;
        same(&db, &bytes)?;
    }
    Ok(())
}
fn check_corrupt_database(h: &Harness) -> Result<(), String> {
    let valid = h.tmp.db("valid-before-corruption.tsv");
    fs::write(&valid, "1\tOPEN\tHealthy\n").map_err(|e| e.to_string())?;
    h.ok("list", &valid, &[], "1\tOPEN\tHealthy\n")?;
    for (name, malformed) in [
        ("bad-field.tsv", "not-a-tsv\n"),
        ("bad-status.tsv", "1\tMAYBE\tWrong\n"),
        ("duplicate.tsv", "1\tOPEN\tA\n1\tDONE\tB\n"),
    ] {
        let db = h.tmp.db(name);
        fs::write(&db, malformed).map_err(|e| e.to_string())?;
        let bytes = fs::read(&db).map_err(|e| e.to_string())?;
        h.rejected("list", &db, &[])?;
        h.rejected("add", &db, &["Never added"])?;
        same(&db, &bytes)?;
    }
    Ok(())
}
fn check_database_isolation(h: &Harness) -> Result<(), String> {
    let first = h.tmp.db("work.tsv");
    let second = h.tmp.db("personal.tsv");
    h.ok("add", &first, &["Work"], "added=1\n")?;
    h.ok("add", &first, &["Work 2"], "added=2\n")?;
    h.ok("add", &second, &["Personal"], "added=1\n")?;
    h.ok("stats", &first, &[], "total=2 open=2 done=0\n")?;
    h.ok("stats", &second, &[], "total=1 open=1 done=0\n")?;
    storage(&first, "1\tOPEN\tWork\n2\tOPEN\tWork 2\n")?;
    storage(&second, "1\tOPEN\tPersonal\n")
}
fn check_bad_cli(h: &Harness) -> Result<(), String> {
    let db = h.tmp.db("badcli.tsv");
    h.ok("stats", &db, &[], "total=0 open=0 done=0\n")?;
    for parts in [
        vec![OsString::from("add"), db.as_os_str().to_owned()],
        vec![OsString::from("unknown"), db.as_os_str().to_owned()],
        vec![OsString::from("stats")],
    ] {
        if h.run(&parts)?.success {
            return Err("缺少参数或未知命令不能以成功状态退出".into());
        }
    }
    if db.exists() {
        return Err("无效命令不应创建或修改数据库文件".into());
    }
    Ok(())
}

type Probe = fn(&Harness) -> Result<(), String>;

fn grade_with_executable(exe: PathBuf, tmp: TempWorkspace, stage: usize) -> Vec<Check> {
    let harness = Harness { exe, tmp };
    let groups: [(&str, Probe, &str); 9] = [
        (
            "01 缺失数据库与空列表",
            check_empty,
            "先实现 list/stats 的空表分支，注意不要创建文件",
        ),
        (
            "02 添加与跨进程持久化",
            check_add_and_persist,
            "检查新增 ID、TSV 存储和每次重新加载",
        ),
        (
            "03 完成状态与幂等",
            check_done,
            "完成指定 ID 并保留其他记录，重复 done 仍成功",
        ),
        (
            "04 多任务动态统计",
            check_dynamic,
            "不要写死 ID、任务数或统计结果",
        ),
        (
            "05 Unicode 与标题修剪",
            check_unicode_and_trim,
            "尝试 trim 后验证与保存；不要破坏 Unicode",
        ),
        (
            "06 非法标题与 ID 不改数据",
            check_invalid_input,
            "先检查输入，失败时返回非零退出码而非写入",
        ),
        (
            "07 损坏/重复记录拒绝修改",
            check_corrupt_database,
            "load 需要校验格式、状态、正整数及 ID 唯一性",
        ),
        (
            "08 不同数据文件相互隔离",
            check_database_isolation,
            "数据文件必须使用 CLI 指定的路径",
        ),
        (
            "09 错误命令契约",
            check_bad_cli,
            "严格检查命令名和参数数量，非法情况不可返回成功",
        ),
    ];
    let first: [(&str, Probe, &str); 2] = [
        (
            "01 缺失数据库与空列表",
            check_empty,
            "只实现 list/stats，不要求 add/done",
        ),
        (
            "02 读取已有记录与排序",
            check_read_only,
            "读取 TSV、按 ID 排序、统计，且不改写原文件",
        ),
    ];
    let selected: Vec<_> = if stage == 1 {
        first.iter().collect()
    } else {
        groups
            .iter()
            .enumerate()
            .filter(|(i, _)| stage == 3 || [0, 1, 2, 3, 4, 7].contains(i))
            .map(|(_, g)| g)
            .collect()
    };
    selected
        .into_iter()
        .map(|(name, probe, suggestion)| match probe(&harness) {
            Ok(()) => Check {
                name,
                passed: true,
                feedback: "通过：实际命令与存储内容符合合同".into(),
            },
            Err(err) => Check {
                name,
                passed: false,
                feedback: format!("{}。建议：{suggestion}", short(&err, 500)),
            },
        })
        .collect()
}

pub(crate) fn assess_stage(root: &Path, stage: usize) -> Vec<Check> {
    let tmp = match TempWorkspace::new() {
        Ok(tmp) => tmp,
        Err(err) => {
            return vec![Check {
                name: "环境检查",
                passed: false,
                feedback: err,
            }]
        }
    };
    match build(root, &tmp) {
        Ok(exe) => grade_with_executable(exe, tmp, stage),
        Err(err) => vec![Check {
            name: "Cargo 编译",
            passed: false,
            feedback: err,
        }],
    }
}

pub(crate) fn report(root: &Path, checks: &[Check]) -> Result<PathBuf, String> {
    let path = root.join("APP_ASSESSMENT.md");
    match fs::symlink_metadata(&path) {
        Ok(meta) => {
            if !meta.file_type().is_file() || meta.file_type().is_symlink() {
                return Err(format!("拒绝覆盖非普通报告文件 {}", path.display()));
            }
            let previous = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            if !previous.starts_with(STAMP) {
                return Err(
                    "APP_ASSESSMENT.md 已包含非工具生成内容，拒绝覆盖，请先自行备份".into(),
                );
            }
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => return Err(err.to_string()),
    }
    let passed = checks.iter().filter(|check| check.passed).count();
    let mut contents = format!("{STAMP}\n# TaskPocket · 自动验收记录\n\n通过 {passed}/{} 项。生成自本机实际 Cargo 编译和独立进程运行；并非人工学习成效评价。\n\n", checks.len());
    for item in checks {
        contents.push_str(&format!(
            "## {} {}\n\n{}\n\n",
            if item.passed { "PASS" } else { "FAIL" },
            item.name,
            item.feedback.replace('\n', " ")
        ));
    }
    contents.push_str("## 下一步\n\n未全通过时先修复第一个 FAIL，再重跑。全通过后填写 PROJECT_JOURNAL.md，独立使用 TaskPocket 管理真实任务，并自行尝试 cargo fmt / test / clippy。\n\n提示：这个检查器会**执行你编写的程序**，不是安全沙箱，切勿评测不可信项目。\n");
    fs::write(&path, contents).map_err(|err| format!("无法保存报告：{err}"))?;
    Ok(path)
}

/// 返回是否全部通过，失败的功能性结果作为 Ok(false) 并保留报告。
pub fn check(root: &Path) -> Result<bool, String> {
    crate::independent::check(root, "taskpocket", 3)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hints_are_staged() {
        assert!(hint(1).unwrap().contains("空库"));
        assert!(hint(3).unwrap().contains("保存"));
        assert!(hint(4).is_err());
    }
    #[test]
    fn generated_starter_is_unsolved() {
        assert!(STUDENT_SOURCE.contains("TODO"));
        assert!(!STUDENT_SOURCE.contains("fs::write"));
        assert!(CONTRACT.contains("<db>"));
        assert!(CARGO_MANIFEST.contains("[workspace]"));
    }
}
