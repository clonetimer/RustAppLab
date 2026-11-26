//! The browser edits a small, explicitly selected learner project. All course
//! content and historical check interpretation come from the offline CLI.
use super::{book_projects, ApiError};
use crate::{beginner, learning, learning_cards, studio};
use learning_cards::Card;
use safesync::sha256::{hex, Sha256};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

const MAX_BYTES: usize = 1024 * 1024;
const MAX_FILES: usize = 64;
const MAX_DEPTH: usize = 16;
const FIRST_TITLE: &str = "FirstCard · 第一张任务卡";
const FIRST_STEPS: &[&str] = &["让名字和任务出现在输出里"];
const FIRST: Card = Card {
    id: "firstcard",
    step: 1,
    function: "println!",
    goal: "只修改 src/main.rs 中的 println! 行，让任务卡随输入的姓名和任务改变。",
    concept: "命令行参数、变量与格式化输出。",
    prediction: "把姓名换成“阿青”、任务换成“整理笔记”，程序应该改变哪两处输出？",
    hints: beginner::HINTS,
    experiment:
        "自选两组不同的姓名和任务，先预测，再运行并比较实际输出。试一次不传参数，观察默认值。",
    reflection: "name 和直接写“小林”有什么区别？为什么只换输入就能得到一张新任务卡？",
};

fn course_ids() -> impl Iterator<Item = &'static str> {
    std::iter::once(FIRST.id)
        .chain(
            learning_cards::CARDS
                .iter()
                .filter(|c| c.step == 1)
                .map(|c| c.id),
        )
        .chain(book_projects::all().map(|c| c.id))
}

struct Course {
    title: &'static str,
    steps: &'static [&'static str],
    default_file: &'static str,
    page: &'static str,
}

fn course(id: &str) -> Result<Course, ApiError> {
    if let Some(book) = book_projects::get(id) {
        return Ok(Course {
            title: book.title,
            steps: book.steps(),
            default_file: book.default_file,
            page: "",
        });
    }
    if id == FIRST.id {
        return Ok(Course {
            title: FIRST_TITLE,
            steps: FIRST_STEPS,
            default_file: "src/main.rs",
            page: include_str!("../../../../START_HERE.html"),
        });
    }
    let p = studio::product(id).map_err(|s| ApiError::new(404, s))?;
    Ok(Course {
        title: p.title,
        steps: &p.steps,
        default_file: "src/lib.rs",
        page: p.page,
    })
}

pub(super) fn validate_step(id: &str, step: usize) -> Result<(), ApiError> {
    let n = course(id)?.steps.len();
    if !(1..=n).contains(&step) {
        return Err(ApiError::new(400, format!("当前课程步骤范围为 1 到 {n}")));
    }
    Ok(())
}

fn run_args(id: &str) -> &'static [&'static str] {
    if let Some(book) = book_projects::get(id) {
        return book.run.as_ref().map(|run| run.args).unwrap_or(&[]);
    }
    match id {
        "firstcard" => &["小林", "写 Rust"],
        "unitlab" => &["convert", "25", "c", "f"],
        "quizdeck" => &["check", "samples/deck.tsv", "samples/answers.txt"],
        "textscope" => &["stats", "samples/notes.txt"],
        "ledgerlite" => &["summary", "samples/expenses.tsv"],
        "badgeforge" => &["demo"],
        "batchmeter" => &["scan", "2", "samples/one.txt", "samples/two.txt"],
        "minikv" => &["memory-demo"],
        "termgarden" | "servicedesk" | "netfetch" => &["demo", "my-demo"],
        "storedesk" => &["doctor"],
        "asyncboard" => &["preview", "samples/jobs.tsv"],
        _ => &[],
    }
}

pub(super) fn json_string(value: &str) -> String {
    let mut out = String::from("\"");
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < ' ' => write!(out, "\\u{:04x}", c as u32).unwrap(),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn json_list<'a>(values: impl IntoIterator<Item = &'a str>) -> String {
    let entries: Vec<_> = values.into_iter().map(json_string).collect();
    format!("[{}]", entries.join(","))
}

fn hash_field(hash: &mut Sha256, value: &[u8]) {
    hash.update(&(value.len() as u64).to_be_bytes());
    hash.update(value);
}

// A second tab may open a different project whose starter bytes are identical.
fn file_revision(path: &Path, data: &[u8]) -> String {
    let mut hash = Sha256::new();
    hash_field(&mut hash, path.as_os_str().as_encoded_bytes());
    hash_field(&mut hash, data);
    hex(&hash.finalize())
}

fn io_error(path: &Path, err: io::Error) -> ApiError {
    let status = match err.kind() {
        io::ErrorKind::NotFound => 404,
        io::ErrorKind::PermissionDenied => 403,
        _ => 500,
    };
    ApiError::new(status, format!("无法访问 {}：{err}", path.display()))
}

fn invalid(message: impl Into<String>) -> ApiError {
    ApiError::new(400, message)
}

fn is_link(meta: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        // Directory junctions and other reparse points can redirect resolution.
        meta.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        meta.file_type().is_symlink()
    }
}

fn metadata(path: &Path) -> Result<fs::Metadata, ApiError> {
    let meta = fs::symlink_metadata(path).map_err(|e| io_error(path, e))?;
    if is_link(&meta) {
        return Err(invalid(format!(
            "路径含符号链接或重解析点：{}",
            path.display()
        )));
    }
    Ok(meta)
}

fn check_dirs(path: &Path) -> Result<(), ApiError> {
    if path.components().any(|c| c == Component::ParentDir) {
        return Err(invalid("工程路径不能包含 ..，请使用实际路径"));
    }
    let ancestors: Vec<_> = path
        .ancestors()
        .filter(|p| !p.as_os_str().is_empty())
        .collect();
    for p in ancestors.into_iter().rev() {
        if !metadata(p)?.is_dir() {
            return Err(invalid(format!("需要普通目录：{}", p.display())));
        }
    }
    Ok(())
}

