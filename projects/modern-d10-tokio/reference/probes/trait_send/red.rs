trait Source { async fn read(&self) -> i32; }
fn launch<S: Source + Send + Sync + 'static>(source: S) -> tokio::task::JoinHandle<i32> {
    tokio::spawn(async move { source.read().await })
}
fn main() {}
