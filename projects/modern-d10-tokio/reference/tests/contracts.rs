use d10_tokio_lab::*;
use futures_util::{Stream, StreamExt, stream};
use std::future::{Future, pending, ready};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::{Context, Poll, Wake, Waker};
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tokio::time::{Instant, sleep, timeout};

const LIMIT: Duration = Duration::from_secs(5);
fn ms(n: u64) -> Duration { Duration::from_millis(n) }

#[tokio::test(start_paused = true)]
async fn step1_join_uses_virtual_time_not_serial_wait() {
    let start = Instant::now();
    assert_eq!(join_samples((7, ms(30)), (9, ms(50))).await, (7, 9));
    assert_eq!(start.elapsed(), ms(50));
    let start = Instant::now();
    assert_eq!(join_samples((-2, ms(70)), (3, ms(10))).await, (-2, 3));
    assert_eq!(start.elapsed(), ms(70));
    // Compare with truly sequential awaits in the same virtual clock.
    let start = Instant::now();
    sleep(ms(30)).await;
    sleep(ms(50)).await;
    assert_eq!(start.elapsed(), ms(80));
}

async fn wait_full<T>(receiver: &mpsc::Receiver<T>) {
    timeout(LIMIT, async {
        while receiver.is_empty() {
            sleep(ms(1)).await;
        }
    }).await.expect("producer must fill its queue");
}

#[tokio::test]
async fn step2_bounded_channel_backpressure_and_receiver_close() {
    assert!(start_producer(vec![1], 0).is_err());
    assert!(start_producer(vec![1], 65).is_err());
    let (mut receiver, producer) = start_producer(vec![1, 2, 3], 1).unwrap();
    wait_full(&receiver).await;
    assert_eq!(receiver.capacity(), 0);
    assert_eq!(receiver.len(), 1);
    assert!(!producer.is_finished(), "producer must wait for capacity");
    let result = timeout(LIMIT, async {
        let mut result = Vec::new();
        while let Some(value) = receiver.recv().await {
            result.push(value);
        }
        result
    }).await.unwrap();
    assert_eq!(result, [1, 2, 3]);
    assert_eq!(timeout(LIMIT, producer).await.unwrap().unwrap(), 3);

    let (receiver, producer) = start_producer(vec![10, 20, 30], 1).unwrap();
    wait_full(&receiver).await;
    drop(receiver);
    assert_eq!(timeout(LIMIT, producer).await.unwrap().unwrap(), 1);
    let (mut receiver, producer) = start_producer(Vec::<i32>::new(), 2).unwrap();
    assert_eq!(timeout(LIMIT, receiver.recv()).await.unwrap(), None);
    assert_eq!(timeout(LIMIT, producer).await.unwrap().unwrap(), 0);
}

#[derive(Clone, Default)]
struct Metrics {
    active: Arc<AtomicUsize>,
    peak: Arc<AtomicUsize>,
    finished: Arc<Mutex<Vec<usize>>>,
}
struct Active(Arc<AtomicUsize>);
impl Drop for Active {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}
impl Metrics {
    fn entered(&self) -> Active {
        let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
        self.peak.fetch_max(active, Ordering::SeqCst);
        Active(Arc::clone(&self.active))
    }
}

#[tokio::test(start_paused = true)]
async fn step3_dynamic_limit_and_input_order_are_observable() {
    let metrics = Metrics::default();
    let observed = metrics.clone();
    let values = run_bounded(vec![(0, 40), (1, 5), (2, 10), (3, 1)], 2,
        move |(index, delay)| {
            let metrics = observed.clone();
            async move {
                let _guard = metrics.entered();
                sleep(ms(delay)).await;
                metrics.finished.lock().unwrap().push(index);
                index * 10
            }
        }).await.unwrap();
    assert_eq!(values, [0, 10, 20, 30]);
    assert_eq!(*metrics.finished.lock().unwrap(), [1, 2, 3, 0]);
    assert_eq!(metrics.peak.load(Ordering::SeqCst), 2);
    assert_eq!(metrics.active.load(Ordering::SeqCst), 0);
    assert_eq!(run_bounded(Vec::<i32>::new(), 1, ready).await, Ok(vec![]));
    assert_eq!(run_bounded(vec![1], 0, |_| async { panic!("invalid limit must not start work") }).await,
               Err(InvalidCapacity));
    assert_eq!(run_bounded(vec![1], 65, ready).await, Err(InvalidCapacity));
}

#[derive(Default)]
struct CountWake(AtomicUsize);
impl Wake for CountWake {
    fn wake(self: Arc<Self>) { self.wake_by_ref(); }
    fn wake_by_ref(self: &Arc<Self>) { self.0.fetch_add(1, Ordering::SeqCst); }
}

