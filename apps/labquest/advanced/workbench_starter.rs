#![allow(dead_code, unused_imports, unused_variables, unused_mut)]
//! Advanced TermGarden: several editor handles, non-owning views, undo and snapshot reports.
//!
//! This is a single-threaded live document. Its owned snapshots, not `Rc<RefCell<_>>`,
//! cross thread boundaries. Edits stage a copy; they do not claim database transactions.
//! A callback's external side effects cannot be rolled back.
//!
//! `Workspace` deliberately is not Send:
//! ```compile_fail,E0277
//! use termgarden::Garden;
//! use garden_workbench::Workspace;
//! let editor = Workspace::new(Garden::default()).unwrap();
//! std::thread::spawn(move || editor.snapshot().unwrap());
//! ```
//! Snapshot borrows cannot escape their owner:
//! ```compile_fail,E0515
//! use termgarden::Garden;
//! use garden_workbench::Snapshot;
//! fn titles<'a>() -> Vec<&'a str> {
//!     let snapshot = Snapshot { revision: 0, garden: Garden::default() };
//!     snapshot.select(|_| true).iter().map(|t| t.title.as_str()).collect()
//! }
//! ```
//! Owned snapshots can be sent and shared:
//! ```
//! use termgarden::{Garden, BoardView};
//! use garden_workbench::{Workspace, Snapshot, spawn_report};
//! use std::sync::Arc;
//! fn send_sync<T: Send + Sync>() {}
//! send_sync::<Snapshot>();
//! let editor = Workspace::new(Garden::default()).unwrap();
//! let report = spawn_report(Arc::new(editor.snapshot().unwrap()), BoardView);
//! assert!(report.join().unwrap().contains("TermGarden"));
//! ```
use termgarden::{Garden, Renderer, Task};
use std::cell::{Ref, RefCell};
use std::fmt;
use std::rc::{Rc, Weak};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

pub const HISTORY_LIMIT: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditError {
    Busy,
    Closed,
    Invalid(String),
    Conflict,
    NoUndo,
    RevisionExhausted,
}
impl fmt::Display for EditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Busy => write!(f, "数据正在被借用；先结束读取作用域，再执行编辑"),
            Self::Closed => write!(f, "工作区已经关闭；视图没有数据所有权"),
            Self::Invalid(reason) => write!(f, "未提交：{reason}"),
            Self::Conflict => write!(f, "编辑期间版本已变化；拒绝用旧草稿覆盖新状态"),
            Self::NoUndo => write!(f, "没有可撤销的编辑"),
            Self::RevisionExhausted => write!(f, "版本号已耗尽"),
        }
    }
}
impl std::error::Error for EditError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub revision: u64,
    pub garden: Garden,
}
impl Snapshot {
    /// The result borrows this snapshot, never a temporary RefCell guard.
    /// `Fn` permits repeated selection without mutating the predicate's captures.
    pub fn select<P>(&self, predicate: P) -> Vec<&Task>
    where
        P: Fn(&Task) -> bool,
    {
    todo!("A1：返回借用当前快照的任务；不要 clone Task")
}
}

struct Session {
    garden: Garden,
    revision: u64,
    undo: Vec<Garden>,
}

/// Cloning the handle shares the document; it does not clone the tasks.
#[derive(Clone)]
pub struct Workspace {
    inner: Rc<RefCell<Session>>,
}
/// A view borrows ownership via Weak; it does not keep a closed document alive.
pub struct View {
    document: Weak<RefCell<Session>>,
    renderer: Box<dyn Renderer>,
}

fn validate(garden: &Garden) -> Result<(), EditError> {
    termgarden::storage::encode(garden).map_err(EditError::Invalid)?;
    if garden.next_id == 0 || garden.tasks.iter().any(|t| t.id >= garden.next_id) {
        return Err(EditError::Invalid(
            "next_id 必须大于所有现有任务编号".into(),
        ));
    }
    if garden
        .selected
        .is_some_and(|id| !garden.tasks.iter().any(|t| t.id == id))
    {
        return Err(EditError::Invalid("选择的任务编号不存在".into()));
    }
    Ok(())
}

