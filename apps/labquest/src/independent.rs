//! Independent products: behavioral checkpoints, not prefilled business functions.
//! Read-only guidance never executes Cargo. Explicit check executes trusted local code.
use crate::capstone::Check;
use safesync::sha256::{hex, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const STAMP: &str = "<!-- RustAppLab Independent Progress v1 -->";
static NONCE: AtomicU64 = AtomicU64::new(0);
const MAX_REPORT: u64 = 256 * 1024;

fn text(path: &Path, limit: u64) -> Result<String, String> {
    let meta = fs::symlink_metadata(path).map_err(|e| format!("{}：{e}", path.display()))?;
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > limit {
        return Err(format!(
            "拒绝读取非普通、符号链接或过大的文件：{}",
            path.display()
        ));
    }
    fs::read_to_string(path).map_err(|e| e.to_string())
}
fn directory(path: &Path) -> Result<(), String> {
    let m = fs::symlink_metadata(path).map_err(|e| format!("{}：{e}", path.display()))?;
    if !m.is_dir() || m.file_type().is_symlink() {
        return Err("独立作品与报告目录必须是普通目录，不能是符号链接".into());
    }
    Ok(())
}
fn title(id: &str) -> Result<&'static str, String> {
    match id {
        "taskpocket" => Ok("TaskPocket"),
        "pagemint" => Ok("PageMint"),
        _ => Err("未知独立作品；仅支持 taskpocket、pagemint".into()),
    }
}
pub fn identify(root: &Path) -> Result<Option<&'static str>, String> {
    directory(root)?;
    if !root.join("CONTRACT.md").exists() || !root.join("PROJECT_JOURNAL.md").exists() {
        return Ok(None);
    }
    let contract = text(&root.join("CONTRACT.md"), 64 * 1024)?;
    let first = contract.lines().next().unwrap_or("");
    if first.starts_with("# TaskPocket ·") {
        Ok(Some("taskpocket"))
    } else if first.starts_with("# PageMint ·") {
        Ok(Some("pagemint"))
    } else {
        Err("未知独立作品合同，不自动按 TaskPocket 猜测".into())
    }
}
fn validate_stage(stage: usize) -> Result<(), String> {
    if (1..=3).contains(&stage) {
        Ok(())
    } else {
        Err("独立开发检查点只能是 1、2、3".into())
    }
}

/// Bounded teaching fingerprint, not a sandbox or reproducible-build attestation.
/// Includes manifest, source/resources, personal tests, build script and toolchain/config.
/// Cargo.lock is deliberately excluded: first build creates it for the std-only template.
fn fingerprint(root: &Path) -> Result<String, String> {
    fn add(
        root: &Path,
        path: &Path,
        map: &mut BTreeMap<String, Vec<u8>>,
        total: &mut u64,
        depth: u8,
    ) -> Result<(), String> {
        if depth > 16 || map.len() >= 128 {
            return Err("工程快照超过 128 个文件或 16 层目录限制".into());
        }
        let m = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
        if m.file_type().is_symlink() {
            return Err("源码/测试/配置中含符号链接，拒绝生成误导性快照".into());
        }
        if m.is_dir() {
            for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
                add(
                    root,
                    &entry.map_err(|e| e.to_string())?.path(),
                    map,
                    total,
                    depth + 1,
                )?;
            }
        } else if m.is_file() {
            *total += m.len();
            if m.len() > 1024 * 1024 || *total > 8 * 1024 * 1024 {
                return Err("工程快照超过单文件 1 MiB / 总计 8 MiB 限制".into());
            }
            let name = path
                .strip_prefix(root)
                .map_err(|e| e.to_string())?
                .to_str()
                .ok_or("源码路径须为 Unicode")?
                .replace('\\', "/");
            map.insert(name, fs::read(path).map_err(|e| e.to_string())?);
        } else {
            return Err("工程快照不接受特殊文件".into());
        }
        Ok(())
    }
    directory(&root.join("src"))?;
    let mut files = BTreeMap::new();
    let mut bytes = 0;
    for part in [
        "Cargo.toml",
        "src",
        "tests",
        "build.rs",
        ".cargo",
        "rust-toolchain",
        "rust-toolchain.toml",
    ] {
        let path = root.join(part);
        match fs::symlink_metadata(&path) {
            Ok(_) => add(root, &path, &mut files, &mut bytes, 0)?,
            Err(e)
                if e.kind() == std::io::ErrorKind::NotFound
                    && !["Cargo.toml", "src"].contains(&part) => {}
            Err(e) => return Err(e.to_string()),
        }
    }
    let mut digest = Sha256::new();
    for (name, data) in files {
        digest.update(&(name.len() as u64).to_le_bytes());
        digest.update(name.as_bytes());
        digest.update(&(data.len() as u64).to_le_bytes());
        digest.update(&data);
    }
    Ok(hex(&digest.finalize()))
}

