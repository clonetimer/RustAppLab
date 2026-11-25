//! D10-B: real Tokio and futures-util Stream exercises, no public network.
use futures_util::Stream;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tokio::task::{JoinError, JoinHandle};

#[derive(Debug, PartialEq, Eq)]
pub struct InvalidCapacity;

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome<T> {
    Completed(T),
    Cancelled,
    TimedOut,
}

pub struct ReceiverStream<T> {
    inner: mpsc::Receiver<T>,
}

impl<T> ReceiverStream<T> {
    pub fn new(inner: mpsc::Receiver<T>) -> Self {
        Self { inner }
    }
}

/// Explicit Send bound on the returned future: unlike a bare async fn in trait,
/// callers can use this contract to spawn the operation on Tokio.
pub trait SampleSource {
    fn read(&self, key: &str) -> impl Future<Output = Option<i32>> + Send;
}

pub type Producer<T> = (mpsc::Receiver<T>, JoinHandle<usize>);

pub async fn join_samples(left: (i32, Duration), right: (i32, Duration)) -> (i32, i32) {
    // TODO(1): use Tokio join to advance both timer futures concurrently.
    let _ = (left, right);
    (0, 0)
}

pub fn start_producer<T: Send + 'static>(
    values: Vec<T>,
    capacity: usize,
) -> Result<Producer<T>, InvalidCapacity> {
    // TODO(2): validate capacity, spawn a bounded mpsc producer, count actual sends.
    let _ = (values, capacity);
    Err(InvalidCapacity)
}

pub async fn run_bounded<J, T, F, Fut>(
    jobs: Vec<J>,
    limit: usize,
    work: F,
) -> Result<Vec<T>, InvalidCapacity>
where
    F: FnMut(J) -> Fut,
    Fut: Future<Output = T>,
{
    // TODO(3): buffer a bounded number of futures and restore input order.
    let _ = (jobs, limit, work);
    Ok(Vec::new())
}

impl<T> Stream for ReceiverStream<T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<T>> {
        // TODO(4): delegate to the actual mpsc receiver poll_recv.
        let _ = &self.inner;
        Poll::Ready(None)
    }
}

pub async fn merge_positive<L, R>(left: L, right: R) -> Vec<i32>
where
    L: Stream<Item = i32>,
    R: Stream<Item = i32>,
{
    // TODO(4b): use StreamExt to merge two sources and filter positive values.
    let _ = (left, right);
    Vec::new()
}

pub async fn race_or_timeout<F: Future>(
    work: F,
    cancel: oneshot::Receiver<()>,
    deadline: Duration,
) -> Outcome<F::Output> {
    // TODO(5): select explicit cancellation, work, and timeout; handle signal closure.
    let _ = (work, cancel, deadline);
    Outcome::TimedOut
}

pub async fn abort_and_join<T>(handle: JoinHandle<T>) -> Result<T, JoinError> {
    // TODO(5b): abort and await termination; dropping a JoinHandle detaches.
    let _ = handle;
    todo!()
}

pub async fn blocking_work<T, F>(job: F) -> Result<T, JoinError>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    // TODO(6): execute this synchronous closure on Tokio's blocking pool.
    let _ = job;
    todo!()
}

pub fn spawn_read<S>(source: S, key: String) -> JoinHandle<Option<i32>>
where
    S: SampleSource + Send + Sync + 'static,
{
    // TODO(7): move ownership into a spawned async task and await the source.
    let _ = (source, key);
    todo!()
}