impl Workspace {
    pub fn new(garden: Garden) -> Result<Self, EditError> {
    todo!("A1：校验 Garden，再建立共享所有权")
}
    /// A runtime-checked borrow. Keeping this guard alive intentionally prevents editing.
    pub fn borrow(&self) -> Result<Ref<'_, Garden>, EditError> {
    todo!("A1：用 try_borrow 和 Ref::map 保留借用守卫")
}
    pub fn snapshot(&self) -> Result<Snapshot, EditError> {
    todo!("A1：创建带版本号的拥有型快照")
}
    pub fn view(&self, renderer: Box<dyn Renderer>) -> View {
    todo!("A1：用非拥有引用连接视图与文档")
}

    /// FnOnce allows a callback to consume owned input. At most one callback invocation.
    /// No RefCell guard is held while user code runs. A reentrant edit is detected by revision.
    pub fn edit<T, F>(&self, change: F) -> Result<T, EditError>
    where
        F: FnOnce(&mut Garden) -> Result<T, String>,
    {
    todo!("A2：释放借用后执行一次闭包；校验草稿与版本，再提交和记录撤销")
}
    /// Fn for selecting; FnMut for a renamer that numbers the selected rows.
    pub fn rename_where<P, F>(&self, predicate: P, mut rename: F) -> Result<usize, EditError>
    where
        P: Fn(&Task) -> bool,
        F: FnMut(&Task) -> String,
    {
    todo!("A2：选择、可变闭包和整批提交")
}
    pub fn undo(&self) -> Result<(), EditError> {
    todo!("A2：恢复最近状态，版本号仍递增")
}
}
impl View {
    pub fn draw(&self) -> Result<String, EditError> {
    todo!("A1：处理关闭状态；释放借用后调用 renderer")
}
}
/// Static dispatch with a concrete renderer, or dynamic dispatch with `&dyn Renderer`.
pub fn render_with<R: Renderer + ?Sized>(snapshot: &Snapshot, renderer: &R) -> String {
    todo!("A1：用同一个接口支持具体类型与 trait 对象")
}
/// Only an owned, immutable snapshot crosses threads. The renderer is moved, not shared.
/// A report describes this captured revision; it is not a continuously updated live view.
pub fn spawn_report<R>(snapshot: Arc<Snapshot>, renderer: R) -> JoinHandle<String>
where
    R: Renderer + Send + 'static,
{
    todo!("A3：移动不可变快照与渲染器进入线程，返回 JoinHandle")
}

/// Closure and thread contracts; failures are expected compiler diagnostics, not runtime errors.
/// A mutable capture requires FnMut, not Fn:
/// ```compile_fail,E0594
/// fn repeated<F: Fn()>(f:F) { f(); f(); }
/// let mut calls=0;
/// repeated(|| calls+=1);
/// ```
/// Moving out of a captured value is FnOnce; invoking twice is an error:
/// ```compile_fail,E0382
/// let payload=String::from("one-time");
/// let consume=move || payload;
/// let _first=consume();
/// let _second=consume();
/// ```
/// Arc makes reference counting thread safe, not arbitrary inner mutation:
/// ```compile_fail,E0277
/// use std::{cell::RefCell, sync::Arc};
/// let value=Arc::new(RefCell::new(0));
/// std::thread::spawn(move || *value.borrow_mut()+=1);
/// ```
/// `move` captures by value, but does not necessarily consume that value on each call:
/// ```
/// fn twice<F:Fn()->usize>(f:F)->usize {f()+f()}
/// let name=String::from("Rust");
/// assert_eq!(twice(move || name.len()),8);
/// ```
pub mod type_contracts {}
