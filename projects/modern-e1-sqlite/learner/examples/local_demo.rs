//! Actual local HTTPS requests, SQLite reopen, and redacted JSON-lines observation.
#[path = "../tests/support/mod.rs"] mod support;
use support::*;
use e1_secure_lab::{Credentials,TlsListener,server_config,serve};
use e1_sqlite_lab::{AccessLog,Store,router,start_store};
use tokio::{net::TcpListener,sync::oneshot};
#[tokio::main]
async fn main(){
    let dir=Temp::new();
    let db=dir.0.join("demo.sqlite");
    Store::init(&db).unwrap();
    let cert=certificate();let read=token();let write=token();
    let auth=Credentials::new(&read,&write).unwrap();
    let (backend,worker)=start_store(&db).unwrap();
    let log=AccessLog::new(8).unwrap();
    let tcp=TcpListener::bind(("127.0.0.1",0)).await.unwrap();let address=tcp.local_addr().unwrap();
    let tls=TlsListener::new(tcp,server_config(&cert.der,&cert.key).unwrap()).unwrap();
    let (stop,signal)=oneshot::channel::<()>();
    let server=tokio::spawn(serve(tls,router(backend,auth,log.clone()),async{let _=signal.await;}));
    let denied=request(address,&cert.der,"localhost",http("POST","/api/tickets",Some(&read),r#"{"title":"demo"}"#,"")).await.unwrap().0;
    let created=request(address,&cert.der,"localhost",http("POST","/api/tickets",Some(&write),r#"{"title":"demo"}"#,"")).await.unwrap().0;
    let closed=request(address,&cert.der,"localhost",http("POST","/api/tickets/1/close",Some(&write),"","")).await.unwrap().0;
    assert_eq!((denied,created,closed),(403,201,200));
    stop.send(()).unwrap();tokio::time::timeout(LIMIT,server).await.unwrap().unwrap().unwrap();
    tokio::time::timeout(LIMIT,worker).await.unwrap().unwrap();
    let reopened=Store::open(&db).unwrap();
    let rows=reopened.list().unwrap();let events=reopened.audit().unwrap();
    assert_eq!(rows.len(),1);assert!(rows[0].done);assert_eq!(events.len(),2);
    println!("HTTPS+SQLite verified; denied={denied}; created={created}; closed={closed}; reopened={}; audit={}",rows.len(),events.len());
    println!("{}",log.json_lines());
}
