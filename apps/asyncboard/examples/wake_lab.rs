//! Run from the platform: cargo run --offline -p asyncboard --example wake_lab
//! A maintainer-provided experiment, not a completed learner exercise.
use asyncboard::runtime::{drive, TimerDriver};
use std::{
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    task::{Context, Poll},
    time::Duration,
};
struct SilentPending(Arc<AtomicUsize>);
impl Future for SilentPending {
    type Output = ();
    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<()> {
        self.0.fetch_add(1, Ordering::SeqCst);
        // Deliberate mistake: no wake registration. The executor must not busy-poll.
        Poll::Pending
    }
}
fn main() -> Result<(), String> {
    let started = Arc::new(AtomicUsize::new(0));
    let state = Arc::clone(&started);
    let future = async move {
        state.fetch_add(1, Ordering::SeqCst);
        42
    };
    assert_eq!(started.load(Ordering::SeqCst), 0);
    println!("created_future body_runs=0");
    assert_eq!(drive(vec![future], Duration::from_secs(1))?, [42]);
    println!("driven_future body_runs={}", started.load(Ordering::SeqCst));
    let polls = Arc::new(AtomicUsize::new(0));
    let outcome = drive(
        vec![SilentPending(Arc::clone(&polls))],
        Duration::from_millis(80),
    );
    assert!(outcome.is_err());
    assert_eq!(polls.load(Ordering::SeqCst), 1);
    println!("missing_wake polls=1 result=timeout (expected)");
    let mut timers = TimerDriver::new()?;
    let delay = timers.delay(Duration::from_millis(20))?;
    let out = drive(
        vec![async move {
            delay.await;
            "awake"
        }],
        Duration::from_secs(2),
    )?;
    assert_eq!(out, ["awake"]);
    println!("timer_wake result=awake");
    println!("Boundary: deadline is cooperative; it cannot interrupt a blocking poll.");
    Ok(())
}
