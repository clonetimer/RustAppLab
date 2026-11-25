use std::future::Future;
trait Source { fn read(&self) -> impl Future<Output = i32> + Send; }
struct Local;
impl Source for Local {
    async fn read(&self) -> i32 { tokio::task::yield_now().await; 7 }
}
fn launch<S: Source + Send + Sync + 'static>(source: S) -> tokio::task::JoinHandle<i32> {
    tokio::spawn(async move { source.read().await })
}
#[tokio::main(flavor = "current_thread")]
async fn main() {
    assert_eq!(launch(Local).await.unwrap(), 7);
    println!("ok");
}
