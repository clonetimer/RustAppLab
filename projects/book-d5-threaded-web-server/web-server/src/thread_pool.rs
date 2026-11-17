//! A small fixed-size worker pool built with std::sync::mpsc.

use std::io;
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

/// A fixed-size pool that drains queued jobs and joins every worker on drop.
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    /// Creates exactly size workers. A zero-sized pool is rejected.
    pub fn new(size: usize) -> io::Result<Self> {
        if size == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "thread pool size must be greater than zero",
            ));
        }

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            match Worker::new(id, Arc::clone(&receiver)) {
                Ok(worker) => workers.push(worker),
                Err(error) => {
                    for _ in &workers {
                        let _ = sender.send(Message::Terminate);
                    }
                    for worker in &mut workers {
                        if let Some(thread) = worker.thread.take() {
                            let _ = thread.join();
                        }
                    }
                    return Err(error);
                }
            }
        }

        Ok(Self { workers, sender })
    }

    /// Adds one Send + 'static job to the shared queue.
    pub fn execute<F>(&self, job: F) -> io::Result<()>
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender
            .send(Message::NewJob(Box::new(job)))
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "thread pool is closed"))
    }

    /// Returns the configured worker count.
    pub fn worker_count(&self) -> usize {
        self.workers.len()
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Messages are FIFO: queued jobs are received before termination.
        for _ in &self.workers {
            let _ = self.sender.send(Message::Terminate);
        }

        // Drop does not return while a worker is still executing a job.
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                let _ = thread.join();
            }
        }
    }
}

struct Worker {
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> io::Result<Self> {
        let thread = thread::Builder::new()
            .name(format!("book-d5-worker-{id}"))
            .spawn(move || loop {
                // Release the receiver mutex immediately after recv. A worker
                // must not hold the lock while running user code.
                let message = match receiver.lock() {
                    Ok(receiver) => receiver.recv(),
                    Err(_) => break,
                };

                match message {
                    Ok(Message::NewJob(job)) => job(),
                    Ok(Message::Terminate) | Err(_) => break,
                }
            })?;

        Ok(Self {
            thread: Some(thread),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ThreadPool;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc, Arc, Mutex,
    };
    use std::time::Duration;

    #[test]
    fn rejects_a_zero_sized_pool() {
        assert!(ThreadPool::new(0).is_err());
    }

    #[test]
    fn executes_a_job_and_reports_the_configured_worker_count() {
        let pool = ThreadPool::new(2).unwrap();
        assert_eq!(pool.worker_count(), 2);
        let (sender, receiver) = mpsc::channel();
        pool.execute(move || sender.send(42).unwrap()).unwrap();
        assert_eq!(receiver.recv_timeout(Duration::from_secs(2)).unwrap(), 42);
    }

    #[test]
    fn two_workers_can_start_jobs_before_either_is_released() {
        let pool = ThreadPool::new(2).unwrap();
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let release_rx = Arc::new(Mutex::new(release_rx));

        for _ in 0..2 {
            let started = started_tx.clone();
            let release = Arc::clone(&release_rx);
            pool.execute(move || {
                started.send(()).unwrap();
                release.lock().unwrap().recv().unwrap();
            })
            .unwrap();
        }
        drop(started_tx);

        let first_started = started_rx.recv_timeout(Duration::from_secs(2)).is_ok();
        let second_started = started_rx.recv_timeout(Duration::from_secs(2)).is_ok();

        // Always release both jobs before asserting, so a failed concurrency
        // assertion cannot strand a worker and hang the test process.
        release_tx.send(()).unwrap();
        release_tx.send(()).unwrap();
        drop(pool);

        assert!(first_started && second_started);
    }

    #[test]
    fn drop_drains_queued_jobs_and_joins_workers() {
        let pool = ThreadPool::new(3).unwrap();
        let completed = Arc::new(AtomicUsize::new(0));

        for _ in 0..16 {
            let completed = Arc::clone(&completed);
            pool.execute(move || {
                completed.fetch_add(1, Ordering::Relaxed);
            })
            .unwrap();
        }

        drop(pool);
        assert_eq!(completed.load(Ordering::Relaxed), 16);
    }
}
