use d10_async_core::runtime::{TimedOut, run_until};
use d10_async_core::*;
use std::cell::{Cell, RefCell};
use std::future::{Future, pending, poll_fn, ready};
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::{Context, Poll, Wake, Waker};
use std::time::Duration;

const LIMIT: Duration = Duration::from_secs(3);

#[derive(Default)]
struct CountWake(AtomicUsize);
impl Wake for CountWake {
    fn wake(self: Arc<Self>) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn step1_lazy_pending_and_real_wake() {
    let wakes = Arc::new(CountWake::default());
    let waker = Waker::from(Arc::clone(&wakes));
    let mut context = Context::from_waker(&waker);
    let mut future = Box::pin(YieldOnce::new(7));
    assert_eq!(future.as_mut().poll(&mut context), Poll::Pending);
    assert_eq!(wakes.0.load(Ordering::SeqCst), 1);
    assert_eq!(future.as_mut().poll(&mut context), Poll::Ready(7));

    let calls = Cell::new(0);
    let lazy = async {
        calls.set(calls.get() + 1);
        YieldOnce::new(19).await
    };
    assert_eq!(calls.get(), 0, "async body is lazy");
    assert_eq!(run_until(lazy, LIMIT), (Ok(19), 2));
    assert_eq!(calls.get(), 1);
    assert_eq!(run_until(pending::<u32>(), Duration::from_millis(20)), (Err(TimedOut), 1));
}

struct TraceFuture {
    label: &'static str,
    pending: usize,
    done: bool,
    events: Rc<RefCell<Vec<&'static str>>>,
    value: i32,
}

impl Future for TraceFuture {
    type Output = i32;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<i32> {
        assert!(!self.done, "completed child was polled again");
        self.events.borrow_mut().push(self.label);
        if self.pending != 0 {
            self.pending -= 1;
            cx.waker().wake_by_ref();
            Poll::Pending
        } else {
            self.done = true;
            Poll::Ready(self.value)
        }
    }
}

fn traced(label: &'static str, pending: usize, value: i32,
          events: &Rc<RefCell<Vec<&'static str>>>) -> TraceFuture {
    TraceFuture { label, pending, done: false, value, events: Rc::clone(events) }
}

#[test]
fn step2_join_polls_both_and_keeps_ready_output() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let left = traced("left", 2, 10, &events);
    let right = traced("right", 0, 20, &events);
    let (result, polls) = run_until(join_pair(left, right), LIMIT);
    assert_eq!(result, Ok((10, 20)));
    assert_eq!(polls, 3);
    assert_eq!(*events.borrow(), ["left", "right", "left", "left"]);
    let (result, _) = run_until(join_pair(async { 3 }, async { String::from("ok") }), LIMIT);
    assert_eq!(result, Ok((3, String::from("ok"))));
}

#[test]
fn step3_dynamic_heterogeneous_futures_keep_order() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let original = String::from("borrow");
    let jobs: Vec<BoxTask<'_, i32>> = vec![
        Box::pin(traced("slow", 2, 9, &events)),
        Box::pin(async { YieldOnce::new(original.len() as i32).await }),
        Box::pin(traced("ready", 0, 3, &events)),
    ];
    let (result, _) = run_until(join_many(jobs), LIMIT);
    assert_eq!(result, Ok(vec![9, 6, 3]));
    assert_eq!(*events.borrow(), ["slow", "ready", "slow", "slow"]);
    assert_eq!(run_until(join_many::<i32>(vec![]), LIMIT), (Ok(vec![]), 1));
}

struct NeverReady {
    drops: Rc<Cell<usize>>,
    polls: Rc<Cell<usize>>,
}

impl Future for NeverReady {
    type Output = u32;
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<u32> {
        self.polls.set(self.polls.get() + 1);
        Poll::Pending
    }
}

impl Drop for NeverReady {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
    }
}

#[test]
fn step4_race_drops_loser_and_has_explicit_tie_policy() {
    let drops = Rc::new(Cell::new(0));
    let polls = Rc::new(Cell::new(0));
    let loser = NeverReady { drops: Rc::clone(&drops), polls: Rc::clone(&polls) };
    assert_eq!(run_until(race_pair(loser, ready("winner")), LIMIT).0, Ok(Winner::Right("winner")));
    assert_eq!(polls.get(), 1);
    assert_eq!(drops.get(), 1, "loser must be dropped before result returns");
    assert_eq!(run_until(race_pair(ready(5), ready(9)), LIMIT).0, Ok(Winner::Left(5)));
    assert_eq!(run_until(race_pair(YieldOnce::new(11), YieldOnce::new(12)), LIMIT).0,
               Ok(Winner::Left(11)));
}

#[test]
fn step5_signal_registers_latest_waker_and_handles_thread_send_close_drop() {
    let (send, receive) = signal::<u32>();
    let mut receive = Box::pin(receive);
    let first = Arc::new(CountWake::default());
    let second = Arc::new(CountWake::default());
    let w1 = Waker::from(Arc::clone(&first));
    let w2 = Waker::from(Arc::clone(&second));
    assert_eq!(receive.as_mut().poll(&mut Context::from_waker(&w1)), Poll::Pending);
    assert_eq!(receive.as_mut().poll(&mut Context::from_waker(&w2)), Poll::Pending);
    std::thread::spawn(move || send.send(42).unwrap()).join().unwrap();
    assert_eq!(first.0.load(Ordering::SeqCst), 0);
    assert_eq!(second.0.load(Ordering::SeqCst), 1);
    assert_eq!(receive.as_mut().poll(&mut Context::from_waker(&w2)), Poll::Ready(Some(42)));

    let (send, receive) = signal::<u32>();
    let mut receive = Box::pin(receive);
    assert_eq!(receive.as_mut().poll(&mut Context::from_waker(&w2)), Poll::Pending);
    drop(send);
    assert_eq!(second.0.load(Ordering::SeqCst), 2);
    assert_eq!(receive.as_mut().poll(&mut Context::from_waker(&w2)), Poll::Ready(None));

    let (send, receive) = signal::<String>();
    drop(receive);
    assert_eq!(send.send("returned".into()), Err("returned".into()));

    // Producer cannot send until the first Pending has been observed.
    let (send, receive) = signal::<i32>();
    let (start, wait) = std::sync::mpsc::channel();
    let producer = std::thread::spawn(move || {
        wait.recv_timeout(LIMIT).unwrap();
        send.send(-7).unwrap();
    });
    let mut receive = Box::pin(receive);
    let mut start = Some(start);
    let observed = poll_fn(|cx| {
        let state = receive.as_mut().poll(cx);
        if state.is_pending() {
            if let Some(start) = start.take() {
                start.send(()).unwrap();
            }
        }
        state
    });
    let (result, polls) = run_until(observed, LIMIT);
    producer.join().unwrap();
    assert_eq!(result, Ok(Some(-7)));
    assert_eq!(polls, 2, "one initial poll and one actual wake, not busy polling");
}

#[test]
fn step6_pin_box_moves_handle_not_pinned_object() {
    let probe = pin_probe(31);
    let address: *const PinProbe = probe.as_ref().get_ref();
    let mut moved = probe;
    assert_eq!(address, moved.as_ref().get_ref() as *const PinProbe);
    let mut cx = Context::from_waker(Waker::noop());
    assert_eq!(moved.as_mut().poll(&mut cx), Poll::Ready(31));
    assert_eq!(run_until(pin_probe(0), LIMIT), (Ok(0), 1));
}
