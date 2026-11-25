//! D10-B: real Tokio and futures-util Stream exercises, no public network.
use futures_util::{Stream, StreamExt, stream};
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

async fn sample_after(sample: (i32, Duration)) -> i32 {
    tokio::time::sleep(sample.1).await;
    sample.0
}

/// Two concurrent waits, retaining argument order, not completion order.
pub async fn join_samples(left: (i32, Duration), right: (i32, Duration)) -> (i32, i32) {
    tokio::join!(sample_after(left), sample_after(right))
}

/// Spawn a producer with a bounded queue. Return the actual number of sends.
pub fn start_producer<T: Send + 'static>(
    values: Vec<T>,
    capacity: usize,
) -> Result<Producer<T>, InvalidCapacity> {
    if !(1..=64).contains(&capacity) {
        return Err(InvalidCapacity);
    }
    let (sender, receiver) = mpsc::channel(capacity);
    let handle = tokio::spawn(async move {
        let mut sent = 0;
        for value in values {
            if sender.send(value).await.is_err() {
                break;
            }
            sent += 1;
        }
        sent
    });
    Ok((receiver, handle))
}

/// Bounded in-flight futures. No per-item spawn, so dropping the outer future
/// also drops its in-scope unfinished children.
pub async fn run_bounded<J, T, F, Fut>(
    jobs: Vec<J>,
    limit: usize,
    mut work: F,
) -> Result<Vec<T>, InvalidCapacity>
where
    F: FnMut(J) -> Fut,
    Fut: Future<Output = T>,
{
    if !(1..=64).contains(&limit) {
        return Err(InvalidCapacity);
    }
    let mut completed: Vec<_> = stream::iter(jobs.into_iter().enumerate())
        .map(|(index, job)| {
            let future = work(job);
            async move { (index, future.await) }
        })
        .buffer_unordered(limit)
        .collect()
        .await;
    completed.sort_unstable_by_key(|(index, _)| *index);
    Ok(completed.into_iter().map(|(_, value)| value).collect())
}

impl<T> Stream for ReceiverStream<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<T>> {
        self.inner.poll_recv(cx)
    }
}

/// Merge streams; preserve each source's order but do not promise global order.
pub async fn merge_positive<L, R>(left: L, right: R) -> Vec<i32>
where
    L: Stream<Item = i32>,
    R: Stream<Item = i32>,
{
    stream::select(left, right)
        .filter(|value| std::future::ready(*value > 0))
        .collect()
        .await
}

/// Explicit cancellation wins simultaneous readiness; closing the signal
/// sender without sending is not itself a cancellation request.
pub async fn race_or_timeout<F: Future>(
    work: F,
    cancel: oneshot::Receiver<()>,
    deadline: Duration,
) -> Outcome<F::Output> {
    let cancellation = async move {
        if cancel.await.is_err() {
            std::future::pending::<()>().await;
        }
    };
    tokio::select! {
        biased;
        _ = cancellation => Outcome::Cancelled,
        value = work => Outcome::Completed(value),
        _ = tokio::time::sleep(deadline) => Outcome::TimedOut,
    }
}

/// Abort a spawned async task, then wait for its actual termination.
pub async fn abort_and_join<T>(handle: JoinHandle<T>) -> Result<T, JoinError> {
    handle.abort();
    handle.await
}

/// A synchronous closure belongs on the blocking pool, not a runtime worker.
pub async fn blocking_work<T, F>(job: F) -> Result<T, JoinError>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    tokio::task::spawn_blocking(job).await
}

pub fn spawn_read<S>(source: S, key: String) -> JoinHandle<Option<i32>>
where
    S: SampleSource + Send + Sync + 'static,
{
    tokio::spawn(async move { source.read(&key).await })
}
