//! 学习导航：只读任务卡与诊断；仅显式 check 子命令执行学员代码。
//!
//! route/next/lesson/hint/diagnose 不执行学员代码；check 复用既有有界评测器。
use crate::studio;
use safesync::sha256::{hex, Sha256};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

const MAX_ERROR_BYTES: u64 = 128 * 1024;
const MAX_REPORT_BYTES: u64 = 256 * 1024;

pub const ROUTE: &str = r#"Rust AppLab · 从零基础到独立软件

第一段（从一行代码起步）：FirstCard → UnitLab（函数、match、Result）。
第二段（文本与状态）：QuizDeck → TextScope → LedgerLite → BadgeForge → TermGarden。
第三段（并发、借用与网络）：BatchMeter → MiniKV → NetFetch → ServiceDesk。
第三段联合产物：个人工单档案，见 course/SYSTEMS_PATH.md。先完成单款，再组合；不用访问公网。
第四段（失败恢复与等待）：StoreDesk → AsyncBoard；36 张微任务卡已覆盖十二款作品。
第四段实验：事务回滚、提醒报告与唤醒边界，见 RELIABILITY_PATH.html / course/RELIABILITY_PATH.md。
第五段（独立开发）：TaskPocket、PageMint；三个累计行为检查点，不规定内部函数。
独立开发起点：INDEPENDENT_PATH.html；自写测试、个人产物和发布说明不能用平台PASS替代。
进阶 A（基础结课后）：TermGarden 工作区；见 ADVANCED_A.html。使用 advanced start 创建独立工程。
进阶 B1–B3 已提供离线可运行的多 crate Workspace，使用 advanced-b start 开始。第三方 Serde 与真实异步网络仍待在线验证。
LabQuest CASE 01 为综合调查练习。
MiniKV 等 Studio 工程有提供的外壳，不应当作从零独立开发考核。
不要把本路线当成必须按天完成的计划。先在当前作品中：
① 预测一个输入 → ② 修改一个 TODO → ③ 检查当步 → ④ 换自己的数据运行 → ⑤ 解释原因。
入门：START_HERE.html / START_WINDOWS.cmd
续学：CONTINUE_WINDOWS.cmd 或 cargo run --offline -p labquest -- studio catalog
查看所有项目：cargo run --offline -p labquest -- learn overview <练习父目录>
查看当前项目：cargo run --offline -p labquest -- learn next <学员项目目录>
当前任务卡：cargo run --offline -p labquest -- learn lesson <学员项目目录>
检查当前步：cargo run --offline -p labquest -- learn check <学员项目目录>
分层提示：cargo run --offline -p labquest -- learn hint <学员项目目录> 1
分析外部报错：cargo run --offline -p labquest -- learn diagnose <编译错误文本文件>
"#;

fn require_regular(path: &Path, max_bytes: u64) -> Result<String, String> {
    let m = fs::symlink_metadata(path).map_err(|e| format!("无法读取 {}：{e}", path.display()))?;
    if !m.is_file() || m.file_type().is_symlink() || m.len() > max_bytes {
        return Err(format!(
            "拒绝读取：必须是非符号链接的普通小文件（上限 {max_bytes} 字节）"
        ));
    }
    fs::read_to_string(path).map_err(|e| format!("文件不是有效 UTF-8 或读取失败：{e}"))
}

fn studio_id(root: &Path) -> Result<Option<String>, String> {
    let marker = root.join(".applab-studio");
    if !marker.exists() && fs::symlink_metadata(&marker).is_err() {
        return Ok(None);
    }
    let text = require_regular(&marker, 128)?;
    let mut lines = text.lines();
    match (lines.next(), lines.next(), lines.next()) {
        (Some("RustAppLab Studio v1"), Some(id), None) => {
            studio::product(id)?;
            Ok(Some(id.to_owned()))
        }
        _ => Err("学习工程标记格式有误；不会自动猜测或修改它".into()),
    }
}

fn source_hashes(root: &Path) -> Result<BTreeMap<String, String>, String> {
    fn walk(
        base: &Path,
        dir: &Path,
        files: &mut BTreeMap<String, String>,
        depth: usize,
    ) -> Result<(), String> {
        if depth > 16 || files.len() > 64 {
            return Err("源码文件数量或目录深度超出教学限制".into());
        }
        for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            let m = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
            if m.file_type().is_symlink() {
                return Err("源码路径包含符号链接，无法进行快照比较".into());
            }
            if m.is_dir() {
                walk(base, &path, files, depth + 1)?;
            } else if m.is_file() && path.extension().and_then(|x| x.to_str()) == Some("rs") {
                if m.len() > 1024 * 1024 {
                    return Err("源码文件大于 1 MiB".into());
                }
                let data = fs::read(&path).map_err(|e| e.to_string())?;
                let mut digest = Sha256::new();
                digest.update(&data);
                let rel = path
                    .strip_prefix(base)
                    .map_err(|e| e.to_string())?
                    .to_string_lossy()
                    .replace('\\', "/");
                files.insert(rel, hex(&digest.finalize()));
            }
        }
        Ok(())
    }
    let dir = root.join("src");
    let meta = fs::symlink_metadata(&dir).map_err(|e| e.to_string())?;
    if !meta.is_dir() || meta.file_type().is_symlink() {
        return Err("src 必须是实际存在的普通目录".into());
    }
    let mut files = BTreeMap::new();
    walk(root, &dir, &mut files, 0)?;
    if files.is_empty() || files.len() > 64 {
        return Err("未找到可比较的 .rs 源码文件或源码数量超限".into());
    }
    Ok(files)
}

/// 只用于教学引导。报告可由学员编辑，绝不据此发出 PASS 证明。
#[derive(Debug, Clone, Copy)]
struct ReportState {
    passed: u8,
    upto: u8,
    same_source: bool,
    compilation_failed: bool,
}
impl ReportState {
    fn step(self) -> usize {
        if self.compilation_failed {
            // Compilation failed before any milestone could run; retry its scope.
            usize::from(self.upto)
        } else if self.passed < self.upto {
            // Tests stop at the FIRST failure: 2/3 means step 3, not step 2.
            usize::from(self.passed + 1)
        } else {
            usize::from((self.upto + 1).min(3))
        }
    }
}
fn last_report(root: &Path) -> Result<Option<ReportState>, String> {
    let reports = root.join(".studio-reports");
    let meta = match fs::symlink_metadata(&reports) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.to_string()),
    };
    if !meta.is_dir() || meta.file_type().is_symlink() {
        return Err("报告目录不是普通目录".into());
    }
    let mut files: Vec<PathBuf> = fs::read_dir(reports)
        .map_err(|e| e.to_string())?
        .filter_map(|res| res.ok().map(|entry| entry.path()))
        .filter(|p| {
            p.file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|s| s.starts_with("ASSESSMENT-") && s.ends_with(".md"))
        })
        .collect();
    files.sort();
    // 如果目录中存在学员其它内容，本函数也不会执行或修改它们。
    let Some(latest) = files.pop() else {
        return Ok(None);
    };
    let report = require_regular(&latest, MAX_REPORT_BYTES)?;
    let Some((passed, upto)) = parse_score(&report) else {
        return Ok(None);
    };
    let embedded = report_hashes(&report);
    let current = source_hashes(root)?;
    Ok(Some(ReportState {
        passed,
        upto,
        same_source: embedded == current,
        compilation_failed: report.contains("## 编译 FAIL"),
    }))
}

fn parse_score(text: &str) -> Option<(u8, u8)> {
    let line = text
        .lines()
        .rev()
        .find(|line| line.trim_start().starts_with("本次累计："))?;
    let tail = line.trim_start().strip_prefix("本次累计：")?;
    let counts: String = tail
        .chars()
        .take_while(|ch| ch.is_ascii_digit() || *ch == '/')
        .collect();
    let (passed, upto) = counts.split_once('/')?;
    let p = passed.parse::<u8>().ok()?;
    let u = upto.parse::<u8>().ok()?;
    if p > u || u > 3 || u == 0 {
        return None;
    }
    Some((p, u))
}