fn read_bytes(path: &Path, limit: usize) -> Result<Vec<u8>, ApiError> {
    let meta = metadata(path)?;
    if !meta.is_file() {
        return Err(invalid(format!("需要普通文件：{}", path.display())));
    }
    if meta.len() > limit as u64 {
        return Err(ApiError::new(413, "文件数量或大小超过教学工作台上限"));
    }
    let mut bytes = Vec::with_capacity(meta.len() as usize);
    File::open(path)
        .and_then(|f| f.take(limit as u64 + 1).read_to_end(&mut bytes))
        .map_err(|e| io_error(path, e))?;
    if bytes.len() > limit {
        return Err(ApiError::new(413, "读取期间文件超过教学工作台上限"));
    }
    Ok(bytes)
}

struct ProjectFile {
    bytes: Vec<u8>,
    editable: bool,
}

#[derive(Default)]
struct Snapshot {
    files: BTreeMap<String, ProjectFile>,
}

impl Snapshot {
    fn read(root: &Path, id: &str) -> Result<Self, ApiError> {
        let mut snapshot = Self::default();
        if let Some(book) = book_projects::get(id) {
            snapshot.walk_book(root, "", book, 0, &mut TreeBudget::default())?;
            for file in book.files.iter().filter(|file| book.protected(file.path)) {
                let actual = snapshot.files.get(file.path).ok_or_else(|| {
                    invalid(format!(
                        "Book 工程缺少课程检查文件 {}；请创建新学员副本并迁移自己的源码，保留旧目录作为备份",
                        file.path
                    ))
                })?;
                let actual = std::str::from_utf8(&actual.bytes)
                    .map_err(|_| invalid("Book 工程清单和检查文件需要是 UTF-8"))?;
                if actual.replace("\r\n", "\n") != file.content.replace("\r\n", "\n") {
                    return Err(invalid(format!(
                        "课程清单或检查文件 {} 已改变；工作台只接受本课程的原始清单和检查文件",
                        file.path
                    )));
                }
            }
            snapshot.file(book.default_file)?;
            return Ok(snapshot);
        }
        snapshot.insert(root, "Cargo.toml", false, MAX_BYTES)?;
        snapshot.walk(root, "src", true, 0, &mut TreeBudget::default())?;
        if present(&root.join("samples"))? {
            snapshot.walk(root, "samples", false, 0, &mut TreeBudget::default())?;
        }
        if present(&root.join("LEARNING_LOG.md"))? {
            snapshot.insert(root, "LEARNING_LOG.md", true, MAX_BYTES)?;
        }
        Ok(snapshot)
    }

    fn insert(
        &mut self,
        root: &Path,
        rel: &str,
        editable: bool,
        limit: usize,
    ) -> Result<usize, ApiError> {
        let bytes = read_bytes(&root.join(rel), limit)?;
        if editable && std::str::from_utf8(&bytes).is_err() {
            return Err(invalid(format!("{rel} 不是有效的 UTF-8 文本")));
        }
        let size = bytes.len();
        self.files
            .insert(rel.to_owned(), ProjectFile { bytes, editable });
        Ok(size)
    }

    fn walk(
        &mut self,
        root: &Path,
        rel: &str,
        source: bool,
        depth: usize,
        budget: &mut TreeBudget,
    ) -> Result<(), ApiError> {
        let dir = root.join(rel);
        if depth > MAX_DEPTH || !metadata(&dir)?.is_dir() {
            return Err(invalid(format!("{rel} 需要是普通目录，嵌套最多 16 层")));
        }
        for item in fs::read_dir(&dir).map_err(|e| io_error(&dir, e))? {
            let entry = item.map_err(|e| io_error(&dir, e))?;
            budget.entries += 1;
            // Also bound trees consisting entirely of empty directories.
            if budget.entries > 1024 {
                return Err(ApiError::new(413, "目录条目过多"));
            }
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| invalid("文件名需要是 UTF-8"))?;
            if name.contains(['\\', ':']) || name.chars().any(char::is_control) {
                return Err(invalid("文件名包含不可用的路径字符"));
            }
            let path = entry.path();
            let child = format!("{rel}/{name}");
            let meta = metadata(&path)?;
            if meta.is_dir() {
                self.walk(root, &child, source, depth + 1, budget)?;
            } else {
                budget.files += 1;
                if budget.files > MAX_FILES {
                    return Err(ApiError::new(413, "源码或样本目录最多支持 64 个文件"));
                }
                let ext = path.extension().and_then(|x| x.to_str());
                let editable = if source {
                    ext == Some("rs")
                } else {
                    matches!(ext, Some("txt" | "tsv" | "csv" | "json" | "md"))
                };
                // Some products embed sample assets such as HTML at compile time.
                // Include their bytes in freshness without exposing them to edits.
                budget.bytes += self.insert(root, &child, editable, MAX_BYTES - budget.bytes)?;
            }
        }
        Ok(())
    }

    fn walk_book(
        &mut self,
        root: &Path,
        rel: &str,
        book: &book_projects::BookCourse,
        depth: usize,
        budget: &mut TreeBudget,
    ) -> Result<(), ApiError> {
        let dir = root.join(rel);
        if depth > MAX_DEPTH || !metadata(&dir)?.is_dir() {
            return Err(invalid("Book 工程需要普通目录，嵌套最多 16 层"));
        }
        for item in fs::read_dir(&dir).map_err(|e| io_error(&dir, e))? {
            let entry = item.map_err(|e| io_error(&dir, e))?;
            budget.entries += 1;
            if budget.entries > 1024 {
                return Err(ApiError::new(413, "目录条目过多"));
            }
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| invalid("文件名需要是 UTF-8"))?;
            if name.contains(['\\', ':']) || name.chars().any(char::is_control) {
                return Err(invalid("文件名包含不可用的路径字符"));
            }
            let child = if rel.is_empty() {
                name.clone()
            } else {
                format!("{rel}/{name}")
            };
            let meta = metadata(&entry.path())?;
            if meta.is_dir() {
                // Cargo output and Git metadata are not inputs. Their directory
                // itself must still be real, never a symlink or reparse point.
                if name == "target" || name == ".git" {
                    continue;
                }
                if name == ".cargo" {
                    return Err(invalid("Book 学员副本不能覆盖课程的 Cargo 执行配置"));
                }
                self.walk_book(root, &child, book, depth + 1, budget)?;
                continue;
            }
            let canonical = book.files.iter().any(|file| file.path == child);
            let source = book.source_path(&child);
            if !canonical && !source && child != "LEARNING_LOG.md" {
                return Err(invalid(format!(
                    "Book 工程包含课程外的输入文件 {child}；请在干净的学员副本中继续"
                )));
            }
            budget.files += 1;
            if budget.files > MAX_FILES {
                return Err(ApiError::new(413, "Book 工程最多支持 64 个输入文件"));
            }
            let editable = (source && child.ends_with(".rs")) || child == "LEARNING_LOG.md";
            budget.bytes += self.insert(root, &child, editable, MAX_BYTES - budget.bytes)?;
        }
        Ok(())
    }

    fn file(&self, rel: &str) -> Result<&ProjectFile, ApiError> {
        self.files
            .get(rel)
            .filter(|f| f.editable)
            .ok_or_else(|| invalid("只能访问当前学员工程内已有的 .rs 源码、学习记录和文本样本"))
    }

    fn digest(&self, root: &Path) -> String {
        let mut hash = Sha256::new();
        hash_field(&mut hash, root.as_os_str().as_encoded_bytes());
        for (rel, file) in &self.files {
            if rel == "LEARNING_LOG.md" {
                continue;
            }
            hash_field(&mut hash, rel.as_bytes());
            hash_field(&mut hash, &file.bytes);
        }
        hex(&hash.finalize())
    }

    fn editable_paths(&self) -> String {
        json_list(
            self.files
                .iter()
                .filter(|(_, f)| f.editable)
                .map(|(p, _)| p.as_str()),
        )
    }
}