#[tokio::test]
async fn step4_stream_has_pending_wake_items_eof_and_merge() {
    let (sender, receiver) = mpsc::channel(1);
    let mut stream = ReceiverStream::new(receiver);
    let wakes = Arc::new(CountWake::default());
    let waker = Waker::from(Arc::clone(&wakes));
    assert_eq!(Pin::new(&mut stream).poll_next(&mut Context::from_waker(&waker)), Poll::Pending);
    sender.send(7).await.unwrap();
    assert!(wakes.0.load(Ordering::SeqCst) >= 1, "send must wake the registered stream");
    assert_eq!(stream.next().await, Some(7));
    sender.send(-3).await.unwrap();
    drop(sender);
    assert_eq!(stream.next().await, Some(-3));
    assert_eq!(stream.next().await, None);
    assert_eq!(stream.next().await, None);
    let (receiver, producer) = start_producer(vec![3, -1, 5, 0], 1).unwrap();
    let result = timeout(LIMIT, merge_positive(ReceiverStream::new(receiver),
                    stream::iter([2, 0, 4, -7]))).await.unwrap();
    assert_eq!(producer.await.unwrap(), 4);
    // Each source remains ordered; global interleaving is not specified.
    let left: Vec<_> = result.iter().copied().filter(|n| n % 2 == 1).collect();
    let right: Vec<_> = result.iter().copied().filter(|n| n % 2 == 0).collect();
    assert_eq!(left, [3, 5]);
    assert_eq!(right, [2, 4]);
    assert_eq!(result.len(), 4);
    assert!(merge_positive(stream::empty::<i32>(), stream::iter([-2, 0])).await.is_empty());
}

struct DropCount(Arc<AtomicUsize>);
impl Drop for DropCount {
    fn drop(&mut self) { self.0.fetch_add(1, Ordering::SeqCst); }
}

#[tokio::test(start_paused = true)]
async fn step5_select_cancel_timeout_and_spawned_abort() {
    let (send, cancel) = oneshot::channel();
    send.send(()).unwrap();
    assert_eq!(race_or_timeout(ready(7), cancel, ms(20)).await, Outcome::Cancelled);
    let (send, cancel) = oneshot::channel();
    drop(send);
    assert_eq!(race_or_timeout(ready(7), cancel, ms(20)).await, Outcome::Completed(7));
    let (_send, cancel) = oneshot::channel();
    let drops = Arc::new(AtomicUsize::new(0));
    let guard = DropCount(Arc::clone(&drops));
    let work = async move { let _guard = guard; pending::<i32>().await };
    assert_eq!(race_or_timeout(work, cancel, ms(10)).await, Outcome::TimedOut);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    let (send, cancel) = oneshot::channel();
    let guard = DropCount(Arc::clone(&drops));
    let work = async move { let _guard = guard; pending::<i32>().await };
    let (outcome, _) = tokio::join!(race_or_timeout(work, cancel, ms(100)), async {
        sleep(ms(5)).await;
        send.send(()).unwrap();
    });
    assert_eq!(outcome, Outcome::Cancelled);
    assert_eq!(drops.load(Ordering::SeqCst), 2);

    let (started, running) = oneshot::channel();
    let guard = DropCount(Arc::clone(&drops));
    let handle = tokio::spawn(async move {
        let _guard = guard;
        started.send(()).unwrap();
        pending::<i32>().await
    });
    running.await.unwrap();
    assert!(abort_and_join(handle).await.unwrap_err().is_cancelled());
    assert_eq!(drops.load(Ordering::SeqCst), 3);
}

#[tokio::test(flavor = "current_thread")]
async fn step6_blocking_pool_does_not_block_async_progress() {
    let runtime_thread = std::thread::current().id();
    let (started, started_rx) = oneshot::channel();
    let (release, wait) = std::sync::mpsc::channel();
    let blocking = blocking_work(move || {
        assert_ne!(std::thread::current().id(), runtime_thread);
        started.send(()).unwrap();
        wait.recv_timeout(LIMIT).expect("async work must release blocking job");
        42
    });
    let async_side = async {
        started_rx.await.unwrap();
        tokio::task::yield_now().await;
        release.send(()).unwrap();
        9
    };
    let (result, other) = timeout(LIMIT, async { tokio::join!(blocking, async_side) }).await.unwrap();
    assert_eq!(result.unwrap(), 42);
    assert_eq!(other, 9);
    assert!(blocking_work(|| -> () { panic!("controlled blocking panic") }).await.unwrap_err().is_panic());
}

struct LocalSource;
impl SampleSource for LocalSource {
    async fn read(&self, key: &str) -> Option<i32> {
        tokio::task::yield_now().await;
        match key { "thermal" => Some(31), "power" => Some(-2), _ => None }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn step7_send_bound_async_trait_can_be_spawned() {
    let first = spawn_read(LocalSource, String::from("thermal"));
    let second = spawn_read(LocalSource, String::from("power"));
    let missing = spawn_read(LocalSource, String::from("missing"));
    assert_eq!(timeout(LIMIT, first).await.unwrap().unwrap(), Some(31));
    assert_eq!(timeout(LIMIT, second).await.unwrap().unwrap(), Some(-2));
    assert_eq!(timeout(LIMIT, missing).await.unwrap().unwrap(), None);
}
