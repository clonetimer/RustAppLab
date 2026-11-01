//! A line-driven task garden: parse input, update state, render without side effects.
//! Persistence and terminal I/O live outside the three learner functions.
pub mod storage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    pub id: u64,
    pub title: String,
    pub done: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Garden {
    pub tasks: Vec<Task>,
    pub selected: Option<u64>,
    pub only_open: bool,
    pub next_id: u64,
}
impl Default for Garden {
    fn default() -> Self {
        Self { tasks: Vec::new(), selected: None, only_open: false, next_id: 1 }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event { Add(String), Next, Prev, Toggle, Filter, Quit }

pub fn valid_title(title: &str) -> Result<(), String> {
    if title.trim().is_empty() || title.chars().count() > 80 || title.chars().any(char::is_control) {
        return Err("标题必须有 1–80 个字符，且不能包含 Tab、换行或控制字符".into());
    }
    Ok(())
}
pub fn visible_ids(garden: &Garden) -> Vec<u64> {
    garden.tasks.iter().filter(|t| !garden.only_open || !t.done).map(|t| t.id).collect()
}

/// Stage 1: parse complete lines; unknown commands are errors, never implicit actions.
pub fn parse_command(input: &str) -> Result<Event, String> {
    // TODO(1): 按 LESSON.md 第 1 步完成；其它辅助代码暂时不改。
    // LESSON 1 BEGIN
    let _ = input;
    Err("尚未完成命令解析".into())
    // LESSON 1 END
}

/// Stage 2: one event, one transition. Errors must leave state unchanged.
/// The cursor is a stable task ID, not a filtered list offset.
pub fn apply_event(garden: &mut Garden, event: Event) -> Result<bool, String> {
    // TODO(2): 按 LESSON.md 第 2 步完成；其它辅助代码暂时不改。
    // LESSON 2 BEGIN
    let _ = (garden, event);
    Err("尚未完成状态变化".into())
    // LESSON 2 END
}

/// Stage 3: rendering borrows state; it must not mutate or read stdin/files.
pub fn render(garden: &Garden) -> String {
    // TODO(3): 按 LESSON.md 第 3 步完成；其它辅助代码暂时不改。
    // LESSON 3 BEGIN
    let _ = garden;
    String::from("尚未完成界面渲染\n")
    // LESSON 3 END
}

/// The same model can have multiple views selected at runtime.
pub trait Renderer { fn draw(&self, garden: &Garden) -> String; }
pub struct BoardView;
pub struct CompactView;
impl Renderer for BoardView { fn draw(&self, garden: &Garden) -> String { render(garden) } }
impl Renderer for CompactView {
    fn draw(&self, garden: &Garden) -> String {
        let done = garden.tasks.iter().filter(|task| task.done).count();
        format!("TermGarden {done}/{} completed; selected={:?}; commands: add/n/p/x/f/q\n", garden.tasks.len(), garden.selected)
    }
}
pub fn renderer(name: &str) -> Result<Box<dyn Renderer>, String> {
    match name {
        "board" => Ok(Box::new(BoardView)),
        "compact" => Ok(Box::new(CompactView)),
        _ => Err("界面只能是 board 或 compact".into()),
    }
}