#[derive(Default)]
struct TreeBudget {
    entries: usize,
    files: usize,
    bytes: usize,
}

fn present(path: &Path) -> Result<bool, ApiError> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(io_error(path, e)),
    }
}

fn identify(root: &Path, id: &str) -> Result<(), ApiError> {
    check_dirs(root)?;
    if let Some(book) = book_projects::get(id) {
        let canonical = root.canonicalize().map_err(|e| io_error(root, e))?;
        if book.is_reference(&canonical) {
            return Err(invalid(
                "这是课程参考源码，请先创建学员副本；工作台不会编辑老师的原始工程",
            ));
        }
        Snapshot::read(root, id)?;
        return Ok(());
    }
    for name in [".applab-studio", ".studio-reports"] {
        let path = root.join(name);
        if present(&path)? {
            metadata(&path)?;
        }
    }
    let marker = learning::studio_id(root).map_err(invalid)?;
    let matches = match marker {
        Some(product) => product == id,
        None => id == FIRST.id && learning::first_card_manifest(root),
    };
    if !matches {
        return Err(invalid(format!(
            "所选目录与课程 {id} 不匹配，请选择该课程的学员工程"
        )));
    }
    let src = root.join("src");
    if !metadata(&src)?.is_dir()
        || !metadata(&src.join("main.rs"))?.is_file()
        || (id != FIRST.id && !metadata(&src.join("lib.rs"))?.is_file())
    {
        return Err(invalid("工程缺少学员源码入口文件"));
    }
    Ok(())
}

pub(super) struct Workspace {
    pub(super) root: PathBuf,
    selected: BTreeMap<String, PathBuf>,
}

impl Workspace {
    pub(super) fn new(root: PathBuf) -> Result<Self, ApiError> {
        let absolute = std::env::current_dir()
            .map_err(|e| io_error(&root, e))?
            .join(root);
        check_dirs(&absolute)?;
        let mut ws = Self {
            root: absolute.components().collect(),
            selected: BTreeMap::new(),
        };
        for id in course_ids() {
            let path = ws.default_path(id);
            if identify(&path, id).is_ok() && Snapshot::read(&path, id).is_ok() {
                ws.selected.insert(id.to_owned(), path);
            }
        }
        Ok(ws)
    }

    fn default_path(&self, id: &str) -> PathBuf {
        self.root.join(if id == FIRST.id {
            "my-first-card".into()
        } else {
            format!("my-{id}")
        })
    }

    pub(super) fn catalog_json(&self) -> Result<String, ApiError> {
        let mut entries = Vec::new();
        for id in course_ids() {
            let c = course(id)?;
            entries.push(format!(
                "{{\"id\":{},\"title\":{},\"steps\":{},\"default_file\":{},\"kind\":{},\"can_run\":{},\"lesson_topic\":{}}}",
                json_string(id),
                json_string(c.title),
                json_list(c.steps.iter().copied()),
                json_string(c.default_file),
                json_string(if book_projects::get(id).is_some() { "book" } else { "studio" }),
                book_projects::get(id).map(|book| book.run.is_some()).unwrap_or(true),
                book_projects::get(id).map(|book| json_string(book.lesson_topic)).unwrap_or_else(|| "null".into())
            ));
        }
        Ok(format!(
            "{{\"version\":{},\"workspace_root\":{},\"courses\":[{}]}}",
            json_string(env!("CARGO_PKG_VERSION")),
            json_string(&self.root.to_string_lossy()),
            entries.join(",")
        ))
    }

