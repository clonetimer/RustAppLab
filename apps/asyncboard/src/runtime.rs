//! 教学用的最小异步执行器 + 单线程定时器驱动。
//! 这是真正的 Future/Poll/Waker，收到唤醒后才再次 poll，不是忙轮询。
//! 它没有网络 I/O reactor、work stealing、TLS、任务隔离或生产环境承诺。
use std::{
    cmp::{Ordering, Reverse},
    collections::BinaryHeap,
    future::Future,
    pin::Pin,
    sync::{mpsc, Arc, Mutex},
    task::{Context, Poll, Wake, Waker},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

struct DelayState {
    ready: bool,
    waker: Option<Waker>,
}

/// 等待未来的时刻。创建 Delay 不会阻塞当前线程，`.await` 才开始轮询。
pub struct Delay {
    state: Arc<Mutex<DelayState>>,
}

impl Future for Delay {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.ready {
            Poll::Ready(())
        } else {
            // 每次被 poll 都更新 waker，避免保存过期的执行上下文。
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

struct Timer {
    when: Instant,
    sequence: usize,
    state: Arc<Mutex<DelayState>>,
}
impl PartialEq for Timer {
    fn eq(&self, other: &Self) -> bool {
        self.when == other.when && self.sequence == other.sequence
    }
}
impl Eq for Timer {}
impl PartialOrd for Timer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Timer {
    fn cmp(&self, other: &Self) -> Ordering {
        self.when
            .cmp(&other.when)
            .then_with(|| self.sequence.cmp(&other.sequence))
    }
}

enum TimerCommand {
    Schedule(Timer),
    Shutdown,
}

/// 不为每个任务启动线程；所有 Delay 共用一个定时器线程。
pub struct TimerDriver {
    tx: mpsc::Sender<TimerCommand>,
    join: Option<JoinHandle<()>>,
    sequence: usize,
}
impl TimerDriver {
    pub fn new() -> Result<Self, String> {
        let (tx, rx) = mpsc::channel();
        let join = thread::Builder::new()
            .name("applab-timer".into())
            .spawn(move || timer_loop(rx))
            .map_err(|e| format!("不能启动定时器驱动：{e}"))?;
        Ok(Self {
            tx,
            join: Some(join),
            sequence: 0,
        })
    }

    pub fn delay(&mut self, duration: Duration) -> Result<Delay, String> {
        let state = Arc::new(Mutex::new(DelayState {
            ready: false,
            waker: None,
        }));
        self.sequence += 1;
        self.tx
            .send(TimerCommand::Schedule(Timer {
                when: Instant::now() + duration,
                sequence: self.sequence,
                state: Arc::clone(&state),
            }))
            .map_err(|_| "定时器驱动已经退出".to_string())?;
        Ok(Delay { state })
    }
}
impl Drop for TimerDriver {
    fn drop(&mut self) {
        let _ = self.tx.send(TimerCommand::Shutdown);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

fn timer_loop(rx: mpsc::Receiver<TimerCommand>) {
    let mut deadlines: BinaryHeap<Reverse<Timer>> = BinaryHeap::new();
    loop {
        let now = Instant::now();
        while deadlines.peek().is_some_and(|t| t.0.when <= now) {
            let Reverse(timer) = deadlines.pop().expect("peek was Some");
            let waker = {
                let mut state = timer.state.lock().unwrap_or_else(|e| e.into_inner());
                state.ready = true;
                state.waker.take()
            };
            if let Some(waker) = waker {
                waker.wake();
            }
        }
        let event = match deadlines.peek() {
            Some(Reverse(next)) => rx
                .recv_timeout(next.when.saturating_duration_since(Instant::now()))
                .map_err(|e| match e {
                    mpsc::RecvTimeoutError::Timeout => false,
                    mpsc::RecvTimeoutError::Disconnected => true,
                }),
            None => rx.recv().map_err(|_| true),
        };
        match event {
            Ok(TimerCommand::Schedule(timer)) => deadlines.push(Reverse(timer)),
            Ok(TimerCommand::Shutdown) | Err(true) => break,
            Err(false) => {} // 恰好到期，再检查堆顶
        }
    }
}

struct TaskWake {
    id: usize,
    tx: mpsc::Sender<usize>,
}
impl Wake for TaskWake {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        let _ = self.tx.send(self.id);
    }
}

/// 单线程执行器：初始 poll 一次；Pending 后必须等 Waker 通知才再次 poll。
/// 仅为教学，不能替代 Tokio。deadline 限制等待唤醒与轮询间的时间。
/// 这是协作式限制，不能打断单次 poll 内的阻塞、死循环或外部副作用。
pub fn drive<F>(futures: Vec<F>, deadline: Duration) -> Result<Vec<F::Output>, String>
where
    F: Future,
{
    let (tx, rx) = mpsc::channel::<usize>();
    let wakers: Vec<Waker> = (0..futures.len())
        .map(|id| Waker::from(Arc::new(TaskWake { id, tx: tx.clone() })))
        .collect();
    let mut tasks: Vec<Option<Pin<Box<F>>>> =
        futures.into_iter().map(|f| Some(Box::pin(f))).collect();
    let total = tasks.len();
    let mut completed = Vec::with_capacity(total);
    let end = Instant::now() + deadline;
    for id in 0..total {
        tx.send(id).map_err(|e| e.to_string())?;
    }
    while completed.len() < total {
        let time_left = end.saturating_duration_since(Instant::now());
        if time_left.is_zero() {
            return Err("异步任务到达执行上限；检查 Future 是否正确唤醒".into());
        }
        let id = rx.recv_timeout(time_left).map_err(|e| match e {
            mpsc::RecvTimeoutError::Timeout => "等待任务唤醒超时；检查 .await 与 Waker".to_string(),
            mpsc::RecvTimeoutError::Disconnected => "执行队列意外断开".to_string(),
        })?;
        if let Some(task) = tasks[id].as_mut() {
            let mut cx = Context::from_waker(&wakers[id]);
            if let Poll::Ready(result) = task.as_mut().poll(&mut cx) {
                tasks[id] = None;
                completed.push(result);
            }
        }
    }
    Ok(completed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn pending_is_not_busy_polled() {
        struct Counter<'a> {
            timer: Delay,
            polls: &'a AtomicUsize,
        }
        impl Future for Counter<'_> {
            type Output = ();
            fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
                self.polls.fetch_add(1, Ordering::Relaxed);
                Pin::new(&mut self.timer).poll(cx)
            }
        }
        let mut driver = TimerDriver::new().unwrap();
        let polls = AtomicUsize::new(0);
        let tasks = vec![Counter {
            timer: driver.delay(Duration::from_millis(30)).unwrap(),
            polls: &polls,
        }];
        drive(tasks, Duration::from_secs(2)).unwrap();
        assert!(polls.load(Ordering::Relaxed) <= 3);
    }

    #[test]
    fn immediate_task_finishes_without_timer() {
        let out = drive(vec![async { 42 }], Duration::from_millis(500)).unwrap();
        assert_eq!(out, vec![42]);
    }
    #[test]
    fn unpolled_async_body_is_lazy() {
        let runs = AtomicUsize::new(0);
        let future = async {
            runs.fetch_add(1, Ordering::SeqCst);
            7
        };
        assert_eq!(runs.load(Ordering::SeqCst), 0);
        assert_eq!(drive(vec![future], Duration::from_secs(1)).unwrap(), [7]);
        assert_eq!(runs.load(Ordering::SeqCst), 1);
    }
    #[test]
    fn pending_without_a_wake_times_out_without_busy_polling() {
        struct Silent<'a>(&'a AtomicUsize);
        impl Future for Silent<'_> {
            type Output = ();
            fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<()> {
                self.0.fetch_add(1, Ordering::SeqCst);
                Poll::Pending
            }
        }
        let polls = AtomicUsize::new(0);
        assert!(drive(vec![Silent(&polls)], Duration::from_millis(20)).is_err());
        assert_eq!(polls.load(Ordering::SeqCst), 1);
    }
}
