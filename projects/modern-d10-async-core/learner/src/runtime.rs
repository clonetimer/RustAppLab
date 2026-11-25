//! Supplied single-root teaching executor. Not a Tokio replacement.
//! No timer driver, IO reactor, spawning, or preemptive deadline.
use std::future::Future;
use std::sync::{Arc, mpsc};
use std::task::{Context, Poll, Wake, Waker};
use std::time::{Duration, Instant};

#[derive(Debug, PartialEq, Eq)]
pub struct TimedOut;

struct Notification(mpsc::SyncSender<()>);

impl Wake for Notification {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        // A full channel already contains a notification; a disconnected
        // channel means the root future has finished or been cancelled.
        let _ = self.0.try_send(());
    }
}

/// Poll initially, then wait for a real wake notification. Return poll count.
/// The deadline is cooperative; a single blocking poll cannot be preempted.
pub fn run_until<F: Future>(future: F, timeout: Duration) -> (Result<F::Output, TimedOut>, usize) {
    let (send, receive) = mpsc::sync_channel(1);
    let waker = Waker::from(Arc::new(Notification(send)));
    let mut context = Context::from_waker(&waker);
    let mut future = std::pin::pin!(future);
    let deadline = Instant::now() + timeout;
    let mut polls = 0;
    loop {
        polls += 1;
        if let Poll::Ready(output) = future.as_mut().poll(&mut context) {
            return (Ok(output), polls);
        }
        let now = Instant::now();
        if now >= deadline || receive.recv_timeout(deadline - now).is_err() {
            return (Err(TimedOut), polls);
        }
        if Instant::now() >= deadline {
            return (Err(TimedOut), polls);
        }
    }
}