fn report_hashes(text: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for line in text.lines() {
        // On Windows Studio prints src\lib.rs; normalize separators before comparing.
        let normalized = line.replace('\\', "/");
        if let Some(tail) = normalized.strip_prefix("- `src/") {
            if let Some((path, digest)) = tail.split_once("`：`") {
                if let Some(hash) = digest.strip_suffix('`') {
                    if hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit()) {
                        map.insert(format!("src/{path}"), hash.to_lowercase());
                    }
                }
            }
        }
    }
    map
}

pub fn next(root: &Path) -> Result<String, String> {
    let meta = fs::symlink_metadata(root).map_err(|_| {
        format!(
            "没有找到项目目录：{}。请先用 studio start 创建，或检查目录名",
            root.display()
        )
    })?;
    if !meta.is_dir() || meta.file_type().is_symlink() {
        return Err("学习目录不能是符号链接或文件".into());
    }
    let q = format!("\"{}\"", root.display());
    if let Some(id) = studio_id(root)? {
        let product = studio::product(&id)?;
        let state = last_report(root)?;
        let step = state.map(ReportState::step).unwrap_or(1);
        let status = match state {
            None => "没有可用的历史检查记录；从第 1 步开始。".into(),
            Some(r) if !r.same_source => format!("最近报告与当前源码不同；不要沿用旧的成绩。重新检查第 1–{step} 步，既验证当前修改，也回归之前的功能。"),
            Some(r) if r.compilation_failed => "最近检查没有编译通过：先修复第一条 error，再检查原来尝试的范围；不是把学习进度重置到第一步。".into(),
            Some(r) if r.passed < r.upto => format!("最近已通过前 {} 步，第一项失败在第 {step} 步；只修复当前失败，后面的步骤尚未检查。", r.passed),
            Some(r) if r.upto == 3 => "最近三步通过记录与当前 .rs 源码一致（仅供导航，不代替复验）。".into(),
            Some(r) => format!("最近第 {} 步通过，源码未变；建议继续下一步。", r.upto),
        };
        let objective = product.steps[step - 1];
        let lesson = format!("\"{}\"", root.join("START.html").display());
        let code = format!("\"{}\"", root.join("src/lib.rs").display());
        let complete = matches!(
            state,
            Some(ReportState {
                passed: 3,
                upto: 3,
                same_source: true,
                compilation_failed: false
            })
        );
        let mut out = format!("{} / {id}\n{status}\n\n", product.title);
        if complete {
            out.push_str("当前三步已有通过记录，不需要再修改 TODO(3)。下一任务是使用自己的输入、保存真实产物、写出一个新的测试。\n");
            out.push_str(&handoff(&id));
        } else {
            out.push_str(&format!("下一步：第 {step} 步 · {objective}\n打开 {lesson}；修改 {code} 对应 TODO({step}) 并保存。\n"));
        }
        out.push_str(&format!("检查：cargo run --offline -p labquest -- learn check {q} {step}\n任务卡：cargo run --offline -p labquest -- learn lesson {q} {step}\n分层提示：cargo run --offline -p labquest -- learn hint {q} 1 {step}\n"));
        out.push_str("注意：本命令不执行测试、不发合格证。只有重新运行 studio check 才能确认当前代码行为。\n");
        return Ok(out);
    }
    if root.join("跟着做.md").is_file() && root.join("src/main.rs").is_file() {
        let guide = format!("\"{}\"", root.join("跟着做.md").display());
        let code = format!("\"{}\"", root.join("src/main.rs").display());
        return Ok(format!("FirstCard · 第一张任务卡\n打开 {guide}，编辑 {code} 的 println! 行。\n检查：cargo run --offline -p labquest -- beginner check {q}\n提示：cargo run --offline -p labquest -- beginner hint 1\n修改后换两组不同的名字与任务真实运行，并保存任务卡文本。\n"));
    }
    if let Some(id) = crate::independent::identify(root)? {
        return crate::independent::next(root, id);
    }

    Err("当前目录不是已识别的 FirstCard/Studio/TaskPocket/PageMint 学习工程；请阅读 START_HERE.html，不会对目录做更改".into())
}

struct Tip {
    code: &'static str,
    meaning: &'static str,
    action: &'static str,
}
const TIPS: &[Tip] = &[
    Tip {code:"E0382",meaning:"值已经移动（move）后又被使用。",action:"定位首次移走所有权的调用；考虑传入 &T 借用、调整所有权流转，确实需要独立副本时再 clone。"},
    Tip {code:"E0502",meaning:"同一个值同时存在冲突的可变借用与不可变借用。",action:"缩短借用的作用范围，先读取所需信息，再单独修改数据；不要靠到处 clone 掩盖关系。"},
    Tip {code:"E0499",meaning:"同一个值同时存在多个活跃的可变借用。",action:"先结束第一处 &mut 借用，再创建下一处；检查循环体中是否长期持有引用。"},
    Tip {code:"E0308",meaning:"表达式的实际类型与要求类型不一致。",action:"核对函数返回类型与最后一个表达式，检查分号是否把返回值变成了 ()，Result 分支是否都返回一致类型。"},
    Tip {code:"E0597",meaning:"引用所指向的数据存活得不够久。",action:"检查引用来源是否是临时局部变量；必要时返回拥有数据的 String，而不是悬空的 &str。"},
    Tip {code:"E0277",meaning:"所需的 trait 约束没有满足。",action:"先看报错提示需要哪个 trait，检查参数/返回类型、derive 和泛型约束；不要直接强制转换。"},
    Tip {code:"E0599",meaning:"当前类型上不存在该方法，或相关 trait 不在作用域。",action:"确认变量实际类型与方法名，查看是否缺少 use 引入 trait 或错误地把 Option/Result 当成值。"},
    Tip {code:"E0425",meaning:"当前作用域中找不到指定值。",action:"核对拼写和变量定义的位置；检查是否只在某个 if、match 或函数内部定义。"},
    Tip {code:"E0432",meaning:"无法导入指定模块或名称。",action:"检查 Cargo.toml 的依赖、模块 pub 可见性、文件结构，以及 use 路径是否对应当前 crate。"},
    Tip {code:"E0004",meaning:"match 未覆盖所有可能分支。",action:"列出枚举全部变体，逐一覆盖；通配符 _ 应有符合需求的含义，不要用错误默认值敷衍。"},
];

fn without_ansi(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && chars.peek() == Some(&'[') {
            chars.next();
            for c in chars.by_ref() {
                if ('@'..='~').contains(&c) {
                    break;
                }
            }
        } else {
            out.push(ch);
        }
    }
    out
}

