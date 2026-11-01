//! A line-driven task garden: parse input, update state, render without side effects.
//! Persistence and terminal I/O live outside the three learner functions.
pub mod storage;
pub mod workbench;

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
        Self {
            tasks: Vec::new(),
            selected: None,
            only_open: false,
            next_id: 1,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Add(String),
    Next,
    Prev,
    Toggle,
    Filter,
    Quit,
}

pub fn valid_title(title: &str) -> Result<(), String> {
    if title.trim().is_empty() || title.chars().count() > 80 || title.chars().any(char::is_control)
    {
        return Err("标题必须有 1–80 个字符，且不能包含 Tab、换行或控制字符".into());
    }
    Ok(())
}
pub fn visible_ids(garden: &Garden) -> Vec<u64> {
    garden
        .tasks
        .iter()
        .filter(|t| !garden.only_open || !t.done)
        .map(|t| t.id)
        .collect()
}

/// Stage 1: parse complete lines; unknown commands are errors, never implicit actions.
pub fn parse_command(input: &str) -> Result<Event, String> {
    let line = input.trim();
    if let Some(title) = line.strip_prefix("add ") {
        let title = title.trim();
        valid_title(title)?;
        return Ok(Event::Add(title.to_owned()));
    }
    match line {
        "n" | "next" => Ok(Event::Next),
        "p" | "prev" => Ok(Event::Prev),
        "x" | "toggle" => Ok(Event::Toggle),
        "f" | "filter" => Ok(Event::Filter),
        "q" | "quit" => Ok(Event::Quit),
        _ => {
            Err("命令：add 标题 / n 下一项 / p 上一项 / x 完成或撤销 / f 仅未完成 / q 退出".into())
        }
    }
}

/// Stage 2: one event, one transition. Errors must leave state unchanged.
/// The cursor is a stable task ID, not a filtered list offset.
pub fn apply_event(garden: &mut Garden, event: Event) -> Result<bool, String> {
    match event {
        Event::Quit => return Ok(false),
        Event::Add(title) => {
            valid_title(&title)?;
            if garden.tasks.len() >= 1000 {
                return Err("最多保存 1000 项任务".into());
            }
            let next = garden.next_id.checked_add(1).ok_or("任务编号已耗尽")?;
            garden.tasks.push(Task {
                id: garden.next_id,
                title,
                done: false,
            });
            garden.selected = Some(garden.next_id);
            garden.next_id = next;
        }
        Event::Filter => {
            garden.only_open = !garden.only_open;
        }
        Event::Toggle => {
            let visible = visible_ids(garden);
            let id = garden
                .selected
                .filter(|id| visible.contains(id))
                .ok_or("没有选中可见任务；先 add 一项")?;
            let task = garden
                .tasks
                .iter_mut()
                .find(|t| t.id == id)
                .ok_or("任务不存在")?;
            task.done = !task.done;
        }
        Event::Next | Event::Prev => {
            let ids = visible_ids(garden);
            if !ids.is_empty() {
                let pos = garden
                    .selected
                    .and_then(|id| ids.iter().position(|v| *v == id))
                    .unwrap_or(0);
                let next = if event == Event::Next {
                    (pos + 1) % ids.len()
                } else {
                    (pos + ids.len() - 1) % ids.len()
                };
                garden.selected = Some(ids[next]);
            }
        }
    }
    let ids = visible_ids(garden);
    if !garden.selected.is_some_and(|id| ids.contains(&id)) {
        garden.selected = ids.first().copied();
    }
    Ok(true)
}

/// Stage 3: rendering borrows state; it must not mutate or read stdin/files.
pub fn render(garden: &Garden) -> String {
    let done = garden.tasks.iter().filter(|t| t.done).count();
    let growth = match done {
        0 => "seed",
        1..=2 => "sprout",
        _ => "bloom",
    };
    let mut out = format!(
        "=== TermGarden ===\ndone={done}/{} growth={growth} view={}\n",
        garden.tasks.len(),
        if garden.only_open { "open" } else { "all" }
    );
    let ids = visible_ids(garden);
    for task in garden.tasks.iter().filter(|t| ids.contains(&t.id)) {
        out.push_str(&format!(
            "{} [{}] #{} {}\n",
            if garden.selected == Some(task.id) {
                ">"
            } else {
                " "
            },
            if task.done { "x" } else { " " },
            task.id,
            task.title
        ));
    }
    if ids.is_empty() {
        out.push_str("(empty) 输入 add 我的第一项任务\n");
    }
    out.push_str("add 标题 | n/p 移动 | x 完成/撤销 | f 筛选 | q 退出\n");
    out
}

/// The same model can have multiple views selected at runtime.
pub trait Renderer {
    fn draw(&self, garden: &Garden) -> String;
}
pub struct BoardView;
pub struct CompactView;
impl Renderer for BoardView {
    fn draw(&self, garden: &Garden) -> String {
        render(garden)
    }
}
impl Renderer for CompactView {
    fn draw(&self, garden: &Garden) -> String {
        let done = garden.tasks.iter().filter(|task| task.done).count();
        format!(
            "TermGarden {done}/{} completed; selected={:?}; commands: add/n/p/x/f/q\n",
            garden.tasks.len(),
            garden.selected
        )
    }
}
pub fn renderer(name: &str) -> Result<Box<dyn Renderer>, String> {
    match name {
        "board" => Ok(Box::new(BoardView)),
        "compact" => Ok(Box::new(CompactView)),
        _ => Err("界面只能是 board 或 compact".into()),
    }
}