    pub(super) fn course_json(&self, id: &str, step: usize) -> Result<String, ApiError> {
        validate_step(id, step)?;
        let project = match self.selected.get(id) {
            Some(path) => Self::project_json(path, id)?,
            None => "null".into(),
        };
        self.card_json(id, step, &project)
    }

    fn card_json(&self, id: &str, step: usize, project: &str) -> Result<String, ApiError> {
        let c = course(id)?;
        let book = book_projects::get(id);
        let book_card = book.map(|book| book.card(step));
        let card = if let Some(card) = book_card.as_ref() {
            card
        } else if id == FIRST.id {
            &FIRST
        } else {
            learning_cards::get(id, step)
                .ok_or_else(|| ApiError::new(404, "没有找到这一步的任务卡"))?
        };
        let next = course_ids()
            .skip_while(|&v| v != id)
            .nth(1)
            .map(json_string)
            .unwrap_or_else(|| "null".into());
        Ok(format!(concat!("{{\"id\":{},\"title\":{},\"step\":{},\"step_count\":{},\"step_title\":{},",
            "\"goal\":{},\"function\":{},\"concept\":{},\"prediction\":{},\"hints\":{},",
            "\"experiment\":{},\"reflection\":{},\"next_course\":{},\"run_args\":{},\"reading\":{},\"project\":{},",
            "\"kind\":{},\"can_run\":{},\"lesson_topic\":{}}}"),
            json_string(id), json_string(c.title), step, c.steps.len(), json_string(c.steps[step-1]),
            json_string(card.goal), json_string(card.function), json_string(card.concept), json_string(card.prediction),
            json_list(card.hints), json_string(card.experiment), json_string(card.reflection), next, json_list(run_args(id).iter().copied()), crate::book_reading::workbench_json(id), project,
            json_string(if book.is_some() { "book" } else { "studio" }),
            book.map(|book| book.run.is_some()).unwrap_or(true),
            book.map(|book| json_string(book.lesson_topic)).unwrap_or_else(|| "null".into())))
    }

    fn project_json(path: &Path, id: &str) -> Result<String, ApiError> {
        identify(path, id)?;
        let snap = Snapshot::read(path, id)?;
        let (text, state) = if book_projects::get(id).is_some_and(|book| book.is_modern()) {
            ("现代课程：先安装固定 Rust 1.100.0。只编辑自己的 src/lib.rs；逐步检查，最后综合验证。学习记录不自动评分。".to_string(), None)
        } else if book_projects::get(id).is_some() {
            ("Book 实验工程：按预测、修改、检查、解释、迁移完成学习记录。检查执行当前副本的真实单测和文档测试；通过不代表学习步骤已完成。".to_string(), None)
        } else {
            learning::navigation(path).map_err(invalid)?
        };
        let history = state.map(|r| format!("{{\"passed_steps\":{},\"upto_step\":{},\"same_source\":{},\"compilation_failed\":{}}}", r.passed, r.upto, r.same_source, r.compilation_failed)).unwrap_or_else(|| "null".into());
        Ok(format!(
            concat!(
                "{{\"path\":{},\"files\":{},\"source_digest\":{},",
                "\"navigation\":{},\"suggested_step\":{},\"history\":{}}}"
            ),
            json_string(&path.to_string_lossy()),
            snap.editable_paths(),
            json_string(&snap.digest(path)),
            json_string(&text),
            state.map(|r| r.step()).unwrap_or(1),
            history
        ))
    }

    pub(super) fn lesson_html(&self, id: &str) -> Result<String, ApiError> {
        if let Some(book) = book_projects::get(id) {
            let body = book
                .readme()
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;");
            return Ok(format!("<!doctype html><html lang=\"zh-CN\"><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>{}</title><body><pre>{body}</pre></body></html>", book.title));
        }
        Ok(course(id)?.page.to_owned())
    }

    pub(super) fn create(&mut self, id: &str) -> Result<String, ApiError> {
        course(id)?;
        check_dirs(&self.root)?;
        let path = self.default_path(id);
        if present(&path)? {
            return Err(ApiError::new(
                409,
                format!("已有路径不会覆盖：{}；请打开已有工程", path.display()),
            ));
        }
        if let Some(book) = book_projects::get(id) {
            book.create(&path)
        } else if id == FIRST.id {
            beginner::init(&path)
        } else {
            studio::init(id, &path)
        }
        .map_err(|e| ApiError::new(500, e))?;
        self.open(id, &path)
    }

    pub(super) fn open(&mut self, id: &str, path: &Path) -> Result<String, ApiError> {
        course(id)?;
        if path.as_os_str().is_empty() {
            return Err(invalid("请填写已有学员工程路径"));
        }
        let path: PathBuf = self.root.join(path).components().collect();
        let project = Self::project_json(&path, id)?;
        let result = self.card_json(id, 1, &project)?;
        self.selected.insert(id.to_owned(), path);
        Ok(result)
    }

    pub(super) fn project_path(&self, id: &str) -> Result<PathBuf, ApiError> {
        course(id)?;
        let path = self
            .selected
            .get(id)
            .ok_or_else(|| ApiError::new(409, "请先创建或打开该课程的学员工程"))?;
        identify(path, id)?;
        Ok(path.clone())
    }

    pub(super) fn source_digest(&self, id: &str) -> Result<String, ApiError> {
        let root = self.project_path(id)?;
        Ok(Snapshot::read(&root, id)?.digest(&root))
    }

    pub(super) fn file_json(&self, id: &str, rel: &str) -> Result<String, ApiError> {
        let root = self.project_path(id)?;
        let snapshot = Snapshot::read(&root, id)?;
        let bytes = &snapshot.file(rel)?.bytes;
        let content = std::str::from_utf8(bytes).map_err(|_| invalid("文本需要是 UTF-8"))?;
        Ok(format!(
            "{{\"path\":{},\"content\":{},\"revision\":{},\"source_digest\":{}}}",
            json_string(rel),
            json_string(content),
            json_string(&file_revision(&root.join(rel), bytes)),
            json_string(&snapshot.digest(&root))
        ))
    }