pub fn diagnose_text(text: &str) -> String {
    let clean = without_ansi(text);
    let lines: Vec<&str> = clean.lines().collect();
    let first = lines.iter().position(|line| {
        let t = line.trim();
        t.starts_with("error[") || t.starts_with("error:")
    });
    let Some(index) = first else {
        return "未找到 Rust error 诊断。正常的 Compiling 不代表编译失败；运行失败或测试断言请阅读报告中的 FAIL，不要先重装工具链。".into();
    };
    let line = lines[index].trim();
    let code = line
        .strip_prefix("error[")
        .and_then(|tail| tail.split_once(']').map(|x| x.0));
    let location = lines[index + 1..]
        .iter()
        .take_while(|l| !l.trim().starts_with("error"))
        .find_map(|line| line.trim().strip_prefix("-->"))
        .map(str::trim);
    let mut out = format!(
        "先处理第一条错误：{}\n",
        line.chars().take(220).collect::<String>()
    );
    if let Some(at) = location {
        out.push_str(&format!("位置（文件:行:列）：{at}\nStudio 在临时目录编译；请修改你自己工程中对应的 src 文件，而不是临时副本。\n"));
    }
    if let Some(tip) = code.and_then(|c| TIPS.iter().find(|t| t.code == c)) {
        out.push_str(&format!(
            "Rust 诊断：{}\n含义：{}\n下一步：{}\n",
            tip.code, tip.meaning, tip.action
        ));
    } else if [
        "delimiter",
        "expected",
        "unterminated",
        "unknown start of token",
    ]
    .iter()
    .any(|key| line.contains(key))
    {
        out.push_str("语法检查：先核对当前行及上一行的括号、双引号、分号和代码块。中文标点不能代替代码中的英文标点。不要为此重新安装 Rust。\n");
    } else if clean.contains("linker")
        || clean.contains("winsqlite3")
        || clean.contains("-lsqlite3")
    {
        out.push_str("链接或系统库错误：检查 MSVC/SDK 或 SQLite 原生库等工具链依赖；不是单凭这条错误就能判断 TODO 写错了。\n");
    } else {
        out.push_str("暂未识别该诊断；保留完整日志，核对第一条错误及其上下文，不要直接替换整个函数或把所有错误都当作安装失败。\n");
    }
    out.push_str("这只是诊断解释，不是检查结果。保存后重新检查；第一处修好后可能出现下一处错误。");
    out
}

fn read_diagnostics(path: &Path) -> Result<String, String> {
    let m = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if !m.is_file() || m.file_type().is_symlink() || m.len() > MAX_ERROR_BYTES {
        return Err("错误日志必须是非符号链接普通文件，且不超过 128 KiB".into());
    }
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_ERROR_BYTES {
        return Err("日志读取期间超出 128 KiB".into());
    }
    if bytes.starts_with(&[0xff, 0xfe]) || bytes.starts_with(&[0xfe, 0xff]) {
        let little = bytes[0] == 0xff;
        if (bytes.len() - 2) % 2 != 0 {
            return Err("UTF-16 日志的字节数不完整".into());
        }
        let words: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|pair| {
                if little {
                    u16::from_le_bytes([pair[0], pair[1]])
                } else {
                    u16::from_be_bytes([pair[0], pair[1]])
                }
            })
            .collect();
        String::from_utf16(&words).map_err(|_| "UTF-16 日志包含无效编码".into())
    } else {
        let bytes = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(&bytes);
        String::from_utf8(bytes.to_vec())
            .map_err(|_| "日志应为 UTF-8，或带 BOM 的 UTF-16（兼容 PowerShell 5.1）".into())
    }
}

fn learning_root(root: &Path) -> Result<(), String> {
    let m = fs::symlink_metadata(root)
        .map_err(|e| format!("无法打开学习目录 {}：{e}", root.display()))?;
    if !m.is_dir() || m.file_type().is_symlink() {
        return Err("学习目录必须是普通目录，不能是符号链接".into());
    }
    Ok(())
}
fn step_for(root: &Path, requested: Option<usize>) -> Result<usize, String> {
    if let Some(step) = requested {
        if !(1..=3).contains(&step) {
            return Err("步骤仅支持 1、2、3（检查时 all 等同于 3）".into());
        }
        return Ok(step);
    }
    Ok(last_report(root)?.map(ReportState::step).unwrap_or(1))
}
fn first_card(root: &Path) -> bool {
    root.join("跟着做.md").is_file() && root.join("src/main.rs").is_file()
}
fn handoff(id: &str) -> String {
    match id {
        "unitlab" => "迁移任务：用自己的数值完成一组合法和一组非法换算，记录结果。完成后进入 QuizDeck：studio start quizdeck <新目录>；从数值输入过渡到文本输入。\n".into(),
        "quizdeck" => "迁移任务：创建自己的三道题及答案，运行一次含错题的 check，保存复习清单。完成后进入 TextScope：studio start textscope <新目录>，观察同一批文本。\n".into(),
        "textscope" => "迁移任务：对自己的题库或笔记运行 stats/top/find，写一个新的边界测试。下一款是 LedgerLite：studio start ledgerlite <新目录>，学习持久化记录。\n".into(),
        "ledgerlite" => "迁移任务：保存个人账目，退出后重读。下一款 BadgeForge：studio start badgeforge <新目录>，把成果做成作品卡。\n".into(),
        "badgeforge" => "迁移任务：生成一张自己的 SVG 并打开。下一款 TermGarden：studio start termgarden <新目录>，学习事件与持续状态。\n".into(),
        "termgarden" => "迁移任务：退出并重放自己的任务脚本。下一款 BatchMeter：studio start batchmeter <新目录>，先单文件再多线程；入口 course/SYSTEMS_PATH.md。\n".into(),
        "batchmeter" => "迁移任务：用 1 与 4 个线程统计同一组文件并比较输出。下一款 MiniKV：studio start minikv <新目录>，建立自己的文件索引；见 course/SYSTEMS_PATH.md。\n".into(),
        "minikv" => "迁移任务：保存并跨进程读取 snapshot/ 索引，检查前缀筛选。下一款 NetFetch：studio start netfetch <新目录>；读写日志不等于事务或断电恢复保证。\n".into(),
        "netfetch" => "迁移任务：保存一份本机服务 JSON，验证同名重下不会覆盖。下一款 ServiceDesk：studio start servicedesk <新目录>；当前下载器同步且仅支持本机 HTTP。\n".into(),
        "servicedesk" => "迁移任务：完成 course/SYSTEMS_PATH.md 的个人工单档案，并重启服务验证保存。之后进入 StoreDesk：studio start storedesk <新目录>，学习事务；再进入 AsyncBoard，它不是 Tokio 网络运行时。\n".into(),
        "storedesk" => "迁移任务：按 RELIABILITY_PATH.html 故意让批次第二项失败，检查库存和流水一起回滚。下一款 AsyncBoard：studio start asyncboard <新目录>；不把两个程序当作一个跨应用事务。\n".into(),
        "asyncboard" => "迁移任务：用自己的提醒清单生成报告，解释 wait_ms 与实测耗时的区别。完成 RELIABILITY_PATH.html 复盘后进入独立作品：app init taskpocket <新目录>，按 CONTRACT.md 独立实现，而不是继续填空。\n".into(),
        _ => "下一步：用自己的数据真实运行 CLI，保存输出，填写 LEARNING_LOG.md，再按 learn route 选择后续作品。\n".into(),
    }
}
/// Included in newly generated learner projects; no absolute build-machine paths.
pub(crate) fn printable_tasks(id: &str) -> Option<String> {
    crate::learning_cards::get(id, 1)?;
    let mut text = format!("# {id} · 逐步任务卡\n\n每次只做一步。命令在本学员工程（含 Cargo.toml）里运行；不需要打开平台内部源码。\n先预测 → 改当前函数 → 保存 → 检查 → 用自己的数据试验 → 用自己的话解释。\n\n");
    for step in 1..=3 {
        let card = crate::learning_cards::get(id, step)?;
        text.push_str(&format!("## 第 {step} 步：{}\n\n编辑 `src/lib.rs` 的 `{}` / `TODO({step})`。\n\n本次概念：{}\n\n先预测：{}\n\n保存后检查：\n\n```text\ncargo test --offline --test milestones step{step}_\n```\n\n这条本地命令只运行当前步。三步完成后运行 `cargo test --offline` 回归全部；完整平台中的 `learn check` 会从第一步累计验证。\n\n",card.goal,card.function,card.concept,card.prediction));
        for (n, hint) in card.hints.iter().enumerate() {
            text.push_str(&format!(
                "<details><summary>提示 {}（先尝试，再展开）</summary>\n\n{hint}\n\n</details>\n\n",
                n + 1
            ));
        }
        text.push_str(&format!(
            "通过后试自己的数据：{}\n\n复盘：{}\n\n",
            card.experiment, card.reflection
        ));
    }
    text.push_str("## 完成后\n\n");
    text.push_str(&handoff(id));
    text.push_str("\n记录输入、预期、实际输出、用过的提示和仍然不理解的地方。功能测试通过不等于无需帮助就掌握了这些概念。\n");
    Some(text)
}

