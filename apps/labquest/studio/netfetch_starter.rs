//! NetFetch: loopback-only HTTP/1.1 downloader (synchronous foundation, NOT async/TLS).
//! The deliberately limited protocol boundary is visible in tests and course notes.
// 学员完成第 3 步才会用到 OpenOptions；入门阶段先不显示无关警告。
#[allow(unused_imports)]
use std::{fs::{self, OpenOptions}, io::{Read, Write}, net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpStream}, path::{Path, PathBuf}, time::Duration};

pub const MAX_BODY: usize = 2 * 1024 * 1024;
const MAX_HEADERS: usize = 16 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalUrl { pub host: String, pub port: u16, pub path: String }
impl LocalUrl {
    fn address(&self) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), self.port)
    }
}

pub fn parse_url(text: &str) -> Result<LocalUrl, String> {
    // LESSON 1 BEGIN
    // TODO(1): 拆解本机 URL；先做合法地址判断，再处理端口与路径。
    let _ = text;
    Err("请完成第 1 步：解析本地 URL".into())
    // LESSON 1 END
}

pub fn parse_response(bytes: &[u8]) -> Result<Vec<u8>, String> {
    // LESSON 2 BEGIN
    // TODO(2): 检查 HTTP 200、Content-Length 和大小限制。
    let _ = bytes;
    Err("请完成第 2 步：解析 HTTP 响应".into())
    // LESSON 2 END
}

pub fn request(url: &str) -> Result<Vec<u8>, String> {
    let target = parse_url(url)?;
    // DNS is not consulted: 'localhost' always connects to a literal loopback address.
    let addr=if target.host=="[::1]" {SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST),target.port)} else {target.address()};
    let mut stream=TcpStream::connect_timeout(&addr,Duration::from_secs(3)).map_err(|e|format!("连接 {addr} 失败：{e}"))?;
    stream.set_read_timeout(Some(Duration::from_secs(3))).map_err(|e|e.to_string())?;
    stream.set_write_timeout(Some(Duration::from_secs(3))).map_err(|e|e.to_string())?;
    let wire=format!("GET {} HTTP/1.1\r\nHost: {}:{}\r\nConnection: close\r\nAccept: */*\r\n\r\n",target.path,target.host,target.port);
    stream.write_all(wire.as_bytes()).map_err(|e|format!("发送请求失败：{e}"))?;
    let mut received=Vec::new();
    stream.take((MAX_BODY+MAX_HEADERS+4096) as u64).read_to_end(&mut received).map_err(|e|format!("读取响应失败：{e}"))?;
    if received.len()>=MAX_BODY+MAX_HEADERS+4096 {return Err("响应超过安全读取上限".into());}
    parse_response(&received)
}

pub fn download(url: &str, output: &Path) -> Result<usize, String> {
    // LESSON 3 BEGIN
    // TODO(3): 下载之前先验证目标不覆盖，失败不能留下不完整文件。
    let _ = (url, output);
    Err("请完成第 3 步：保存真实下载文件".into())
    // LESSON 3 END
}

fn file_name(name:&str)->Result<(),String>{
    if name.is_empty()||name.len()>64||matches!(name,"."|"..")||!name.bytes().all(|b|b.is_ascii_alphanumeric()||matches!(b,b'.'|b'-'|b'_')) {return Err("输出文件名只能使用 ASCII 字母、数字、点、下划线、短横线，且不能为 . 或 ..".into());}
    Ok(())
}
struct Stage(PathBuf);
impl Drop for Stage{fn drop(&mut self){let _=fs::remove_dir_all(&self.0);}}
/// Atomically publishes a multi-file result only if all requests succeed.
pub fn batch(manifest: &str, destination: &Path)->Result<Vec<(String,usize)>,String>{
    let mut jobs=Vec::new();let mut seen=std::collections::HashSet::new();
    for (i,line) in manifest.lines().enumerate(){
        if line.trim().is_empty(){continue;}
        let (name,url)=line.split_once('\t').ok_or_else(||format!("清单第 {} 行需要 文件名<TAB>URL",i+1))?;
        file_name(name)?; parse_url(url)?;
        if !seen.insert(name.to_string()){return Err("清单含重复输出文件名".into());}
        if jobs.len()==31{return Err("批量最多 32 个文件".into());}
        jobs.push((name.to_owned(),url.to_owned()));
    }
    if jobs.is_empty(){return Err("清单没有任何任务".into());}
    let parent=destination.parent().filter(|p|!p.as_os_str().is_empty()).unwrap_or_else(||Path::new("."));
    if !parent.is_dir(){return Err("批量输出的父目录不存在".into());}
    if fs::symlink_metadata(destination).is_ok(){return Err("批量目标目录已存在，不会覆盖".into());}
    let name=destination.file_name().and_then(|s|s.to_str()).ok_or("目标目录名称无效")?;
    let stage=parent.join(format!(".{name}.netfetch-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(|e|e.to_string())?.as_nanos()));
    fs::create_dir(&stage).map_err(|e|e.to_string())?;
    let guard=Stage(stage);
    let mut report=Vec::new();
    for (name,url) in jobs {let n=download(&url,&guard.0.join(&name))?;report.push((name,n));}
    if fs::symlink_metadata(destination).is_ok(){return Err("批量目标创建期间被占用，拒绝提交".into());}
    fs::rename(&guard.0,destination).map_err(|e|format!("提交结果失败：{e}"))?;
    Ok(report)
}
