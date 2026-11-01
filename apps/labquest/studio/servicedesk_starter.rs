//! ServiceDesk: small, synchronous loopback work-order app. Intentionally no async framework yet.
use std::{fs::{self,OpenOptions},io::{Read,Write},path::Path};
const MAGIC:&str="SERVICEDESK1\n";
const MAX_LOG:usize=2*1024*1024;
// 这些基础设施在后续里程碑使用；第 1 步尚未引用时不产生警告。
#[allow(dead_code)]
const MAX_TICKETS:usize=1000;
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct Ticket{pub id:u64,pub title:String,pub done:bool}
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum Action{Add(String),Close(u64)}

pub fn validate_title(text:&str)->Result<String,String>{
    // LESSON 1 BEGIN
    // TODO(1): trim + 字符边界 + 控制字符检测。
    let _ = text;
    Err("请完成第 1 步：验证标题".into())
    // LESSON 1 END
}

pub fn apply(tickets:&mut Vec<Ticket>,action:Action)->Result<Ticket,String>{
    // LESSON 2 BEGIN
    // TODO(2): Add 与 Close 必须修改状态，错误输入不能污染已有数据。
    let _ = (tickets, action);
    Err("请完成第 2 步：状态转换".into())
    // LESSON 2 END
}

#[allow(dead_code)]
fn json_string(s:&str)->String{
    let mut out=String::from("\"");
    for c in s.chars(){match c{
        '"'=>out.push_str("\\\""),'\\'=>out.push_str("\\\\"),'\n'=>out.push_str("\\n"),'\r'=>out.push_str("\\r"),'\t'=>out.push_str("\\t"),
        '<'=>out.push_str("\\u003c"),'>' =>out.push_str("\\u003e"),'&'=>out.push_str("\\u0026"),
        x if x<' '=>out.push_str(&format!("\\u{:04x}",x as u32)),x=>out.push(x),
    }}out.push('"');out
}
pub fn render_json(tickets:&[Ticket])->String{
    // LESSON 3 BEGIN
    // TODO(3): 输出规范 JSON；注意引号、反斜杠、HTML 特殊字符。
    let _ = tickets;
    "{}".to_string()
    // LESSON 3 END
}

fn hex_encode(s:&str)->String{const DIGITS:&[u8;16]=b"0123456789abcdef";let mut out=String::with_capacity(s.len()*2);for b in s.bytes(){out.push(DIGITS[(b>>4)as usize] as char);out.push(DIGITS[(b&15)as usize] as char);}out}
fn hex_decode(s:&str)->Result<String,String>{if s.len()%2!=0{return Err("工单存储含奇数位十六进制数据".into());}let mut buf=Vec::new();for p in s.as_bytes().chunks_exact(2){let hi=(p[0] as char).to_digit(16).ok_or("工单存储包含非法十六进制")?;let lo=(p[1] as char).to_digit(16).ok_or("工单存储包含非法十六进制")?;buf.push(((hi<<4)|lo)as u8);}String::from_utf8(buf).map_err(|_|"工单存储包含非法 UTF-8".into())}
/// The append journal is teaching-grade: detects torn records but does not repair them automatically.
pub fn init(path:&Path)->Result<(),String>{
    let mut f=OpenOptions::new().write(true).create_new(true).open(path).map_err(|e|format!("数据库不能创建（禁止覆盖）：{e}"))?;
    f.write_all(MAGIC.as_bytes()).and_then(|_|f.sync_all()).map_err(|e|e.to_string())
}
pub fn load(path:&Path)->Result<Vec<Ticket>,String>{
    let f=OpenOptions::new().read(true).open(path).map_err(|e|format!("打开工单存储失败：{e}"))?;
    let mut raw=Vec::new();
    f.take((MAX_LOG+1)as u64).read_to_end(&mut raw).map_err(|e|e.to_string())?;
    if raw.len()>MAX_LOG {return Err("工单日志超过 2 MiB".into());}
    let text=std::str::from_utf8(&raw).map_err(|_|"工单日志不是 UTF-8")?;
    if !text.starts_with(MAGIC)||!text.ends_with('\n'){return Err("工单日志标记错误或末尾记录不完整；禁止自动修复".into());}
    let mut rows=Vec::new();
    for line in text[MAGIC.len()..].lines(){
        let parts=line.split('\t').collect::<Vec<_>>();
        match parts.as_slice(){
            ["A",id,encoded]=>{
                let n=id.parse::<u64>().map_err(|_|"非法工单编号")?;
                let title=hex_decode(encoded)?;
                if rows.last().map_or(n!=1,|t:&Ticket|t.id.checked_add(1)!=Some(n)) {return Err("工单记录编号不连续".into());}
                let t=apply(&mut rows,Action::Add(title))?;
                if t.id!=n{return Err("工单编号与日志不匹配".into());}
            }
            ["C",id]=>{let n=id.parse::<u64>().map_err(|_|"非法关闭记录编号")?;apply(&mut rows,Action::Close(n))?;}
            _=>return Err("工单日志包含非法或空行".into()),
        }
    }
    Ok(rows)
}
pub fn persist(path:&Path, action:Action)->Result<Ticket,String>{
    let mut tickets=load(path)?;
    let ticket=apply(&mut tickets,action.clone())?;
    let line=match action {Action::Add(_)=>format!("A\t{}\t{}\n",ticket.id,hex_encode(&ticket.title)),Action::Close(_)=>format!("C\t{}\n",ticket.id)};
    // A single writer process is assumed; another concurrent server is unsupported.
    let mut file=OpenOptions::new().append(true).open(path).map_err(|e|e.to_string())?;
    file.write_all(line.as_bytes()).and_then(|_|file.sync_data()).map_err(|e|format!("写入或同步日志失败，重新运行前先检查日志完整性：{e}"))?;
    Ok(ticket)
}
pub fn ensure_store(path:&Path)->Result<(),String>{let m=fs::symlink_metadata(path).map_err(|e|format!("需要先 init 创建工单存储：{e}"))?;if !m.is_file() || m.file_type().is_symlink(){return Err("工单存储必须是普通文件".into());}load(path).map(|_|())}
