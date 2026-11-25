//! D10-A: Future, Poll, Waker, Pin, composition and cancellation.
//! The supplied executor is a local teaching fixture, not a network runtime.
pub mod runtime;

use std::future::{Future, poll_fn};
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

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        if !self.yielded {
            self.yielded = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        } else {
            Poll::Ready(self.value.take().expect("do not poll again after Ready"))
        }
    }
}

/// Drive both children; never poll a completed child again.
pub async fn join_pair<L: Future, R: Future>(left: L, right: R) -> (L::Output, R::Output) {
    let mut left = std::pin::pin!(left);
    let mut right = std::pin::pin!(right);
    let mut a = None;
    let mut b = None;
    poll_fn(|cx| {
        if a.is_none() {
            if let Poll::Ready(value) = left.as_mut().poll(cx) {
                a = Some(value);
            }
        }
        if b.is_none() {
            if let Poll::Ready(value) = right.as_mut().poll(cx) {
                b = Some(value);
            }
        }
        if a.is_some() && b.is_some() {
            Poll::Ready((a.take().unwrap(), b.take().unwrap()))
        } else {
            Poll::Pending
        }
    })
    .await
}

/// Accept a dynamic heterogeneous set, preserving original input order.
/// This is an all-in-one teaching combinator, not a production task scheduler.
pub async fn join_many<'a, T>(mut tasks: Vec<BoxTask<'a, T>>) -> Vec<T> {
    let mut outputs: Vec<Option<T>> = (0..tasks.len()).map(|_| None).collect();
    let mut remaining = tasks.len();
    poll_fn(|cx| {
        for (task, output) in tasks.iter_mut().zip(&mut outputs) {
            if output.is_none() {
                if let Poll::Ready(value) = task.as_mut().poll(cx) {
                    *output = Some(value);
                    remaining -= 1;
                }
            }
        }
        if remaining == 0 {
            Poll::Ready(outputs.iter_mut().map(|value| value.take().unwrap()).collect())
        } else {
            Poll::Pending
        }
    })
    .await
}

/// Left wins ties; the losing in-scope future is dropped, not spawned.
pub async fn race_pair<L: Future, R: Future>(left: L, right: R) -> Winner<L::Output, R::Output> {
    let mut left = std::pin::pin!(left);
    let mut right = std::pin::pin!(right);
    poll_fn(|cx| {
        if let Poll::Ready(value) = left.as_mut().poll(cx) {
            Poll::Ready(Winner::Left(value))
        } else if let Poll::Ready(value) = right.as_mut().poll(cx) {
            Poll::Ready(Winner::Right(value))
        } else {
            Poll::Pending
        }
    })
    .await
}

impl<T> Future for SignalReceiver<T> {
    type Output = Option<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<T>> {
        let mut state = self.state.lock().expect("signal lock not poisoned");
        if let Some(value) = state.value.take() {
            Poll::Ready(Some(value))
        } else if state.closed {
            Poll::Ready(None)
        } else {
            // Register under the same lock used by send/drop, avoiding a lost wake.
            if state.waker.as_ref().is_none_or(|old| !old.will_wake(cx.waker())) {
                state.waker = Some(cx.waker().clone());
            }
            Poll::Pending
        }
    }
}

pub fn pin_probe(value: u32) -> Pin<Box<PinProbe>> {
    Box::pin(PinProbe { value, _pin: PhantomPinned })
}
