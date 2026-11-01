//! ServiceDesk: small, synchronous loopback work-order app. Intentionally no async framework yet.
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::Path,
};
const MAGIC: &str = "SERVICEDESK1\n";
const MAX_LOG: usize = 2 * 1024 * 1024;
const MAX_TICKETS: usize = 1000;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ticket {
    pub id: u64,
    pub title: String,
    pub done: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Add(String),
    Close(u64),
}

pub fn validate_title(text: &str) -> Result<String, String> {
    // LESSON 1 BEGIN
    if text.chars().any(char::is_control) {
        return Err("标题不能包含控制字符".into());
    }
    let title = text.trim();
    let n = title.chars().count();
    if !(1..=120).contains(&n) || title.chars().any(char::is_control) {
        return Err("工单标题必须有 1–120 个字符，不能包含换行或控制字符".into());
    }
    Ok(title.to_owned())
    // LESSON 1 END
}

pub fn apply(tickets: &mut Vec<Ticket>, action: Action) -> Result<Ticket, String> {
    // LESSON 2 BEGIN
    match action {
        Action::Add(input) => {
            let title = validate_title(&input)?;
            if tickets.len() >= MAX_TICKETS {
                return Err("最多 1000 张工单".into());
            }
            let next = tickets.last().map_or(Ok(1), |t| {
                t.id.checked_add(1)
                    .ok_or_else(|| "工单编号耗尽".to_string())
            })?;
            let ticket = Ticket {
                id: next,
                title,
                done: false,
            };
            tickets.push(ticket.clone());
            Ok(ticket)
        }
        Action::Close(id) => {
            let ticket = tickets
                .iter_mut()
                .find(|t| t.id == id)
                .ok_or("工单编号不存在")?;
            if ticket.done {
                return Err("工单已关闭，不允许重复操作".into());
            }
            ticket.done = true;
            Ok(ticket.clone())
        }
    }
    // LESSON 2 END
}

fn json_string(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '<' => out.push_str("\\u003c"),
            '>' => out.push_str("\\u003e"),
            '&' => out.push_str("\\u0026"),
            x if x < ' ' => out.push_str(&format!("\\u{:04x}", x as u32)),
            x => out.push(x),
        }
    }
    out.push('"');
    out
}
pub fn render_json(tickets: &[Ticket]) -> String {
    // LESSON 3 BEGIN
    let mut s = String::from("{\"tickets\":[");
    for (i, t) in tickets.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!(
            "{{\"id\":{},\"title\":{},\"done\":{}}}",
            t.id,
            json_string(&t.title),
            t.done
        ));
    }
    s.push_str(&format!("],\"count\":{}}}", tickets.len()));
    s
    // LESSON 3 END
}

fn hex_encode(s: &str) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(s.len() * 2);
    for b in s.bytes() {
        out.push(DIGITS[(b >> 4) as usize] as char);
        out.push(DIGITS[(b & 15) as usize] as char);
    }
    out
}
fn hex_decode(s: &str) -> Result<String, String> {
    if s.len() % 2 != 0 {
        return Err("工单存储含奇数位十六进制数据".into());
    }
    let mut buf = Vec::new();
    for p in s.as_bytes().chunks_exact(2) {
        let hi = (p[0] as char)
            .to_digit(16)
            .ok_or("工单存储包含非法十六进制")?;
        let lo = (p[1] as char)
            .to_digit(16)
            .ok_or("工单存储包含非法十六进制")?;
        buf.push(((hi << 4) | lo) as u8);
    }
    String::from_utf8(buf).map_err(|_| "工单存储包含非法 UTF-8".into())
}
/// The append journal is teaching-grade: detects torn records but does not repair them automatically.
pub fn init(path: &Path) -> Result<(), String> {
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("数据库不能创建（禁止覆盖）：{e}"))?;
    f.write_all(MAGIC.as_bytes())
        .and_then(|_| f.sync_all())
        .map_err(|e| e.to_string())
}
pub fn load(path: &Path) -> Result<Vec<Ticket>, String> {
    let f = OpenOptions::new()
        .read(true)
        .open(path)
        .map_err(|e| format!("打开工单存储失败：{e}"))?;
    let mut raw = Vec::new();
    f.take((MAX_LOG + 1) as u64)
        .read_to_end(&mut raw)
        .map_err(|e| e.to_string())?;
    if raw.len() > MAX_LOG {
        return Err("工单日志超过 2 MiB".into());
    }
    let text = std::str::from_utf8(&raw).map_err(|_| "工单日志不是 UTF-8")?;
    if !text.starts_with(MAGIC) || !text.ends_with('\n') {
        return Err("工单日志标记错误或末尾记录不完整；禁止自动修复".into());
    }
    let mut rows = Vec::new();
    for line in text[MAGIC.len()..].lines() {
        let parts = line.split('\t').collect::<Vec<_>>();
        match parts.as_slice() {
            ["A", id, encoded] => {
                let n = id.parse::<u64>().map_err(|_| "非法工单编号")?;
                let title = hex_decode(encoded)?;
                if rows
                    .last()
                    .map_or(n != 1, |t: &Ticket| t.id.checked_add(1) != Some(n))
                {
                    return Err("工单记录编号不连续".into());
                }
                let t = apply(&mut rows, Action::Add(title))?;
                if t.id != n {
                    return Err("工单编号与日志不匹配".into());
                }
            }
            ["C", id] => {
                let n = id.parse::<u64>().map_err(|_| "非法关闭记录编号")?;
                apply(&mut rows, Action::Close(n))?;
            }
            _ => return Err("工单日志包含非法或空行".into()),
        }
    }
    Ok(rows)
}
pub fn persist(path: &Path, action: Action) -> Result<Ticket, String> {
    let mut tickets = load(path)?;
    let ticket = apply(&mut tickets, action.clone())?;
    let line = match action {
        Action::Add(_) => format!("A\t{}\t{}\n", ticket.id, hex_encode(&ticket.title)),
        Action::Close(_) => format!("C\t{}\n", ticket.id),
    };
    // A single writer process is assumed; another concurrent server is unsupported.
    let mut file = OpenOptions::new()
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    file.write_all(line.as_bytes())
        .and_then(|_| file.sync_data())
        .map_err(|e| format!("写入或同步日志失败，重新运行前先检查日志完整性：{e}"))?;
    Ok(ticket)
}
pub fn ensure_store(path: &Path) -> Result<(), String> {
    let m = fs::symlink_metadata(path).map_err(|e| format!("需要先 init 创建工单存储：{e}"))?;
    if !m.is_file() || m.file_type().is_symlink() {
        return Err("工单存储必须是普通文件".into());
    }
    load(path).map(|_| ())
}
