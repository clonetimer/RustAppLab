use netfetch_c1::{download_batch, Error, Job, Limits};
use std::{ffi::OsString, path::PathBuf, time::Duration};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpListener, sync::watch, task::JoinSet};

const HELP: &str = "NetFetch C1 (literal-loopback HTTP)\n  netfetch-c1 demo <new-output-directory>\n  netfetch-c1 download <new-output-directory> [--concurrency 1..8] [--timeout-ms 5000] [--idle-ms 2000] [--connect-ms 2000] [--max-bytes 1048576] <url>...\nCtrl+C cancels and cleans this batch. Existing output directories are never reused.";
fn value(arg: Option<&OsString>) -> Result<u64, Error> {
    arg.and_then(|x| x.to_str()).and_then(|x| x.parse().ok())
        .ok_or_else(|| Error::Invalid("option requires an unsigned integer".into()))
}
async fn run(args: Vec<OsString>) -> Result<(), Error> {
    if args.len() == 1 && matches!(args[0].to_str(), Some("--help" | "help" | "-h")) {
        println!("{HELP}"); return Ok(());
    }
    let command = args.first().and_then(|x| x.to_str()).unwrap_or("");
    let dest = PathBuf::from(args.get(1).ok_or_else(|| Error::Invalid(HELP.into()))?);
    let mut limits = Limits::default();
    let mut jobs = Vec::new();
    let mut servers = JoinSet::new();
    match command {
        "demo" if args.len() == 2 => {
            let listener = TcpListener::bind("127.0.0.1:0").await?;
            let address = listener.local_addr()?;
            for i in 1..=3 {
                jobs.push(Job {name: format!("{i:03}.bin"), url: format!("http://{address}/{i}")});
            }
            servers.spawn(async move {
                let mut connections = JoinSet::new();
                for _ in 0..3 {
                    let (mut stream, _) = listener.accept().await?;
                    connections.spawn(async move {
                        let mut request = [0; 4096];
                        let read = stream.read(&mut request).await?;
                        let first = String::from_utf8_lossy(&request[..read]).contains("GET /1 ");
                        tokio::time::sleep(Duration::from_millis(if first {80} else {5})).await;
                        let body = "Rust 异步下载\n";
                        let response = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
                        stream.write_all(response.as_bytes()).await
                    });
                }
                while let Some(result) = connections.join_next().await {
                    result.map_err(std::io::Error::other)??;
                }
                Ok::<(), std::io::Error>(())
            });
        }
        "download" => {
            let mut i = 2;
            while i < args.len() {
                let text = args[i].to_str().ok_or_else(|| Error::Invalid("URL/option must be UTF-8".into()))?;
                if text.starts_with("--") {
                    let n = value(args.get(i+1))?;
                    match text {
                        "--concurrency" => limits.concurrency = usize::try_from(n).map_err(|e| Error::Invalid(e.to_string()))?,
                        "--max-bytes" => limits.max_bytes = usize::try_from(n).map_err(|e| Error::Invalid(e.to_string()))?,
                        "--timeout-ms" => limits.total_timeout = Duration::from_millis(n),
                        "--idle-ms" => limits.idle_timeout = Duration::from_millis(n),
                        "--connect-ms" => limits.connect_timeout = Duration::from_millis(n),
                        _ => return Err(Error::Invalid(format!("unknown option: {text}"))),
                    }
                    i += 2;
                } else {
                    jobs.push(Job {name: format!("{:03}.bin", jobs.len()+1), url: text.into()});
                    i += 1;
                }
            }
        }
        _ => return Err(Error::Invalid(HELP.into())),
    }
    let (cancel, receiver) = watch::channel(false);
    let signal = tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() { let _ = cancel.send(true); }
    });
    let outcome = download_batch(jobs, &dest, limits, receiver).await;
    signal.abort(); let _ = signal.await;
    servers.shutdown().await;
    for receipt in outcome? { println!("{}\t{}", receipt.name, receipt.bytes); }
    Ok(())
}
#[tokio::main(flavor = "current_thread")]
async fn main() -> std::process::ExitCode {
    match run(std::env::args_os().skip(1).collect()).await {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => { eprintln!("{error}"); std::process::ExitCode::FAILURE }
    }
}
