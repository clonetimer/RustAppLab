use servicedesk_c2::{router, serve, start_store, ApiError};
use std::{ffi::OsString, io::{self, Write}, path::PathBuf, process::ExitCode};
use tokio::{net::TcpListener, sync::watch};
const HELP: &str = "ServiceDesk C2 — loopback Axum HTTP/JSON\n  servicedesk-c2 init <new-journal-file>\n  servicedesk-c2 serve <existing-journal-file> [--port 3000]\n  servicedesk-c2 --help\nNever bind to a public interface. Ctrl+C triggers graceful shutdown.";

async fn run(args: Vec<OsString>) -> Result<(), ApiError> {
    if args.len()==1 && matches!(args[0].to_str(),Some("--help" | "-h" | "help")) {
        println!("{HELP}"); return Ok(());
    }
    let command = args.first().and_then(|v| v.to_str()).unwrap_or("");
    let db = PathBuf::from(args.get(1).ok_or_else(|| ApiError::Invalid(HELP.into()))?);
    match command {
        "init" if args.len()==2 => {
            servicedesk::init(&db).map_err(|_| ApiError::Storage)?;
            println!("CREATED {}", db.display()); Ok(())
        }
        "serve" if args.len()==2 || args.len()==4 => {
            let port: u16 = if args.len()==2 {3000} else {
                if args[2].to_str()!=Some("--port") {return Err(ApiError::Invalid(HELP.into()));}
                args[3].to_str().and_then(|p| p.parse().ok()).ok_or_else(||ApiError::Invalid("端口号不合法".into()))?
            };
            let state = start_store(&db)?;
            let listener = TcpListener::bind(("127.0.0.1",port)).await
                .map_err(|e|ApiError::Lifecycle(e.to_string()))?;
            let address = listener.local_addr().map_err(|e|ApiError::Lifecycle(e.to_string()))?;
            println!("LISTENING http://{address}/");
            io::stdout().flush().map_err(|e|ApiError::Lifecycle(e.to_string()))?;
            let (tx,rx) = watch::channel(false);
            let signal = tokio::spawn(async move {
                if tokio::signal::ctrl_c().await.is_ok() {let _=tx.send(true);}
            });
            let result = serve(listener,router(state),rx).await;
            signal.abort();let _=signal.await;
            result
        }
        _ => Err(ApiError::Invalid(HELP.into())),
    }
}
#[tokio::main]
async fn main() -> ExitCode {
    match run(std::env::args_os().skip(1).collect()).await {
        Ok(())=>ExitCode::SUCCESS,
        Err(e)=>{eprintln!("{e}");ExitCode::FAILURE}
    }
}