#[derive(Debug)]
struct Record {
    stage: usize,
    passed: bool,
    hash: String,
}
fn parse_record(body: &str, id: &str) -> Result<Record, String> {
    if !body.starts_with(STAMP) {
        return Err("独立进度报告标记不正确，不能据此导航".into());
    }
    fn one<'a>(body: &'a str, key: &str) -> Result<&'a str, String> {
        let values: Vec<_> = body.lines().filter_map(|l| l.strip_prefix(key)).collect();
        if values.len() != 1 {
            return Err("报告字段缺失或重复，请重新真实检查".into());
        }
        Ok(values[0])
    }
    if one(body, "Product: ")? != id {
        return Err("进度报告不属于当前作品".into());
    }
    let stage = one(body, "Checkpoint: ")?
        .parse::<usize>()
        .map_err(|e| e.to_string())?;
    validate_stage(stage)?;
    let passed = match one(body, "Outcome: ")? {
        "PASS" => true,
        "FAIL" => false,
        _ => return Err("报告结果格式不正确".into()),
    };
    let hash = one(body, "Source-SHA256: ")?.to_owned();
    if hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("源码指纹格式不正确".into());
    }
    Ok(Record {
        stage,
        passed,
        hash,
    })
}
fn latest(root: &Path, id: &str) -> Result<Option<Record>, String> {
    let dir = root.join(".app-reports");
    match fs::symlink_metadata(&dir) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.to_string()),
        Ok(_) => directory(&dir)?,
    }
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with("CHECKPOINT-") && n.ends_with(".md"))
        {
            files.push(path);
        }
        if files.len() > 512 {
            return Err("进度记录超过 512 份，请先人工归档旧报告".into());
        }
    }
    files.sort();
    files
        .last()
        .map(|p| parse_record(&text(p, MAX_REPORT)?, id))
        .transpose()
}
pub fn suggested(root: &Path, id: &str, explicit: Option<usize>) -> Result<usize, String> {
    if let Some(n) = explicit {
        validate_stage(n)?;
        return Ok(n);
    }
    match latest(root, id)? {
        Some(r) if r.passed && r.hash == fingerprint(root)? => Ok((r.stage + 1).min(3)),
        Some(r) => Ok(r.stage),
        None => Ok(1),
    }
}
pub fn summary(root: &Path, id: &str) -> Result<String, String> {
    Ok(match latest(root, id)? {
        None => {
            "未检查；先设计数据，再从只读检查点 1 开始（旧 APP_ASSESSMENT 不自动兑换新进度）".into()
        }
        Some(r) if r.hash != fingerprint(root)? => format!(
            "源码/测试/配置有改动；报告已过期，重查累计范围 1–{}",
            r.stage
        ),
        Some(r) if !r.passed => format!("检查点 {} 未通过；读取最新失败报告后重试", r.stage),
        Some(r) if r.stage < 3 => format!(
            "检查点 {} 有通过记录；下一检查点 {}，尚非全作品通过",
            r.stage,
            r.stage + 1
        ),
        Some(_) => {
            "最终合同已有通过记录；仍需自己的测试、个人产物与使用说明，不是学习能力认证".into()
        }
    })
}
pub fn next(root: &Path, id: &str) -> Result<String, String> {
    let stage = suggested(root, id, None)?;
    Ok(format!("{} · 独立开发\n{}\n\n{}\n任务：learn lesson \"{}\" {stage}\n检查：learn check \"{}\" {stage}\n最终合同：app check {id} \"{}\"\n导航只读，不运行代码；查看历史不能代替当前真实检查。\n", title(id)?, summary(root,id)?, goal(id,stage)?, root.display(),root.display(),root.display()))
}
fn goal(id: &str, stage: usize) -> Result<&'static str, String> {
    title(id)?;
    validate_stage(stage)?;
    Ok(match (id, stage) {
        ("taskpocket",1) => "检查点 1：只读最小功能。缺失文件与已有 TSV 的 list/stats；此时不要求 add/done。",
        ("taskpocket",2) => "检查点 2：完整任务生命周期。add、跨进程保存、done 幂等、动态统计、中文标题和路径隔离。",
        ("taskpocket",_) => "检查点 3：全合同与失败不改数据。非法输入、坏记录、重复 ID、错误命令；累计回归所有功能。",
        ("pagemint",1) => "检查点 1：只读最小功能。stats 识别直接子级 .note、动态统计段落，不需要先实现 build。",
        ("pagemint",2) => "检查点 2：可见产物。完整 HTML、确定排序、动态输入、五字符转义，复用读取逻辑。",
        _ => "检查点 3：全合同与安全边界。非法输入不产出半成品，拒绝覆盖和输入目录内输出。",
    })
}
pub fn lesson(root: &Path, id: &str, requested: Option<usize>) -> Result<String, String> {
    let n = suggested(root, id, requested)?;
    let prompts = match (id,n) {
        ("taskpocket",1) => "先手绘一条 Task：ID、状态、标题各用什么类型？预测没有文件时 list/stats 的输出、文件是否应存在。先给已有两条记录写一项测试。",
        ("taskpocket",2) => "预测连续 add、done、再 add 后 ID 与统计如何变化；画出 解析→读取→业务修改→保存→输出 的顺序。用两个进程验证数据真的持久化。",
        ("taskpocket",_) => "复制一份练习数据，故意放入重复 ID；记录执行前后的完整字节。程序退出失败却把原数据截断，能否算正确？用自己的回归测试固定该行为。",
        ("pagemint",1) => "手算两份笔记的标题、非空段落数；预测子目录、.txt、CRLF 的影响。先设计 Note 的字段，再决定读取函数的输入和返回值。",
        ("pagemint",2) => "写一份含 <script> 与 & 的笔记，预测应该显示文本还是标签。先完成纯文本转义测试，再连接目录读取与网页构造，不先做 CSS。",
        _ => "复制笔记做无标题实验，预测是否应该留下 HTML；对已存在的输出比较执行前后字节。程序拒绝覆盖是设计目标，不是需要绕过的报错。",
    };
    Ok(format!("{}\n\n{prompts}\n\n编辑：自己的 src/main.rs，可自行拆为 lib.rs 或模块；没有规定必须填写某个函数。\n阅读 START.html / DEVELOPMENT_PLAN.md；旧工程无新文件时，读平台 INDEPENDENT_PATH.html 和原 CONTRACT.md。\n在学员目录 cargo test --offline：初始只有入口烟测，不能代表业务完成；tests/self_check.rs 的未完成个人测试必须自己补完、取消 ignore 并运行。\n在平台目录检查：cargo run --offline -p labquest -- learn check \"{}\" {n}\n检查点累计回归；记录是函数行为的证据，不是教学成效认证。\n", goal(id,n)?, root.display()))
}
pub fn hint(root: &Path, id: &str, level: usize, stage: Option<usize>) -> Result<String, String> {
    validate_stage(level)?;
    let n = suggested(root, id, stage)?;
    let hints = match (id, n) {
        ("taskpocket", 1) => [
            "把不存在文件视为空数据，与读取权限错误区分。",
            "只读路径：参数 → load → list/stats；Vec 保存多条任务，状态不要写死。",
            "自测：空库；两条 ID 8/3 逆序记录；一条 DONE。比较输出顺序与文件字节。",
        ],
        ("taskpocket", 2) => [
            "每次启动都重新加载；内存里正确不等于退出后还在。",
            "ID 来自当前最大值而不是记录数量。done 只更新目标；统计从当前集合计算。",
            "自测：五项任务完成第2/4项，再加一项；两个不同文件必须互不干扰。",
        ],
        ("taskpocket", _) => [
            "错误要早于任何写入，不能先清空文件再去验证。",
            "区分 CLI 输入、已有数据解析和保存三类错误，先收齐有效数据再提交修改。",
            "自测：空白标题、制表符、重复 ID、坏状态、未知命令；每次都比较原文件字节。",
        ],
        ("pagemint", 1) => [
            "先只读笔记并计算统计；标题不是正文段落。",
            "将读取结果存为一组 Note；lines/trim 处理 CRLF 与空行，不递归。",
            "自测：两个 note 共三段；新增一段后是四段；放一个 txt 和子目录不改变计数。",
        ],
        ("pagemint", 2) => [
            "数据变为 HTML 前先按文件名排序，再把文本转义。",
            "读取层被 stats/build 共用；转义是纯函数，文件名、标题、正文都需要。",
            "自测：标题和文件名有 &；正文有 <script>、单双引号；修改原笔记后换新输出名。",
        ],
        _ => [
            "先完整读取验证，再尝试创建不存在的新输出。",
            "create_new 可拒绝覆盖，不等于所有 I/O 失败都自动回滚。目录关系需独立检查。",
            "自测：空标题、无效UTF-8、已有输出、输出在输入目录；检查失败后的文件状态。",
        ],
    };
    Ok(format!("{} · 检查点 {n} · 提示 {level}\n{}\n把提示使用情况记在 PROJECT_JOURNAL.md；这里不给完整业务实现。\n",title(id)?,hints[level-1]))
}

