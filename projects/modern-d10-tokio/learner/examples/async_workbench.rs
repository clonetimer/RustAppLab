use d10_tokio_lab::*;
use futures_util::stream;
use std::future::pending;
use std::time::Duration;
use tokio::sync::oneshot;

struct Local;
impl SampleSource for Local {
    async fn read(&self, key: &str) -> Option<i32> {
        tokio::task::yield_now().await;
        (key == "thermal").then_some(31)
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let delay = Duration::from_millis(1);
    println!("joined={:?}", join_samples((7, delay), (9, delay)).await);
    let (receiver, producer) = start_producer(vec![3, -2, 5], 1).expect("capacity");
    let mut positive = merge_positive(ReceiverStream::new(receiver), stream::iter([2, 0, 4])).await;
    // Global merge order is intentionally not part of the result contract.
    positive.sort_unstable();
    println!("positive={positive:?}; sent={}", producer.await.expect("producer"));
    let result = run_bounded(vec![3, 1, 2], 2, |n| async move {
        tokio::task::yield_now().await;
        n * n
    }).await.expect("limit");
    println!("ordered={result:?}");
    let (send, cancel) = oneshot::channel();
    send.send(()).expect("signal");
    println!("race={:?}", race_or_timeout(pending::<i32>(), cancel, delay).await);
    println!("blocking={}", blocking_work(|| 42).await.expect("blocking"));
    println!("sample={:?}", spawn_read(Local, String::from("thermal")).await.expect("task"));
}