    pub(super) fn save(
        &self,
        id: &str,
        rel: &str,
        content: &str,
        revision: &str,
    ) -> Result<String, ApiError> {
        if content.len() > MAX_BYTES {
            return Err(ApiError::new(413, "文件大小超过 1 MiB"));
        }
        let root = self.project_path(id)?;
        let mut snapshot = Snapshot::read(&root, id)?;
        let file = snapshot.file(rel)?;
        let path = root.join(rel);
        if file_revision(&path, &file.bytes) != revision {
            return Err(conflict());
        }
        if book_projects::get(id).is_some() {
            let other_bytes: usize = snapshot
                .files
                .iter()
                .filter(|(p, _)| p.as_str() != rel)
                .map(|(_, file)| file.bytes.len())
                .sum();
            if other_bytes + content.len() > MAX_BYTES {
                return Err(ApiError::new(413, "Book 工程输入总量超过 1 MiB"));
            }
        } else if let Some((dir, _)) = rel.split_once('/') {
            let prefix = format!("{dir}/");
            let other_bytes: usize = snapshot
                .files
                .iter()
                .filter(|(p, _)| p.starts_with(&prefix) && p.as_str() != rel)
                .map(|(_, v)| v.bytes.len())
                .sum();
            if other_bytes + content.len() > MAX_BYTES {
                return Err(ApiError::new(413, "源码或样本总量超过 1 MiB"));
            }
        }
        replace_file(&path, content, revision)?;
        snapshot.files.get_mut(rel).unwrap().bytes = content.as_bytes().to_vec();
        Ok(format!(
            "{{\"path\":{},\"revision\":{},\"source_digest\":{}}}",
            json_string(rel),
            json_string(&file_revision(&path, content.as_bytes())),
            json_string(&snapshot.digest(&root))
        ))
    }
}

fn conflict() -> ApiError {
    ApiError::new(
        409,
        "工程或磁盘文件已更改，草稿未写入。请保留草稿并重新载入磁盘版本后合并。",
    )
}