fn ensure_reports(root: &Path) -> Result<PathBuf, String> {
    let dir = root.join(".app-reports");
    match fs::create_dir(&dir) {
        Ok(()) => (),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => (),
        Err(e) => return Err(e.to_string()),
    }
    directory(&dir)?;
    Ok(dir)
}
fn save_record(
    root: &Path,
    id: &str,
    stage: usize,
    hash: &str,
    checks: &[Check],
    passed: bool,
) -> Result<PathBuf, String> {
    let dir = ensure_reports(root)?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let target = dir.join(format!(
        "CHECKPOINT-{now:039}-{}-{}.md",
        std::process::id(),
        NONCE.fetch_add(1, Ordering::Relaxed)
    ));
    let mut body=format!("{STAMP}\n# {} · 独立开发检查点\n\nProduct: {id}\nCheckpoint: {stage}\nOutcome: {}\nSource-SHA256: {hash}\nEvaluator: {}\n\n本次通过 {}/{} 项。第1/2点只覆盖累计子集；第3点覆盖完整合同。不是学习能力证书，不运行学员自写测试。\n\n",title(id)?,if passed{"PASS"}else{"FAIL"},env!("CARGO_PKG_VERSION"),checks.iter().filter(|x|x.passed).count(),checks.len());
    for c in checks {
        body.push_str(&format!(
            "## {} {}\n\n{}\n\n",
            if c.passed { "PASS" } else { "FAIL" },
            c.name,
            c.feedback
        ));
    }
    body.push_str("源码指纹包括 Cargo.toml、src、tests、build.rs、.cargo 和工具链配置（存在时）。不覆盖 Cargo.lock、外部依赖、环境和数据集，不能作为供应链证明。旧版本无此记录时请重新检查。报告可编辑，仅用于学习导航；每次 check 始终实际运行。\n");
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&target)
        .map_err(|e| e.to_string())?;
    f.write_all(body.as_bytes()).map_err(|e| e.to_string())?;
    Ok(target)
}
/// Checkpoint 3 is the existing full contract, not an easier replacement.
pub fn check(root: &Path, id: &str, stage: usize) -> Result<bool, String> {
    title(id)?;
    validate_stage(stage)?;
    if identify(root)? != Some(id) {
        return Err("作品类型与合同不一致；拒绝执行错误的评测器".into());
    }
    let root = fs::canonicalize(root).map_err(|e| e.to_string())?;
    ensure_reports(&root)?;
    if stage == 3 && fs::symlink_metadata(root.join("APP_ASSESSMENT.md")).is_ok() {
        let previous = text(&root.join("APP_ASSESSMENT.md"), MAX_REPORT)?;
        let prefix = format!("<!-- RustAppLab {} Assessment v1 -->", title(id)?);
        if !previous.starts_with(&prefix) {
            return Err("APP_ASSESSMENT.md 是个人文件，拒绝覆盖".into());
        }
    }
    let before = fingerprint(&root)?;
    println!(
        "{}：真实编译并运行检查点 {stage} 的累计行为；只检查自己的可信代码（不是沙箱）。",
        title(id)?
    );
    let mut checks = if id == "taskpocket" {
        crate::capstone::assess_stage(&root, stage)
    } else {
        crate::transfer::assess_stage(&root, stage)
    };
    if fingerprint(&root)? != before {
        checks.push(Check {
            name: "源码快照一致性",
            passed: false,
            feedback: "检查期间源码或配置发生变化，请保存后重新检查，不能沿用本次结果".into(),
        });
    }
    let passed = !checks.is_empty() && checks.iter().all(|c| c.passed);
    for c in &checks {
        println!(
            "{} {}\n    {}",
            if c.passed { "[PASS]" } else { "[FAIL]" },
            c.name,
            c.feedback
        );
    }
    for c in &checks {
        if c.name == "Cargo 编译" && !c.passed {
            println!("{}", crate::learning::diagnose_text(&c.feedback));
        }
    }
    if stage == 3 {
        let p = if id == "taskpocket" {
            crate::capstone::report(&root, &checks)?
        } else {
            crate::transfer::report(&root, &checks)?
        };
        println!("完整合同报告：{}", p.display());
    }
    let p = save_record(&root, id, stage, &before, &checks, passed)?;
    println!("检查点报告：{}", p.display());
    if !passed {
        println!(
            "按最新 FAIL 修改；编译失败可用 learn diagnose <错误日志>，不要删除测试换取通过。"
        );
    }
    println!("{}", next(&root, id)?);
    Ok(passed)
}

