use e1_secure_lab::{Credentials, TlsListener, server_config, serve};
use e1_sqlite_lab::{AccessLog, Store, router, start_store};
use std::{env, fs, path::Path, process::ExitCode};
use tokio::net::TcpListener;

const HELP: &str = "E1-B local SQLite HTTPS API\n  e1-sqlite-lab init <new-db>\n  e1-sqlite-lab serve <db> <cert.der> <key.pkcs8.der> <port>\n  Set E1_READ_TOKEN and E1_WRITE_TOKEN; localhost only.";

async fn run() -> Result<(), &'static str> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if args.len()==1 && args[0]=="--help" { println!("{HELP}"); return Ok(()); }
    if args.len()==2 && args[0]=="init" {
        Store::init(Path::new(&args[1])).map_err(|_| "init failed; never overwrite")?;
        println!("SQLite created");
        return Ok(());
    }
    if args.len()!=5 || args[0]!="serve" { return Err(HELP); }
    let port=args[4].to_str().and_then(|v| v.parse::<u16>().ok()).ok_or("invalid port")?;
    let reader=env::var("E1_READ_TOKEN").map_err(|_| "reader credential missing")?;
    let writer=env::var("E1_WRITE_TOKEN").map_err(|_| "writer credential missing")?;
    let credentials=Credentials::new(&reader,&writer)?;
    let cert=fs::read(&args[2]).map_err(|_| "certificate missing")?;
    let key=fs::read(&args[3]).map_err(|_| "key missing")?;
    let config=server_config(&cert,&key)?;
    let tcp=TcpListener::bind(("127.0.0.1",port)).await.map_err(|_| "bind failed")?;
    let address=tcp.local_addr().map_err(|_| "address unavailable")?;
    let listener=TlsListener::new(tcp,config).map_err(|_| "listener rejected")?;
    let (backend, worker)=start_store(Path::new(&args[1])).map_err(|_| "storage unavailable")?;
    let log=AccessLog::new(256).map_err(|_| "log configuration")?;
    println!("LISTENING https://localhost:{}/",address.port());
    let result=serve(listener,router(backend,credentials,log.clone()),async {
        let _=tokio::signal::ctrl_c().await;
    }).await;
    worker.await.map_err(|_| "database worker failed")?;
    let (total,dropped,_)=log.snapshot();
    println!("{}",log.json_lines());
    println!("{}",serde_json::json!({"requests":total,"dropped_events":dropped}));
    result.map_err(|_| "server failed")
}

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(())=>ExitCode::SUCCESS,
        Err(message)=>{eprintln!("{message}"); ExitCode::FAILURE}
    }
}
