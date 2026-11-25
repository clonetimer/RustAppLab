//! D10-A: Future, Poll, Waker, Pin, composition and cancellation.
//! The supplied executor is a local teaching fixture, not a network runtime.
pub mod runtime;

use std::future::Future;
use std::marker::PhantomPinned;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

pub struct YieldOnce<T> {
    value: Option<T>,
    yielded: bool,
}

impl<T> YieldOnce<T> {
    pub fn new(value: T) -> Self {
        Self { value: Some(value), yielded: false }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Winner<L, R> {
    Left(L),
    Right(R),
}

pub type BoxTask<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

struct State<T> {
    value: Option<T>,
    closed: bool,
    receiver_alive: bool,
    waker: Option<Waker>,
}

pub struct SignalSender<T> {
    state: Arc<Mutex<State<T>>>,
}

pub struct SignalReceiver<T> {
    state: Arc<Mutex<State<T>>>,
}

pub fn signal<T>() -> (SignalSender<T>, SignalReceiver<T>) {
    let state = Arc::new(Mutex::new(State {
        value: None,
        closed: false,
        receiver_alive: true,
        waker: None,
    }));
    (SignalSender { state: Arc::clone(&state) }, SignalReceiver { state })
}

impl<T> SignalSender<T> {
    pub fn send(self, value: T) -> Result<(), T> {
        let wake = {
            let mut state = self.state.lock().expect("signal lock not poisoned");
            if !state.receiver_alive {
                return Err(value);
            }
            state.value = Some(value);
            state.closed = true;
            state.waker.take()
        };
        if let Some(waker) = wake {
            waker.wake();
        }
        Ok(())
    }
}

impl<T> Drop for SignalSender<T> {
    fn drop(&mut self) {
        let wake = {
            let mut state = self.state.lock().expect("signal lock not poisoned");
            state.closed = true;
            state.waker.take()
        };
        if let Some(waker) = wake {
            waker.wake();
        }
    }
}

impl<T> Drop for SignalReceiver<T> {
    fn drop(&mut self) {
        let (value, waker) = {
            let mut state = self.state.lock().expect("signal lock not poisoned");
            state.receiver_alive = false;
            (state.value.take(), state.waker.take())
        };
        // Drop potentially user-owned resources outside the mutex.
        drop((value, waker));
    }
}

pub struct PinProbe {
    value: u32,
    _pin: PhantomPinned,
}

impl Future for PinProbe {
    type Output = u32;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<u32> {
        Poll::Ready(self.as_ref().get_ref().value)
    }
}

impl<T: Unpin> Future for YieldOnce<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<T> {
        // TODO(1): first yield Pending and arrange a wake, then return the value.
        let _ = self.yielded;
        Poll::Ready(self.value.take().unwrap())
    }
}

pub async fn join_pair<L: Future, R: Future>(_left: L, _right: R) -> (L::Output, R::Output) {
    // TODO(2): pin both children, keep separate completion slots, never re-poll Ready.
    todo!()
}

pub async fn join_many<'a, T>(_tasks: Vec<BoxTask<'a, T>>) -> Vec<T> {
    // TODO(3): dynamically poll all unfinished heterogeneous futures; preserve input order.
    Vec::new()
}

pub async fn race_pair<L: Future, R: Future>(_left: L, _right: R) -> Winner<L::Output, R::Output> {
    // TODO(4): poll both until a winner; document left priority and drop the loser.
    todo!()
}

impl<T> Future for SignalReceiver<T> {
    type Output = Option<T>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<T>> {
        // TODO(5): inspect value/closed under the mutex; save the latest Waker if pending.
        let _ = &self.state;
        Poll::Ready(None)
    }
}

pub fn pin_probe(_value: u32) -> Pin<Box<PinProbe>> {
    // TODO(6): pin a non-Unpin object without using unsafe.
    todo!()
}
