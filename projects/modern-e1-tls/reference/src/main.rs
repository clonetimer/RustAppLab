use e1_secure_lab::{Credentials, TlsListener, protected_router, server_config, serve};
use std::{env, fs, path::Path, process::ExitCode};
use tokio::net::TcpListener;

const HELP: &str = "E1-A local HTTPS API\n  e1-secure-lab init <new-journal>\n  e1-secure-lab serve <journal> <cert.der> <key.pkcs8.der> <port>\n  Set E1_READ_TOKEN and E1_WRITE_TOKEN (distinct random 64-hex credentials).\n  Loopback only; never use a public interface or real credentials in this lesson.";

async fn run() -> Result<(), &'static str> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if args.len() == 1 && args[0] == "--help" {
        println!("{HELP}");
        return Ok(());
    }
    if args.len() == 2 && args[0] == "init" {
        servicedesk_c2::init_store(Path::new(&args[1])).map_err(|_| "journal init failed (will not overwrite)")?;
        println!("Journal created");
        return Ok(());
    }
    if args.len() != 5 || args[0] != "serve" {
        return Err(HELP);
    }
    let port: u16 = args[4].to_str().and_then(|s| s.parse().ok()).ok_or("invalid port")?;
    let reader = env::var("E1_READ_TOKEN").map_err(|_| "reader credential missing")?;
    let writer = env::var("E1_WRITE_TOKEN").map_err(|_| "writer credential missing")?;
    let auth = Credentials::new(&reader, &writer)?;
    let cert = fs::read(&args[2]).map_err(|_| "cannot read certificate")?;
    let key = fs::read(&args[3]).map_err(|_| "cannot read key")?;
    let config = server_config(&cert, &key)?;
    let tcp = TcpListener::bind(("127.0.0.1", port)).await.map_err(|_| "bind failed")?;
    let addr = tcp.local_addr().map_err(|_| "local address unavailable")?;
    let listener = TlsListener::new(tcp, config).map_err(|_| "listener rejected")?;
    let state = servicedesk_c2::start_store(Path::new(&args[1])).map_err(|_| "storage unavailable")?;
    println!("LISTENING https://localhost:{}/", addr.port());
    serve(listener, protected_router(state, auth), async {
        let _ = tokio::signal::ctrl_c().await;
    }).await.map_err(|_| "serve failed")
}

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => { eprintln!("{message}"); ExitCode::FAILURE }
    }
}