/// Standalone, script-free task cards generated from the same source as CLI hints.
/// No progress is stored or asserted by this page.
pub fn printable_tasks_html(id: &str) -> Option<String> {
    let first = crate::learning_cards::get(id, 1)?;
    let product = studio::product(id).ok()?;
    fn html(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }
    let mut out = String::from(
        r#"<!doctype html><html lang="zh-CN"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">"#,
    );
    out.push_str(&format!(
        "<title>{} · 分步任务</title>",
        html(product.title)
    ));
    out.push_str(r#"<style>
*{box-sizing:border-box}body{margin:0;background:#f4f7fb;color:#23364d;font:16px/1.8 system-ui,"Microsoft YaHei",sans-serif}header{background:#17324d;color:white}header>div,main{max-width:920px;margin:auto;padding:28px 24px}h1{font-size:clamp(25px,4vw,36px);line-height:1.4;margin:10px 0}h2{line-height:1.5;font-size:23px}h3{font-size:18px}a{color:#15659a}header a{color:#c1ecf4}nav{display:flex;gap:18px;flex-wrap:wrap}section{background:white;border:1px solid #d9e3ed;border-radius:14px;padding:22px 26px;margin-bottom:22px}p,li{overflow-wrap:anywhere}code{font-family:Consolas,monospace;background:#edf3f7;padding:2px 4px;border-radius:4px;overflow-wrap:anywhere}pre{padding:15px;background:#17324d;color:white;border-radius:8px;white-space:pre-wrap;overflow-wrap:anywhere}pre code{background:none;padding:0}details{border-top:1px solid #d9e3ed;padding:12px 0}summary{cursor:pointer;font-weight:600}.predict{background:#e8f5ef;padding:14px;border-radius:8px}.label{font-size:13px;letter-spacing:.1em;color:#446e8b}footer{padding:10px 4px 30px;color:#546a80}@media(max-width:550px){header>div,main{padding:20px 14px}section{padding:18px 16px}h2{font-size:21px}}</style></head><body><header><div><small>RUST APPLAB / ONE FUNCTION AT A TIME</small>"#);
    out.push_str(&format!(
        "<h1>{}</h1><p>现在先做第 1 步：{}。读目标、写下预测、修改自己的函数，再运行测试。</p>",
        html(product.title),
        html(first.goal)
    ));
    out.push_str("<nav aria-label=\"里程碑导航\"><a href=\"#step-1\">第 1 步</a><a href=\"#step-2\">第 2 步</a><a href=\"#step-3\">第 3 步</a><a href=\"START.html\">完整讲义</a></nav></div></header><main>");
    for step in 1..=3 {
        let c = crate::learning_cards::get(id, step)?;
        out.push_str(&format!("<section id=\"step-{step}\"><span class=\"label\">MILESTONE 0{step}</span><h2>第 {step} 步 · {}</h2><p>只编辑自己的 <code>src/lib.rs</code>：<code>{}</code> / <code>TODO({step})</code>。</p><p>本次概念：{}</p><p class=\"predict\"><strong>先预测：</strong>{}</p><h3>保存后运行</h3><pre><code>cargo test --offline --test milestones step{step}_</code></pre><p>在本学员目录执行，只验证当前步。完整平台的 learn check 会累计回归前面步骤；语法错误仍需先修复。</p><h3>遇到困难再打开提示</h3>",html(c.goal),html(c.function),html(c.concept),html(c.prediction)));
        for (index, hint) in c.hints.iter().enumerate() {
            out.push_str(&format!(
                "<details><summary>第 {} 层提示</summary><p>{}</p></details>",
                index + 1,
                html(hint)
            ));
        }
        out.push_str(&format!(
            "<h3>用自己的数据</h3><p>{}</p><h3>自己解释</h3><p>{}</p></section>",
            html(c.experiment),
            html(c.reflection)
        ));
    }
    out.push_str(&format!("<section><h2>三步完成之后</h2><pre><code>cargo test --offline</code></pre><p>{}</p><p>学习路线中提到的 course/ 路径位于完整平台或学员包根目录。记录真实输入、产物和用过的提示：<a href=\"LEARNING_LOG.md\">LEARNING_LOG.md</a>。</p></section><footer>页面不保存成绩，不执行代码、不请求网络。展开提示不代表通关；真实检查请使用 Cargo 或平台的 learn check。</footer></main></body></html>",html(&handoff(id))));
    Some(out)
}

pub fn lesson(root: &Path, requested: Option<usize>) -> Result<String, String> {
    learning_root(root)?;
    if let Some(id) = studio_id(root)? {
        let step = step_for(root, requested)?;
        let product = studio::product(&id)?;
        if let Some(card) = crate::learning_cards::get(&id, step) {
            let source = require_regular(&root.join("src/lib.rs"), 1024 * 1024)?;
            let line = source
                .lines()
                .position(|line| line.contains(&format!("fn {}", card.function)))
                .map(|n| format!("第 {} 行附近", n + 1))
                .unwrap_or_else(|| "在编辑器中按函数名搜索".into());
            Ok(format!("{} · 第 {step} 步\n目标：{}\n只编辑：{}（{line}），函数 {}，TODO({step})。\n本次概念：{}\n\n先预测：{}\n然后修改并保存。检查：cargo run --offline -p labquest -- learn check \"{}\" {step}\n通过后换自己的数据：{}\n复盘：{}\n提示按需递进：learn hint \"{}\" 1 {step} → 2 → 3；先尝试再查看。\n",product.title,card.goal,root.join("src/lib.rs").display(),card.function,card.concept,card.prediction,root.display(),card.experiment,card.reflection,root.display()))
        } else {
            Ok(format!("{} · 第 {step} 步：{}\n打开 {}/LESSON.md，编辑自己的 src/lib.rs 的 TODO({step})。本作品暂未配置额外微任务卡，使用原有分步讲义与 HINTS.md。\n检查：learn check \"{}\" {step}\n",product.title,product.steps[step-1],root.display(),root.display()))
        }
    } else if first_card(root) {
        if requested.is_some_and(|s| s != 1) {
            return Err("FirstCard 是单步热身，请省略步骤或使用 1".into());
        }
        Ok(format!("FirstCard · 第一张任务卡\n只编辑 {}/src/main.rs 中的 println! 行。\n先预测：把姓名和任务都换掉，程序应该改变哪两处输出？\nCtrl+S 保存后执行 learn check \"{}\"。\n通过后在学员目录用两组自己的姓名与任务运行，保存 TXT；再创建 UnitLab，不要从零编写平台源码。\n",root.display(),root.display()))
    } else if let Some(id) = crate::independent::identify(root)? {
        crate::independent::lesson(root, id, requested)
    } else {
        Err("未知工程，任务卡支持 FirstCard / Studio / 两款独立作品".into())
    }
}
pub fn hint(root: &Path, level: usize, requested: Option<usize>) -> Result<String, String> {
    learning_root(root)?;
    if !(1..=3).contains(&level) {
        return Err("提示级别只支持 1（概念）、2（思路）、3（自测）".into());
    }
    if let Some(id) = studio_id(root)? {
        let step = step_for(root, requested)?;
        if let Some(card) = crate::learning_cards::get(&id, step) {
            Ok(format!("{id} · 第 {step} 步 · 第 {level} 层提示\n{}\n请在 LEARNING_LOG.md 如实记录使用了哪一层提示；不会替你写入源码。\n",card.hints[level-1]))
        } else {
            Ok(format!("此作品没有新版三级微提示。请打开 {}/HINTS.md 或运行 studio hint {id}；不会把通用提示冒充当前步骤的完整解答。\n",root.display()))
        }
    } else if first_card(root) {
        if requested.is_some_and(|s| s != 1) {
            return Err("FirstCard 只支持第 1 步".into());
        }
        Ok(crate::beginner::hint(level)?.into())
    } else if let Some(id) = crate::independent::identify(root)? {
        crate::independent::hint(root, id, level, requested)
    } else {
        Err("未知工程，请用 learn next 检查路径".into())
    }
}
/// Explicitly requested execution; history only selects a scope, never supplies PASS.
pub fn check_current(root: &Path, requested: Option<usize>) -> Result<bool, String> {
    learning_root(root)?;
    if studio_id(root)?.is_some() {
        let step = step_for(root, requested)?;
        println!("将真实编译并检查第 1–{step} 步；不会依赖旧报告判定通过。只运行自己的可信代码。");
        let passed = studio::check(root, step)?;
        if passed {
            print!("{}", next(root)?);
        } else {
            let failed_step = step_for(root, None)?;
            println!(
                "分层提示：learn hint \"{}\" 1 {failed_step}（后两个数字依次为提示级别、步骤）",
                root.display()
            );
        }
        Ok(passed)
    } else if first_card(root) {
        if requested.is_some_and(|s| s != 1) {
            return Err("FirstCard 只支持第 1 步".into());
        }
        let passed = crate::beginner::check(root)?;
        if passed {
            println!("换两组自己的输入，保存实际任务卡。然后在平台根目录运行 studio start unitlab my-unitlab，从函数返回值继续学习。");
        }
        Ok(passed)
    } else if let Some(id) = crate::independent::identify(root)? {
        let stage = crate::independent::suggested(root, id, requested)?;
        crate::independent::check(root, id, stage)
    } else {
        Err("未知工程，请用 learn next 检查路径".into())
    }
}
pub(crate) fn failure_feedback(id: &str, step: usize, log: &str) -> String {
    let mut result = format!(
        "学习反馈：第 {step} 步已经进入功能检查，但尚未满足合同。这不等于 Rust 安装失败。\n"
    );
    if let Some(card) = crate::learning_cards::get(id, step) {
        result.push_str(&format!(
            "先回到 src/lib.rs 的 {}。{}\n",
            card.function, card.hints[0]
        ));
    }
    if log.contains("left:") && log.contains("right:") {
        result.push_str("断言里 left/right 是两个表达式的值；不能一概当作实际/预期。查看对应断言或当前合同后，先用失败输入手工算一遍。\n");
    }
    result.push_str("只修复当前失败，保存后重试相同范围。原始失败输出和源码指纹保存在本次报告中。");
    result
}

/// 多作品学习总览：只读已知的 Studio 项目、FirstCard 和独立练习工程。
/// 不执行学员代码、不创建任何报告；磁盘报告均被视为学员可编辑的历史线索。
pub fn overview(parent: &Path) -> Result<String, String> {
    learning_root(parent)?;
    fn children(dir: &Path) -> Result<Vec<PathBuf>, String> {
        let mut found = Vec::new();
        // bounded directory scan: only immediate children, never follow symlinks.
        for item in fs::read_dir(dir).map_err(|e| e.to_string())? {
            let item = item.map_err(|e| e.to_string())?;
            if found.len() >= 256 {
                return Err("目录条目超过 256；请对更小的练习目录使用 learn overview".into());
            }
            let path = item.path();
            let meta = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
            if meta.is_dir() && !meta.file_type().is_symlink() {
                found.push(path);
            }
        }
        found.sort();
        Ok(found)
    }
    let mut candidates = children(parent)?;
    // The learner kit nests its twelve Studio projects in Studio/<product>.
    // Keep the traversal shape explicit and bounded; never recurse into src/target.
    let studio_dir = parent.join("Studio");
    if studio_dir.is_dir()
        && !fs::symlink_metadata(&studio_dir)
            .map_err(|e| e.to_string())?
            .file_type()
            .is_symlink()
    {
        candidates.extend(children(&studio_dir)?);
    }
    let advanced_dir = parent.join("Advanced");
    if advanced_dir.is_dir()
        && !fs::symlink_metadata(&advanced_dir)
            .map_err(|e| e.to_string())?
            .file_type()
            .is_symlink()
    {
        candidates.extend(children(&advanced_dir)?);
    }
    // Allow pointing the command at a single student project too.
    candidates.push(parent.to_path_buf());
    let mut rows = Vec::new();
    let mut recorded_complete = 0usize;
    let mut stale = 0usize;
    let mut needs_action = 0usize;
    for dir in candidates {
        if rows.len() >= 48 {
            return Err("找到超过 48 个练习工程；请缩小总览目录范围".into());
        }
        // Prefer exact marker over titles inferred from filenames.
        let id = match studio_id(&dir) {
            Ok(Some(id)) => Some(id),
            Ok(None) => None,
            Err(e) => {
                if dir.join(".applab-studio").exists() {
                    rows.push((dir, "标记异常".to_string(), format!("标记不可解析：{e}")));
                }
                continue;
            }
        };
        let (name, status) = if let Some(id) = id {
            let label = format!("Studio/{id}");
            let status = match last_report(&dir) {
                Ok(None) => {
                    needs_action += 1;
                    "未检查；从第 1 步开始".into()
                }
                Ok(Some(r)) if !r.same_source => {
                    stale += 1;
                    needs_action += 1;
                    "源码有改动；旧报告已过期，须重新检查".into()
                }
                Ok(Some(r)) if r.compilation_failed => {
                    needs_action += 1;
                    format!("上次编译失败；回到第 {} 步", r.step())
                }
                Ok(Some(r)) if r.passed < r.upto => {
                    needs_action += 1;
                    format!("通过 {}/{}，需修复第 {} 步", r.passed, r.upto, r.step())
                }
                Ok(Some(r)) if r.upto == 3 => {
                    recorded_complete += 1;
                    "3/3 历史通过记录；还需个人数据作品".into()
                }
                Ok(Some(r)) => {
                    needs_action += 1;
                    format!("已记录 {}/{}；下一步 {}", r.passed, r.upto, r.step())
                }
                Err(e) => {
                    needs_action += 1;
                    format!("报告读取异常：{e}")
                }
            };
            (label, status)
        } else if crate::advanced_b::is_project(&dir) {
            (
                "Advanced/TaskPocketPro".into(),
                crate::advanced_b::next(&dir).unwrap_or_else(|e| format!("进阶 B 记录异常：{e}")),
            )
        } else if crate::advanced::is_project(&dir) {
            (
                "Advanced/TermGarden".into(),
                crate::advanced::next(&dir).unwrap_or_else(|e| format!("进阶记录异常：{e}")),
            )
        } else if first_card(&dir) {
            (
                "FirstCard".into(),
                "热身工程；请用 beginner check 检查".into(),
            )
        } else if dir.join("CONTRACT.md").is_file() && dir.join("PROJECT_JOURNAL.md").is_file() {
            // Unknown products are NOT guessed as one of the two established contracts.
            match require_regular(&dir.join("CONTRACT.md"), 64 * 1024) {
                Ok(contract)
                    if contract
                        .lines()
                        .next()
                        .is_some_and(|s| s.contains("PageMint")) =>
                {
                    (
                        "PageMint".into(),
                        crate::independent::summary(&dir, "pagemint")
                            .unwrap_or_else(|e| format!("报告异常：{e}")),
                    )
                }
                Ok(contract)
                    if contract
                        .lines()
                        .next()
                        .is_some_and(|s| s.contains("TaskPocket")) =>
                {
                    (
                        "TaskPocket".into(),
                        crate::independent::summary(&dir, "taskpocket")
                            .unwrap_or_else(|e| format!("报告异常：{e}")),
                    )
                }
                _ => continue,
            }
        } else {
            continue;
        };
        rows.push((dir, name, status));
    }
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    let mut out = format!(
        "Rust AppLab · 学习进度总览（只读）\n目录：{}\n已识别工程：{}；Studio 三步历史通过：{}；报告过期：{}；待行动：{}\n\n",
        parent.display(), rows.len(), recorded_complete, stale, needs_action
    );
    if rows.is_empty() {
        out.push_str(
            "未找到练习工程。请指定包含 my-unitlab 等文件夹的目录，或学员包根目录（含 Studio）。\n",
        );
    }
    for (dir, name, status) in &rows {
        out.push_str(&format!("- {name} [{}]\n  {status}\n  下步：cargo run --offline -p labquest -- learn next \"{}\"\n",dir.display(),dir.display()));
    }
    out.push_str("\n重要：以上是可编辑的本地记录与当前源码快照比对，不是防作弊成绩或教师认证。不读取用户隐私数据，也不会运行学员代码。\n真正验收请使用 learn check / app check，并用个人数据生成作品。\n");
    Ok(out)
}

pub fn run(args: &[OsString]) -> Result<(), String> {
    if args.len() >= 2 && crate::advanced_b::is_project(Path::new(&args[1])) {
        let root = Path::new(&args[1]);
        return match args[0].to_str() {
            Some("next") if args.len() == 2 => {
                println!("{}", crate::advanced_b::next(root)?);
                Ok(())
            }
            Some("lesson") if args.len() >= 2 && args.len() <= 3 => {
                println!(
                    "{}",
                    include_str!("../../../projects/advanced-b/ADVANCED_B.md")
                );
                Ok(())
            }
            Some("check") if args.len() == 3 => {
                let n = args[2]
                    .to_str()
                    .and_then(|s| s.parse::<usize>().ok())
                    .ok_or("步骤应为1..3")?;
                if crate::advanced_b::check(root, n)? {
                    Ok(())
                } else {
                    Err("进阶 B 检查未通过".into())
                }
            }
            _ => Err("进阶 B 学习导航：learn next/lesson/check <工程> [步骤]".into()),
        };
    }
    if args.len() >= 2 && crate::advanced::is_project(Path::new(&args[1])) {
        return crate::advanced::learn_run(args);
    }
    let command = args.first().and_then(|s| s.to_str()).unwrap_or("route");
    fn parse_step(arg: Option<&OsString>) -> Result<Option<usize>, String> {
        arg.map(|s| match s.to_str() {
            Some("1") => Ok(1),
            Some("2") => Ok(2),
            Some("3" | "all") => Ok(3),
            _ => Err("步骤应为 1、2、3 或 all".into()),
        })
        .transpose()
    }
    match command {
        "route" if args.len() <= 1 => { print!("{ROUTE}"); Ok(()) }
        "overview" if args.len() == 2 => { print!("{}", overview(Path::new(&args[1]))?); Ok(()) }
        "next" if args.len() == 2 => { print!("{}", next(Path::new(&args[1]))?); Ok(()) }
        "lesson" if (2..=3).contains(&args.len()) => { print!("{}",lesson(Path::new(&args[1]),parse_step(args.get(2))?)?); Ok(()) }
        "hint" if (3..=4).contains(&args.len()) => {
            let level=args[2].to_str().and_then(|x|x.parse().ok()).ok_or("提示级别应为 1、2、3")?;
            print!("{}",hint(Path::new(&args[1]),level,parse_step(args.get(3))?)?); Ok(())
        }
        "check" if (2..=3).contains(&args.len()) => {
            if check_current(Path::new(&args[1]),parse_step(args.get(2))?)? { Ok(()) }
            else { Err("当前练习尚未通过；阅读本次反馈、保存修改后，重新运行 learn check".into()) }
        }
        "diagnose" if args.len()==2 => { println!("{}",diagnose_text(&read_diagnostics(Path::new(&args[1]))?)); Ok(()) }
        _ => Err("用法：learn route | overview <练习父目录> | next <目录> | lesson <目录> [步骤] | hint <目录> <级别1..3> [步骤] | check <目录> [步骤|all] | diagnose <报错文件>".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn diagnostic_distinguishes_errors() {
        let moved = diagnose_text("error[E0382]: borrow of moved value");
        assert!(moved.contains("所有权"));
        let mismatch = diagnose_text("Compiling something\nerror[E0308]: mismatched types");
        assert!(mismatch.contains("分号"));
        assert!(diagnose_text("Compiling xx").contains("不代表编译失败"));
        assert!(diagnose_text("error: linker failed").contains("工具链"));
        assert!(diagnose_text("error[E9999]: unknown").contains("暂未识别"));
    }
    #[test]
    fn score_parser_only_accepts_valid_milestones() {
        assert_eq!(parse_score("本次累计：2/2。这是源码快照"), Some((2, 2)));
        assert_eq!(parse_score("本次累计：4/4"), None);
        assert_eq!(parse_score("本次累计：x/3"), None);
    }
    #[test]
    fn next_ignores_stale_and_fake_reports() {
        let temp = crate::capstone::TempWorkspace::new_named("learning-next").unwrap();
        let project = temp.0.join("course");
        studio::init("unitlab", &project).unwrap();
        let initial = next(&project).unwrap();
        assert!(initial.contains("第 1 步"));
        fs::create_dir(project.join(".studio-reports")).unwrap();
        fs::write(project.join(".studio-reports/ASSESSMENT-123.md"),"本次累计：3/3。\n- `src/lib.rs`：`0000000000000000000000000000000000000000000000000000000000000000`\n").unwrap();
        assert!(next(&project).unwrap().contains("不要沿用旧的成绩"));
        fs::write(project.join("src/lib.rs"), "// modified\n").unwrap();
        assert!(next(&project).unwrap().contains("重新检查"));
    }
    #[test]
    fn next_matches_snapshot_but_does_not_certify() {
        let temp = crate::capstone::TempWorkspace::new_named("learning-matching").unwrap();
        let root = temp.0.join("project");
        studio::init("unitlab", &root).unwrap();
        let files = source_hashes(&root).unwrap();
        fs::create_dir(root.join(".studio-reports")).unwrap();
        let mut report = String::from("# 学习记录\n\n");
        for (file, hash) in files {
            report.push_str(&format!("- `{file}`：`{hash}`\n"));
        }
        report.push_str("本次累计：1/1。这是源码快照的功能证据\n");
        fs::write(root.join(".studio-reports/ASSESSMENT-12.md"), report).unwrap();
        let next = next(&root).unwrap();
        assert!(next.contains("第 2 步"));
        assert!(next.contains("不发合格证"));
    }
    #[test]
    fn windows_report_path_separators_are_normalized() {
        let raw =
            "- `src\\lib.rs`：`0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef`";
        let parsed = report_hashes(raw);
        assert!(parsed.contains_key("src/lib.rs"));
    }
    #[test]
    fn distinguishes_both_independent_products() {
        let temp = crate::capstone::TempWorkspace::new_named("learning-independent").unwrap();
        let task = temp.0.join("task");
        crate::capstone::init(&task).unwrap();
        assert!(next(&task).unwrap().contains("TaskPocket"));
        let page = temp.0.join("page");
        crate::transfer::init(&page).unwrap();
        assert!(next(&page).unwrap().contains("PageMint"));
    }
    #[test]
    fn rejects_unknown_or_symlink_directories() {
        let temp = crate::capstone::TempWorkspace::new_named("learning-invalid").unwrap();
        assert!(next(&temp.0).unwrap_err().contains("不是已识别"));
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&temp.0, temp.0.join("link")).unwrap();
            assert!(next(&temp.0.join("link")).is_err());
        }
    }
    fn write_matching_report(root: &Path, score: &str, extra: &str) {
        let mut text = String::new();
        for (p, hash) in source_hashes(root).unwrap() {
            text.push_str(&format!("- `{p}`：`{hash}`\n"));
        }
        text.push_str(extra);
        text.push_str(&format!("\n本次累计：{score}。这是导航测试\n"));
        fs::create_dir_all(root.join(".studio-reports")).unwrap();
        fs::write(root.join(".studio-reports/ASSESSMENT-123.md"), text).unwrap();
    }
    #[test]
    fn failure_resumes_the_first_failed_step_not_the_last_pass() {
        let t = crate::capstone::TempWorkspace::new_named("learn-step").unwrap();
        let p = t.0.join("中文 space");
        studio::init("unitlab", &p).unwrap();
        for (score, step) in [("0/3", 1), ("1/3", 2), ("2/3", 3)] {
            write_matching_report(&p, score, "");
            assert_eq!(step_for(&p, None).unwrap(), step);
            assert!(next(&p)
                .unwrap()
                .contains(&format!("第一项失败在第 {step} 步")));
        }
    }
    #[test]
    fn compilation_failure_keeps_the_attempted_scope() {
        let t = crate::capstone::TempWorkspace::new_named("learn-compile").unwrap();
        let p = t.0.join("p");
        studio::init("quizdeck", &p).unwrap();
        write_matching_report(&p, "0/3", "\n## 编译 FAIL\nerror[E0308]");
        assert_eq!(step_for(&p, None).unwrap(), 3);
        assert!(next(&p).unwrap().contains("先修复第一条 error"));
    }
    #[test]
    fn completed_navigation_offers_real_work_not_more_todo() {
        let t = crate::capstone::TempWorkspace::new_named("learn-complete").unwrap();
        let p = t.0.join("p");
        studio::init("unitlab", &p).unwrap();
        write_matching_report(&p, "3/3", "");
        let out = next(&p).unwrap();
        assert!(out.contains("studio start quizdeck"));
        assert!(!out.contains("对应 TODO(3) 并保存"));
    }
    #[test]
    fn modified_source_rechecks_preceding_steps_without_certifying_old_pass() {
        let t = crate::capstone::TempWorkspace::new_named("learn-stale").unwrap();
        let p = t.0.join("p");
        studio::init("unitlab", &p).unwrap();
        write_matching_report(&p, "1/1", "");
        assert_eq!(step_for(&p, None).unwrap(), 2);
        fs::write(p.join("src/lib.rs"), "// edited\n").unwrap();
        assert_eq!(step_for(&p, None).unwrap(), 2);
        assert!(next(&p).unwrap().contains("重新检查第 1–2 步"));
    }
    #[test]
    fn task_cards_are_complete_read_only_and_locate_functions() {
        let t = crate::capstone::TempWorkspace::new_named("learn-cards").unwrap();
        for id in ["unitlab", "quizdeck", "textscope"] {
            let p = t.0.join(id);
            studio::init(id, &p).unwrap();
            let before = source_hashes(&p).unwrap();
            for step in 1..=3 {
                let c = lesson(&p, Some(step)).unwrap();
                assert!(c.contains("先预测"));
                assert!(c.contains("复盘"));
                assert!(c.contains("行附近"));
                let hints: Vec<_> = (1..=3).map(|n| hint(&p, n, Some(step)).unwrap()).collect();
                assert_ne!(hints[0], hints[1]);
                assert_ne!(hints[1], hints[2]);
            }
            assert_eq!(before, source_hashes(&p).unwrap());
            assert!(!p.join(".studio-reports").exists());
        }
    }
    #[test]
    fn second_tranche_cards_are_actionable_and_do_not_edit_student_source() {
        let t = crate::capstone::TempWorkspace::new_named("learn-next-cards").unwrap();
        for (id, funcs) in [
            ("ledgerlite", ["parse_cents", "parse_entries", "totals"]),
            ("badgeforge", ["escape_xml", "render", "document"]),
            ("termgarden", ["parse_command", "apply_event", "render"]),
        ] {
            let root = t.0.join(id);
            studio::init(id, &root).unwrap();
            let before = source_hashes(&root).unwrap();
            for step in 1..=3 {
                let card = crate::learning_cards::get(id, step).unwrap();
                assert_eq!(card.function, funcs[step - 1]);
                let page = lesson(&root, Some(step)).unwrap();
                assert!(page.contains("先预测"));
                assert!(page.contains("行附近"));
                assert!(page.contains("复盘"));
                for level in 1..=3 {
                    assert!(hint(&root, level, Some(step)).unwrap().contains("第"));
                }
                assert!(crate::learning::printable_tasks(id)
                    .unwrap()
                    .contains(&format!("第 {step} 步")));
            }
            assert_eq!(before, source_hashes(&root).unwrap());
            assert!(!root.join(".studio-reports").exists());
        }
    }
    #[test]
    fn systems_cards_match_real_functions_and_are_read_only() {
        let t = crate::capstone::TempWorkspace::new_named("systems-cards").unwrap();
        for (id, functions) in [
            ("batchmeter", ["analyze", "aggregate", "analyze_many"]),
            ("minikv", ["validate_key", "execute", "next"]),
            ("netfetch", ["parse_url", "parse_response", "download"]),
            ("servicedesk", ["validate_title", "apply", "render_json"]),
        ] {
            let root = t.0.join(id);
            studio::init(id, &root).unwrap();
            let before = source_hashes(&root).unwrap();
            for step in 1..=3 {
                let card = crate::learning_cards::get(id, step).unwrap();
                assert_eq!(card.function, functions[step - 1]);
                let task = lesson(&root, Some(step)).unwrap();
                for term in ["先预测", "行附近", "复盘", functions[step - 1]] {
                    assert!(task.contains(term), "{id}: {term}");
                }
                let hints: Vec<_> = (1..=3)
                    .map(|n| hint(&root, n, Some(step)).unwrap())
                    .collect();
                assert_ne!(hints[0], hints[1]);
                assert_ne!(hints[1], hints[2]);
            }
            let printable = printable_tasks(id).unwrap();
            assert_eq!(printable.matches("<details>").count(), 9);
            assert_eq!(
                fs::read_to_string(root.join("TASK_CARDS.md")).unwrap(),
                printable
            );
            assert_eq!(before, source_hashes(&root).unwrap());
            assert!(!root.join(".studio-reports").exists());
        }
    }
    #[test]
    fn cards_are_unique_and_supported() {
        let mut seen = std::collections::BTreeSet::new();
        for card in crate::learning_cards::CARDS {
            assert!(seen.insert((card.id, card.step)));
            assert!(studio::product(card.id).is_ok());
            assert!((1..=3).contains(&card.step));
            assert!(card.hints.iter().all(|hint| !hint.is_empty()));
        }
        assert_eq!(seen.len(), 36);
    }
    #[test]
    fn task_pages_are_offline_and_escape_code_symbols() {
        for id in [
            "unitlab",
            "quizdeck",
            "textscope",
            "ledgerlite",
            "badgeforge",
            "termgarden",
            "batchmeter",
            "minikv",
            "netfetch",
            "servicedesk",
            "storedesk",
            "asyncboard",
        ] {
            let page = printable_tasks_html(id).unwrap();
            assert_eq!(page.matches("<details>").count(), 9);
            assert!(page.contains("id=\"step-3\""));
            assert!(page.contains("step1_"));
            assert!(!page.contains("<script"));
            assert!(!page.contains("/mnt/data/"));
            assert!(!page.contains("src=\"http"));
            assert!(!page.contains("href=\"http"));
        }
        assert!(printable_tasks_html("minikv").unwrap().contains("&amp;str"));
        assert!(printable_tasks_html("unknown").is_none());
        assert!(printable_tasks_html("storedesk")
            .unwrap()
            .contains("transaction"));
    }
    #[test]
    fn systems_handoff_preserves_real_product_boundaries() {
        assert!(handoff("batchmeter").contains("studio start minikv"));
        assert!(handoff("minikv").contains("studio start netfetch"));
        assert!(handoff("netfetch").contains("同步"));
        assert!(handoff("servicedesk").contains("不是 Tokio"));
        assert!(ROUTE.contains("SYSTEMS_PATH.md"));
    }
    #[test]
    fn overview_is_read_only_for_multiple_projects_and_preserves_stale_distinction() {
        let t = crate::capstone::TempWorkspace::new_named("learn-overview").unwrap();
        let base = t.0.join("学习 中文 space");
        fs::create_dir(&base).unwrap();
        let studio_dir = base.join("Studio");
        fs::create_dir(&studio_dir).unwrap();
        let unit = studio_dir.join("unitlab");
        let ledger = studio_dir.join("ledgerlite");
        studio::init("unitlab", &unit).unwrap();
        studio::init("ledgerlite", &ledger).unwrap();
        let first = base.join("FirstCard");
        crate::beginner::init(&first).unwrap();
        let independent = base.join("TaskPocket");
        crate::capstone::init(&independent).unwrap();
        write_matching_report(&unit, "3/3", "");
        write_matching_report(&ledger, "1/1", "");
        fs::write(ledger.join("src/lib.rs"), "// learner changed this\n").unwrap();
        let u_before = source_hashes(&unit).unwrap();
        let l_before = source_hashes(&ledger).unwrap();
        let message = overview(&base).unwrap();
        assert!(message.contains("已识别工程：4"));
        assert!(message.contains("历史通过：1"));
        assert!(message.contains("报告过期：1"));
        assert!(message.contains("旧报告已过期"));
        assert!(message.contains("FirstCard"));
        assert!(message.contains("TaskPocket"));
        assert!(message.contains("不是防作弊成绩"));
        assert_eq!(u_before, source_hashes(&unit).unwrap());
        assert_eq!(l_before, source_hashes(&ledger).unwrap());
        assert!(!first.join(".studio-reports").exists());
        // A second read leaves the exact same result, and does not execute programs.
        assert_eq!(message, overview(&base).unwrap());
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&unit, base.join("linked")).unwrap();
            assert_eq!(message, overview(&base).unwrap());
        }
    }
    #[test]
    fn overview_does_not_certify_fake_or_unrelated_reports() {
        let t = crate::capstone::TempWorkspace::new_named("learn-overview-stale").unwrap();
        let project = t.0.join("my-badgeforge");
        studio::init("badgeforge", &project).unwrap();
        assert!(overview(&t.0).unwrap().contains("未检查"));
        fs::create_dir(project.join(".studio-reports")).unwrap();
        fs::write(project.join(".studio-reports/ASSESSMENT-123.md"), "本次累计：3/3。\n- `src/lib.rs`：`0000000000000000000000000000000000000000000000000000000000000000`\n").unwrap();
        let status = overview(&t.0).unwrap();
        assert!(status.contains("旧报告已过期"));
        assert!(status.contains("历史通过：0"));
    }
    #[test]
    fn invalid_steps_and_hint_levels_do_not_create_reports() {
        let t = crate::capstone::TempWorkspace::new_named("learn-invalid-step").unwrap();
        let p = t.0.join("p");
        studio::init("unitlab", &p).unwrap();
        assert!(hint(&p, 0, None).is_err());
        assert!(hint(&p, 4, None).is_err());
        assert!(lesson(&p, Some(0)).is_err());
        assert!(check_current(&p, Some(4)).is_err());
        assert!(!p.join(".studio-reports").exists());
    }
    #[test]
    fn asyncboard_guidance_is_not_a_fallback() {
        let t = crate::capstone::TempWorkspace::new_named("learn-fallback").unwrap();
        let p = t.0.join("p");
        studio::init("asyncboard", &p).unwrap();
        assert!(lesson(&p, None).unwrap().contains("parse_jobs"));
        assert!(hint(&p, 1, None).unwrap().contains("Tab"));
    }
    #[test]
    fn diagnostic_handles_color_locations_and_first_error_only() {
        let d=diagnose_text("\x1b[31merror[E0308]\x1b[0m: mismatched types\n --> src/lib.rs:12:5\nerror[E0382]: moved");
        assert!(d.contains("src/lib.rs:12:5"));
        assert!(d.contains("分号"));
        assert!(!d.contains("Rust 诊断：E0382"));
        assert!(
            diagnose_text("error: unexpected closing delimiter\nerror[E0308]: later")
                .contains("语法检查")
        );
        assert!(
            !diagnose_text("error[E9999]: first\nerror[E0308]: later").contains("Rust 诊断：E0308")
        );
    }
    #[test]
    fn diagnostic_files_support_utf8_and_bom_utf16() {
        let t = crate::capstone::TempWorkspace::new_named("learn-enc").unwrap();
        let p = t.0.join("error.txt");
        let text = "error[E0308]: 类型错误\n";
        for little in [true, false] {
            let mut bytes = if little {
                vec![0xff, 0xfe]
            } else {
                vec![0xfe, 0xff]
            };
            for word in text.encode_utf16() {
                bytes.extend_from_slice(&if little {
                    word.to_le_bytes()
                } else {
                    word.to_be_bytes()
                });
            }
            fs::write(&p, bytes).unwrap();
            assert_eq!(read_diagnostics(&p).unwrap(), text);
        }
        fs::write(
            &p,
            [vec![0xef, 0xbb, 0xbf], text.as_bytes().to_vec()].concat(),
        )
        .unwrap();
        assert_eq!(read_diagnostics(&p).unwrap(), text);
        fs::write(&p, [0xff, 0xfe, 1]).unwrap();
        assert!(read_diagnostics(&p).is_err());
        fs::write(&p, [0xff, 0xfe, 0x00, 0xd8]).unwrap();
        assert!(read_diagnostics(&p).is_err());
        fs::write(&p, vec![b'x'; MAX_ERROR_BYTES as usize + 1]).unwrap();
        assert!(read_diagnostics(&p).is_err());
    }
    #[test]
    fn firstcard_and_symlink_behaviors_are_explicit() {
        let t = crate::capstone::TempWorkspace::new_named("learn-first").unwrap();
        let p = t.0.join("p");
        crate::beginner::init(&p).unwrap();
        assert!(lesson(&p, None).unwrap().contains("println!"));
        assert!(hint(&p, 1, None).is_ok());
        assert!(lesson(&p, Some(2)).is_err());
        #[cfg(unix)]
        {
            let link = t.0.join("link");
            std::os::unix::fs::symlink(&p, &link).unwrap();
            assert!(lesson(&link, None).is_err());
            assert!(hint(&link, 1, None).is_err());
            assert!(check_current(&link, None).is_err());
        }
    }
    #[test]
    fn score_uses_final_summary_not_program_output() {
        assert_eq!(
            parse_score("本次累计：3/3\nprogram output\n本次累计：0/2。"),
            Some((0, 2))
        );
    }
    #[test]
    fn printed_task_cards_cover_all_steps_without_machine_paths() {
        for id in ["unitlab", "quizdeck", "textscope"] {
            let text = printable_tasks(id).unwrap();
            assert_eq!(text.matches("<details>").count(), 9);
            assert!(text.contains("step3_"));
            assert!(!text.contains("/mnt/data/"));
        }
        assert!(printable_tasks("storedesk").unwrap().contains("回滚"));
        assert!(printable_tasks("asyncboard").unwrap().contains("Waker"));
    }
    #[test]
    fn reliability_cards_match_functions_and_are_read_only() {
        let t = crate::capstone::TempWorkspace::new_named("reliability-cards").unwrap();
        for (id, names) in [
            ("storedesk", ["validate_item", "save_item", "apply_changes"]),
            ("asyncboard", ["parse_jobs", "render_report", "complete"]),
        ] {
            let root = t.0.join(id);
            studio::init(id, &root).unwrap();
            let before = fs::read(root.join("src/lib.rs")).unwrap();
            for step in 1..=3 {
                let card = crate::learning_cards::get(id, step).unwrap();
                assert_eq!(card.function, names[step - 1]);
                let task = lesson(&root, Some(step)).unwrap();
                assert!(task.contains("行附近"), "{task}");
                for level in 1..=3 {
                    assert!(hint(&root, level, Some(step)).is_ok());
                }
            }
            assert_eq!(before, fs::read(root.join("src/lib.rs")).unwrap());
            assert!(!root.join(".studio-reports").exists());
            assert!(root.join("TASK_CARDS.html").is_file());
        }
    }
}