// Stage the complete new bytes next to the target, then recheck the expected
// revision immediately before replacement. A failed write preserves the file.
fn replace_file(path: &Path, content: &str, revision: &str) -> Result<(), ApiError> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let dir = path.parent().ok_or_else(|| invalid("文件缺少父目录"))?;
    check_dirs(dir)?;
    let permissions = metadata(path)?.permissions();
    if permissions.readonly() {
        return Err(ApiError::new(403, "磁盘文件是只读的"));
    }
    let (temp, mut out) = loop {
        let index = NEXT.fetch_add(1, Ordering::Relaxed);
        let temp = dir.join(format!(
            ".rustapplab-save-{}-{index}.tmp",
            std::process::id()
        ));
        match OpenOptions::new().write(true).create_new(true).open(&temp) {
            Ok(f) => break (temp, f),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(io_error(&temp, e)),
        }
    };
    let result = (|| {
        out.set_permissions(permissions)
            .and_then(|_| out.write_all(content.as_bytes()))
            .and_then(|_| out.sync_all())
            .map_err(|e| io_error(path, e))?;
        drop(out);
        check_dirs(dir)?;
        if file_revision(path, &read_bytes(path, MAX_BYTES)?) != revision {
            return Err(conflict());
        }
        fs::rename(&temp, path).map_err(|e| io_error(path, e))
    })();
    if result.is_err() {
        let _ = fs::remove_file(temp);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capstone::TempWorkspace;

    fn workspace() -> (TempWorkspace, Workspace) {
        let parent = TempWorkspace::new_named("workbench-model").unwrap();
        let ws = Workspace::new(parent.0.clone()).unwrap();
        (parent, ws)
    }

    #[test]
    fn catalog_includes_every_canonical_card_and_lesson() {
        let (_parent, ws) = workspace();
        let catalog = ws.catalog_json().unwrap();
        let mut remaining = catalog.as_str();
        for id in course_ids() {
            let entry = format!("\"id\":{}", json_string(id));
            remaining = remaining.split_once(&entry).unwrap().1;
            assert!(ws.lesson_html(id).unwrap().contains("</html>"));
            assert!(ws.course_json(id, 1).unwrap().contains("\"project\":null"));
        }
        assert_eq!(course_ids().count(), 13 + book_projects::all().count());
        let unitlab = ws.course_json("unitlab", 1).unwrap();
        assert!(unitlab.contains(r#""reading":{"topics":["#));
        assert!(unitlab.contains("rust.bootcss.com/ch06-02-match.html"));
        for card in learning_cards::CARDS {
            let course = ws.course_json(card.id, card.step).unwrap();
            for field in [card.function, card.goal, card.hints[2], card.reflection] {
                assert!(course.contains(&json_string(field)));
            }
        }
        assert!(validate_step("firstcard", 2).is_err());
        assert!(validate_step("unitlab", 0).is_err());
        assert!(validate_step("taskpocket", 1).is_err());
    }

    #[test]
    fn book_copies_expose_nested_sources_without_fabricated_history() {
        let (_parent, mut ws) = workspace();
        let cases = [
            ("book-d1", "book-card/src/lib.rs"),
            ("book-d2", "smart-pointer-lab/src/lib.rs"),
            ("book-d3", "oop-pattern-lab/src/lib.rs"),
            ("book-d4", "advanced-lab/src/lib.rs"),
            ("book-d5", "webserver-lab/src/lib.rs"),
            ("book-d6", "appendix-lab/src/lib.rs"),
            ("book-foundations", "foundation-lab/src/lib.rs"),
        ];
        for (id, source) in cases {
            let card = ws.create(id).unwrap();
            assert!(card.contains("\"kind\":\"book\""), "{id}");
            assert!(card.contains("\"history\":null"), "{id}");
            assert!(ws.file_json(id, source).is_ok(), "{id}: {source}");
            assert!(ws.file_json(id, "LEARNING_LOG.md").is_ok());
            assert!(ws.file_json(id, "Cargo.toml").is_err());
            assert!(ws.file_json(id, "Cargo.lock").is_err());
            assert_eq!(ws.create(id).unwrap_err().status, 409);
        }
    }

    #[test]
    fn book_check_inputs_are_immutable_and_source_edits_change_freshness() {
        let (_parent, mut ws) = workspace();
        ws.create("book-d1").unwrap();
        let root = ws.project_path("book-d1").unwrap();
        let before = ws.source_digest("book-d1").unwrap();
        let source = root.join("book-card/src/lib.rs");
        let original = fs::read(&source).unwrap();
        let revised = format!(
            "{}\n// 学员自己的实验\n",
            String::from_utf8_lossy(&original)
        );
        ws.save(
            "book-d1",
            "book-card/src/lib.rs",
            &revised,
            &file_revision(&source, &original),
        )
        .unwrap();
        assert_ne!(ws.source_digest("book-d1").unwrap(), before);
        assert!(ws
            .file_json("book-d1", "cargo-applab-map/tests/cli.rs")
            .is_err());
        fs::write(root.join("cargo-applab-map/tests/cli.rs"), "// no tests").unwrap();
        assert!(ws.source_digest("book-d1").is_err());
    }

    #[test]
    fn book_nested_input_budget_and_extra_check_files_are_enforced() {
        let (_parent, mut ws) = workspace();
        ws.create("book-d4").unwrap();
        let root = ws.project_path("book-d4").unwrap();
        let source = root.join("d4-proc-macros/src/lib.rs");
        let original = fs::read(&source).unwrap();
        assert_eq!(
            ws.save(
                "book-d4",
                "d4-proc-macros/src/lib.rs",
                &"x".repeat(MAX_BYTES),
                &file_revision(&source, &original)
            )
            .unwrap_err()
            .status,
            413
        );
        assert_eq!(fs::read(&source).unwrap(), original);
        fs::write(
            root.join("advanced-lab/tests/extra.rs"),
            "#[test] fn fabricated() {}",
        )
        .unwrap();
        assert!(ws.source_digest("book-d4").is_err());
        fs::remove_file(root.join("advanced-lab/tests/extra.rs")).unwrap();
        fs::write(root.join("advanced-lab/build.rs"), "fn main() {}").unwrap();
        assert!(ws.source_digest("book-d4").is_err());
        fs::remove_file(root.join("advanced-lab/build.rs")).unwrap();
        fs::create_dir(root.join(".cargo")).unwrap();
        assert!(ws.source_digest("book-d4").is_err());
        fs::remove_dir(root.join(".cargo")).unwrap();
        let before = ws.source_digest("book-d4").unwrap();
        let log = root.join("LEARNING_LOG.md");
        let old_log = fs::read(&log).unwrap();
        ws.save(
            "book-d4",
            "LEARNING_LOG.md",
            "我预测宏会在编译期展开",
            &file_revision(&log, &old_log),
        )
        .unwrap();
        assert_eq!(ws.source_digest("book-d4").unwrap(), before);
    }

    #[cfg(unix)]
    #[test]
    fn book_nested_sources_and_output_paths_cannot_follow_symlinks() {
        use std::os::unix::fs::symlink;
        let (parent, mut ws) = workspace();
        ws.create("book-d1").unwrap();
        let root = ws.project_path("book-d1").unwrap();
        let external = parent.0.join("outside.rs");
        fs::write(&external, "// preserved").unwrap();
        symlink(&external, root.join("book-card/src/linked.rs")).unwrap();
        assert!(ws.file_json("book-d1", "book-card/src/linked.rs").is_err());
        fs::remove_file(root.join("book-card/src/linked.rs")).unwrap();
        symlink(&parent.0, root.join("target")).unwrap();
        assert!(ws.source_digest("book-d1").is_err());
        assert_eq!(fs::read_to_string(external).unwrap(), "// preserved");
    }

    #[test]
    fn teacher_book_sources_are_not_opened_for_editing() {
        let (_parent, mut ws) = workspace();
        let reference = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../projects/book-d1-cargo")
            .canonicalize()
            .unwrap();
        assert_eq!(ws.open("book-d1", &reference).unwrap_err().status, 400);
    }

    #[test]
    fn learner_kit_book_references_are_read_only_without_platform_sources() {
        let (parent, mut ws) = workspace();
        let kit = parent.0.join("learner-kit");
        fs::create_dir_all(kit.join("projects")).unwrap();
        fs::create_dir(kit.join("course")).unwrap();
        // These root paths are shipped by package_learner_kit.py. The kit does
        // not contain apps/labquest; reference detection must not depend on it.
        for name in [
            "START_HERE.md",
            "SUPPORTED_PLATFORMS.md",
            "course/CURRICULUM.md",
        ] {
            fs::write(kit.join(name), "learner-kit material").unwrap();
        }
        let reference = kit.join("projects/book-d6-appendices");
        book_projects::get("book-d6")
            .unwrap()
            .create(&reference)
            .unwrap();
        assert!(!kit.join("apps").exists());
        assert_eq!(ws.open("book-d6", &reference).unwrap_err().status, 400);
        let standalone = parent.0.join("my-older-book-copy");
        fs::rename(&reference, &standalone).unwrap();
        assert!(ws.open("book-d6", &standalone).is_ok());
    }

    #[test]
    fn standalone_book_copy_keeps_edits_and_rejects_wrong_identity() {
        let (parent, mut ws) = workspace();
        ws.create("book-d4").unwrap();
        let old = ws.project_path("book-d4").unwrap();
        let moved = parent.0.join("已有 D4 工程");
        fs::rename(&old, &moved).unwrap();
        fs::remove_file(moved.join("README.md")).unwrap();
        fs::write(moved.join("advanced-lab/src/lib.rs"), "// 我的修改").unwrap();
        assert!(ws.open("book-d2", &moved).is_err());
        assert!(ws.open("book-d4", &moved).is_ok());
        assert_eq!(
            fs::read_to_string(moved.join("advanced-lab/src/lib.rs")).unwrap(),
            "// 我的修改"
        );
        assert!(!moved.join("README.md").exists());
        fs::write(
            moved.join("advanced-lab/Cargo.toml"),
            "[package]\nname = \"unrelated\"\n",
        )
        .unwrap();
        assert!(ws.open("book-d4", &moved).is_err());
    }

    #[test]
    fn opening_existing_projects_preserves_edits_and_requires_the_right_course() {
        let (parent, mut ws) = workspace();
        let path = parent.0.join("我的 旧项目");
        studio::init("unitlab", &path).unwrap();
        fs::remove_file(path.join("TASK_CARDS.html")).unwrap();
        fs::remove_file(path.join("README.md")).unwrap();
        fs::write(path.join("src/lib.rs"), "// 我的源码").unwrap();
        assert!(ws.open("quizdeck", &path).is_err());
        assert!(ws.open("firstcard", &path).is_err());
        assert!(ws.open("unitlab", &path).unwrap().contains("我的 旧项目"));
        assert_eq!(
            fs::read_to_string(path.join("src/lib.rs")).unwrap(),
            "// 我的源码"
        );
        assert!(!path.join("README.md").exists());

        let default = parent.0.join("my-unitlab");
        fs::create_dir(&default).unwrap();
        let mut again = Workspace::new(parent.0.clone()).unwrap();
        assert!(again.project_path("unitlab").is_err());
        assert_eq!(again.create("unitlab").unwrap_err().status, 409);
        assert_eq!(fs::read_dir(&default).unwrap().count(), 0);
    }

    #[test]
    fn older_firstcard_needs_no_docs_and_never_awards_pass_from_a_record() {
        let (parent, _ws) = workspace();
        let path = parent.0.join("my-first-card");
        beginner::init(&path).unwrap();
        fs::remove_file(path.join("跟着做.md")).unwrap();
        fs::write(path.join("FIRST_WIN.md"), "4/4 PASS").unwrap();
        let mut ws = Workspace::new(parent.0.clone()).unwrap();
        assert!(ws
            .course_json("firstcard", 1)
            .unwrap()
            .contains("\"history\":null"));
        assert!(ws
            .file_json("firstcard", "src/main.rs")
            .unwrap()
            .contains("等待完成"));
        assert_eq!(ws.create("firstcard").unwrap_err().status, 409);
        assert!(!learning::check_current(&path, Some(1)).unwrap());
        let code = path.join("src/main.rs");
        let starter = fs::read_to_string(&code).unwrap();
        fs::write(
            code,
            starter.replace(
                "println!(\"等待完成第一张任务卡\");",
                "println!(\"今日任务卡 | {name} | {task}\");",
            ),
        )
        .unwrap();
        assert!(learning::check_current(&path, Some(1)).unwrap());
        assert!(ws
            .course_json("firstcard", 1)
            .unwrap()
            .contains("\"history\":null"));
        fs::write(
            path.join("Cargo.toml"),
            "[package]\nname = \"unrelated\"\n[dependencies]\nname = \"first-rust-card\"\n",
        )
        .unwrap();
        assert!(ws.open("firstcard", &path).is_err());
    }

    #[test]
    fn revision_conflicts_and_execution_inputs_are_tracked_separately_from_notes() {
        let (_parent, mut ws) = workspace();
        ws.create("quizdeck").unwrap();
        let root = ws.project_path("quizdeck").unwrap();
        let path = root.join("src/lib.rs");
        let revision = file_revision(&path, &fs::read(&path).unwrap());
        fs::write(&path, "// 外部修改").unwrap();
        assert_eq!(
            ws.save("quizdeck", "src/lib.rs", "// 我的草稿", &revision)
                .unwrap_err()
                .status,
            409
        );
        assert_eq!(fs::read_to_string(path).unwrap(), "// 外部修改");
        let before = ws.source_digest("quizdeck").unwrap();
        let log = fs::read(root.join("LEARNING_LOG.md")).unwrap();
        ws.save(
            "quizdeck",
            "LEARNING_LOG.md",
            "我的复盘",
            &file_revision(&root.join("LEARNING_LOG.md"), &log),
        )
        .unwrap();
        assert_eq!(ws.source_digest("quizdeck").unwrap(), before);
        fs::write(root.join("samples/answers.txt"), "A\nB\n").unwrap();
        let sample_changed = ws.source_digest("quizdeck").unwrap();
        assert_ne!(before, sample_changed);
        fs::write(root.join("Cargo.toml"), "[workspace]\n").unwrap();
        assert_ne!(sample_changed, ws.source_digest("quizdeck").unwrap());
        for bad in [
            "src/../Cargo.toml",
            "Cargo.toml",
            "/src/main.rs",
            "src\\main.rs",
            "tests/milestones.rs",
            "./src/lib.rs",
            "src//lib.rs",
        ] {
            assert!(ws.file_json("quizdeck", bad).is_err(), "{bad}");
        }
        fs::write(root.join("samples/answers.txt"), [0xff]).unwrap();
        assert!(ws.file_json("quizdeck", "samples/answers.txt").is_err());
    }

    #[test]
    fn file_reads_return_the_project_digest_after_external_edits() {
        let (_parent, mut ws) = workspace();
        ws.create("unitlab").unwrap();
        let root = ws.project_path("unitlab").unwrap();
        let source_path = root.join("src/lib.rs");
        let old_digest = ws.source_digest("unitlab").unwrap();
        let log_path = root.join("LEARNING_LOG.md");
        let log_revision = file_revision(&log_path, &fs::read(&log_path).unwrap());
        let edited = "// 外部修改";
        fs::write(&source_path, edited).unwrap();

        let current = ws.source_digest("unitlab").unwrap();
        assert_ne!(old_digest, current);
        let digest_field = format!("\"source_digest\":\"{current}\"");
        let log = ws.file_json("unitlab", "LEARNING_LOG.md").unwrap();
        assert!(log.contains(&digest_field));
        assert!(log.contains(&format!("\"revision\":\"{log_revision}\"")));

        let source = ws.file_json("unitlab", "src/lib.rs").unwrap();
        assert!(source.contains(&digest_field));
        assert!(source.contains(&format!("\"content\":{}", json_string(edited))));
        let new_revision = file_revision(&source_path, edited.as_bytes());
        assert!(source.contains(&format!("\"revision\":\"{new_revision}\"")));
    }

    #[test]
    fn real_studio_history_uses_the_same_snapshot_as_cli_navigation() {
        let (_parent, mut ws) = workspace();
        ws.create("unitlab").unwrap();
        let root = ws.project_path("unitlab").unwrap();
        let file = root.join("src/lib.rs");
        let starter = fs::read_to_string(&file).unwrap();
        fs::write(
            &file,
            starter.replace("let _ = celsius;\n    0.0", "celsius * 1.8 + 32.0"),
        )
        .unwrap();
        assert!(!learning::check_current(&root, Some(2)).unwrap());
        let card = ws.course_json("unitlab", 2).unwrap();
        assert!(card.contains(
            "\"passed_steps\":1,\"upto_step\":2,\"same_source\":true,\"compilation_failed\":false"
        ));
        assert!(card.contains("\"suggested_step\":2"));
        fs::write(file, starter).unwrap();
        assert!(ws
            .course_json("unitlab", 1)
            .unwrap()
            .contains("\"same_source\":false"));
    }

    #[test]
    fn a_draft_from_another_tab_cannot_overwrite_an_identical_project() {
        let (parent, mut ws) = workspace();
        ws.create("unitlab").unwrap();
        let original = ws.file_json("unitlab", "src/lib.rs").unwrap();
        let revision = original
            .split_once("\"revision\":\"")
            .unwrap()
            .1
            .split('"')
            .next()
            .unwrap();
        let digest = ws.source_digest("unitlab").unwrap();
        let other = parent.0.join("另一份");
        studio::init("unitlab", &other).unwrap();
        ws.open("unitlab", &other).unwrap();
        let target = other.join("src/lib.rs");
        let before = fs::read(&target).unwrap();
        assert_eq!(
            ws.save("unitlab", "src/lib.rs", "// old tab", revision)
                .unwrap_err()
                .status,
            409
        );
        assert_eq!(fs::read(target).unwrap(), before);
        assert_ne!(ws.source_digest("unitlab").unwrap(), digest);
    }

    #[test]
    fn save_enforces_total_source_size_before_touching_disk() {
        let (_parent, mut ws) = workspace();
        ws.create("unitlab").unwrap();
        let path = ws.project_path("unitlab").unwrap().join("src/lib.rs");
        let original = fs::read(&path).unwrap();
        let err = ws
            .save(
                "unitlab",
                "src/lib.rs",
                &"x".repeat(MAX_BYTES),
                &file_revision(&path, &original),
            )
            .unwrap_err();
        assert_eq!(err.status, 413);
        assert_eq!(fs::read(path).unwrap(), original);
    }

    #[test]
    fn traversal_has_bounded_depth_and_file_count() {
        let (_parent, mut ws) = workspace();
        ws.create("unitlab").unwrap();
        let src = ws.project_path("unitlab").unwrap().join("src");
        for n in 0..MAX_FILES - 2 {
            fs::write(src.join(format!("part{n}.rs")), "").unwrap();
        }
        assert!(ws.file_json("unitlab", "src/main.rs").is_ok());
        let extra = src.join("extra.rs");
        fs::write(&extra, "").unwrap();
        assert_eq!(ws.source_digest("unitlab").unwrap_err().status, 413);
        fs::remove_file(extra).unwrap();
        let mut nested = src;
        for _ in 0..MAX_DEPTH {
            nested = nested.join("f");
        }
        fs::create_dir_all(&nested).unwrap();
        assert!(ws.source_digest("unitlab").is_ok());
        fs::create_dir(nested.join("extra")).unwrap();
        assert!(ws.source_digest("unitlab").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn links_in_files_directories_and_project_ancestors_are_refused() {
        use std::os::unix::fs::symlink;
        let (parent, mut ws) = workspace();
        ws.create("unitlab").unwrap();
        let path = ws.project_path("unitlab").unwrap();
        let elsewhere = parent.0.join("external");
        fs::create_dir(&elsewhere).unwrap();
        fs::write(elsewhere.join("data.rs"), "external").unwrap();
        symlink(&elsewhere, path.join("src/other")).unwrap();
        assert!(ws.file_json("unitlab", "src/other/data.rs").is_err());
        fs::remove_file(path.join("src/other")).unwrap();
        symlink(elsewhere.join("data.rs"), path.join("src/linked.rs")).unwrap();
        assert!(ws
            .save(
                "unitlab",
                "src/linked.rs",
                "changed",
                &file_revision(&path.join("src/linked.rs"), b"external")
            )
            .is_err());
        assert_eq!(fs::read(elsewhere.join("data.rs")).unwrap(), b"external");
        fs::remove_file(path.join("src/linked.rs")).unwrap();
        symlink(&parent.0, parent.0.join("alias")).unwrap();
        assert!(ws
            .open("unitlab", &parent.0.join("alias/my-unitlab"))
            .is_err());
    }
}