pub(crate) fn write_materials(root: &Path, id: &str) -> Result<(), String> {
    title(id)?;
    let (guide, html, test) = if id == "taskpocket" {
        (
            include_str!("../independent/taskpocket.md"),
            include_str!("../independent/taskpocket.html"),
            include_str!("../independent/taskpocket_self_check.rs"),
        )
    } else {
        (
            include_str!("../independent/pagemint.md"),
            include_str!("../independent/pagemint.html"),
            include_str!("../independent/pagemint_self_check.rs"),
        )
    };
    fs::create_dir(root.join("tests")).map_err(|e| e.to_string())?;
    if id == "taskpocket" {
        fs::create_dir(root.join("samples")).map_err(|e| e.to_string())?;
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(root.join("samples/read-only.tsv"))
            .map_err(|e| e.to_string())?;
        f.write_all("8\tDONE\t复盘\n3\tOPEN\t写自己的测试\n".as_bytes())
            .map_err(|e| e.to_string())?;
    }
    for (name, contents) in [
        ("START.html", html),
        ("DEVELOPMENT_PLAN.md", guide),
        ("TEST_PLAN.md", include_str!("../independent/TEST_PLAN.md")),
        (
            "RELEASE_CHECKLIST.md",
            include_str!("../independent/RELEASE_CHECKLIST.md"),
        ),
        ("tests/self_check.rs", test),
    ] {
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(root.join(name))
            .map_err(|e| e.to_string())?;
        f.write_all(contents.as_bytes())
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn metadata_rejects_wrong_id_duplicate_fields_and_invalid_stage() {
        let b = format!(
            "{STAMP}\nProduct: taskpocket\nCheckpoint: 2\nOutcome: PASS\nSource-SHA256: {}\n",
            "0".repeat(64)
        );
        assert_eq!(parse_record(&b, "taskpocket").unwrap().stage, 2);
        assert!(parse_record(&b, "pagemint").is_err());
        assert!(parse_record(&(b.clone() + "Outcome: FAIL\n"), "taskpocket").is_err());
        assert!(parse_record(&b.replace("Checkpoint: 2", "Checkpoint: 9"), "taskpocket").is_err());
    }
    #[test]
    fn navigation_fingerprint_covers_manifest_tests_and_sources() {
        let tmp = crate::capstone::TempWorkspace::new_named("independent-hash").unwrap();
        let r = tmp.0.join("student");
        crate::capstone::init(&r).unwrap();
        let a = fingerprint(&r).unwrap();
        save_record(&r, "taskpocket", 1, &a, &[], true).unwrap();
        assert_eq!(suggested(&r, "taskpocket", None).unwrap(), 2);
        let p = r.join("Cargo.toml");
        fs::write(&p, fs::read_to_string(&p).unwrap() + "\n# changed\n").unwrap();
        assert!(summary(&r, "taskpocket").unwrap().contains("过期"));
        let b = fingerprint(&r).unwrap();
        assert_ne!(a, b);
        fs::write(r.join("tests/extra.rs"), "// my test").unwrap();
        assert_ne!(b, fingerprint(&r).unwrap());
    }
    #[test]
    fn guidance_never_changes_source_or_creates_reports() {
        let t = crate::capstone::TempWorkspace::new_named("independent-readonly").unwrap();
        let r = t.0.join("p");
        crate::transfer::init(&r).unwrap();
        let before = fingerprint(&r).unwrap();
        for n in 1..=3 {
            assert!(lesson(&r, "pagemint", Some(n)).unwrap().contains("编辑"));
            assert!(hint(&r, "pagemint", n, Some(n)).is_ok());
        }
        assert!(next(&r, "pagemint").unwrap().contains("未检查"));
        assert!(!r.join(".app-reports").exists());
        assert_eq!(before, fingerprint(&r).unwrap());
        assert!(suggested(&r, "pagemint", Some(0)).is_err());
    }
    #[cfg(unix)]
    #[test]
    fn symlink_report_directory_is_rejected_before_execution() {
        let t = crate::capstone::TempWorkspace::new_named("independent-link").unwrap();
        let r = t.0.join("p");
        crate::capstone::init(&r).unwrap();
        std::os::unix::fs::symlink(&t.0, r.join(".app-reports")).unwrap();
        assert!(check(&r, "taskpocket", 1).is_err());
        assert!(!r.join(".applab-target").exists());
    }
}
