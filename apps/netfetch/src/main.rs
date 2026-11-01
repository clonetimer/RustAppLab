use std::{
    env, fs,
    io::{Read, Write},
    net::TcpListener,
    path::PathBuf,
    thread,
};
fn demo(path: PathBuf) -> Result<(), String> {
    fs::create_dir(&path).map_err(|e| format!("创建演示目录失败（不能覆盖）：{e}"))?;
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    let server = thread::spawn(move || -> Result<(), String> {
        for _ in 0..2 {
            let (mut client, _) = listener.accept().map_err(|e| e.to_string())?;
            client
                .set_read_timeout(Some(std::time::Duration::from_secs(3)))
                .map_err(|e| e.to_string())?;
            let mut request = [0u8; 1024];
            let n = client.read(&mut request).map_err(|e| e.to_string())?;
            let line = String::from_utf8_lossy(&request[..n]);
            let body = if line.starts_with("GET /intro HTTP/") {
                "你好，Rust 网络！\n"
            } else {
                "第 1 份本地网络实验\n"
            };
            let wire = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            client
                .write_all(wire.as_bytes())
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    });
    let manifest = format!(
        "intro.txt\thttp://127.0.0.1:{port}/intro\nnotes.txt\thttp://127.0.0.1:{port}/notes\n"
    );
    fs::write(path.join("downloads.tsv"), &manifest).map_err(|e| e.to_string())?;
    let outcome = netfetch::batch(&manifest, &path.join("files"));
    server
        .join()
        .map_err(|_| "本地演示服务器线程意外结束".to_string())??;
    let files = outcome?;
    fs::write(
        path.join("README.md"),
        format!(
            "# NetFetch 实际成果\n\n本机 HTTP 请求，逐个下载并原子提交目录。\n\n{:?}\n",
            files
        ),
    )
    .map_err(|e| e.to_string())?;
    println!(
        "NetFetch demo: {} 个文件保存到 {}",
        files.len(),
        path.join("files").display()
    );
    Ok(())
}
fn run() -> Result<(), String> {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    let arg = |i: usize| {
        args.get(i)
            .and_then(|s| s.to_str())
            .ok_or_else(|| "参数必须是 UTF-8".to_string())
    };
    match args.first().and_then(|s|s.to_str()).unwrap_or("help"){
        "help"|"--help" if args.len()<=1=>{println!("NetFetch · 本机 HTTP 下载器（同步基础版，不支持 HTTPS/chunked）\n  demo <不存在的目录>\n  fetch <http://127.0.0.1:端口/路径> <新文件>\n  batch <任务.tsv> <不存在的目录>\n限制：仅连接本机，最大响应 2 MiB，不覆盖已有文件；异步版未实现。");Ok(())}
        "demo" if args.len()==2=>demo(PathBuf::from(&args[1])),
        "fetch" if args.len()==3=>{let n=netfetch::download(arg(1)?,&PathBuf::from(&args[2]))?;println!("saved={n}");Ok(())}
        "batch" if args.len()==3=>{let manifest=fs::read_to_string(&args[1]).map_err(|e|e.to_string())?;let result=netfetch::batch(&manifest,&PathBuf::from(&args[2]))?;for (name,bytes) in result{println!("{name}\t{bytes}");}Ok(())}
        _=>Err("用法：netfetch help | demo <新目录> | fetch <本机URL> <新文件> | batch <清单> <新目录>".into()),
    }
}
fn main() {
    if let Err(e) = run() {
        eprintln!("NetFetch 错误：{e}");
        std::process::exit(2)
    }
}
