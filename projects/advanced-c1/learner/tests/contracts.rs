use netfetch_c1::{download_batch, fetch_one, validate_jobs, Error, Job, Limits};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::watch,
};

fn job(url: &str) -> Job {
    Job {
        name: "one.txt".into(),
        url: url.into(),
    }
}
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "c1-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

async fn server(response: &'static [u8], delay: Duration) -> (String, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/test", listener.local_addr().unwrap());
    let handle = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut request = [0u8; 4096];
        let _ = stream.read(&mut request).await;
        tokio::time::sleep(delay).await;
        let _ = stream.write_all(response).await;
    });
    (url, handle)
}

#[test]
fn c1_1_accepts_loopback_and_valid_limits() {
    for url in ["http://127.0.0.1:1234/a", "http://[::1]:1234/a"] {
        validate_jobs(&[job(url)], Limits::default()).unwrap();
    }
}
#[test]
fn c1_1_rejects_external_dns_credentials_and_fragment() {
    for url in [
        "https://127.0.0.1/a",
        "http://example.com/a",
        "http://localhost/a",
        "http://192.168.1.1/a",
        "file:///etc/passwd",
        "http://u:p@127.0.0.1/a",
        "http://127.0.0.1/a#frag",
    ] {
        assert!(
            validate_jobs(&[job(url)], Limits::default()).is_err(),
            "{url}"
        );
    }
}
#[test]
fn c1_1_rejects_unsafe_and_duplicate_names() {
    for name in [
        "../x",
        "CON.txt",
        "nul",
        "x/y",
        "x\\y",
        "x:",
        ".",
        "..",
        "trailing.",
        "a b",
        "",
    ] {
        let mut item = job("http://127.0.0.1/a");
        item.name = name.into();
        assert!(validate_jobs(&[item], Limits::default()).is_err(), "{name}");
    }
    let a = job("http://127.0.0.1/a");
    let mut b = a.clone();
    b.name = "ONE.txt".into();
    assert!(validate_jobs(&[a, b], Limits::default()).is_err());
}
#[test]
fn c1_1_checks_job_count_and_limits() {
    assert!(validate_jobs(&[], Limits::default()).is_err());
    let jobs: Vec<_> = (0..65)
        .map(|i| Job {
            name: format!("x{i}.txt"),
            url: "http://127.0.0.1/a".into(),
        })
        .collect();
    assert!(validate_jobs(&jobs, Limits::default()).is_err());
    validate_jobs(&jobs[..64], Limits::default()).unwrap();
    for concurrency in [0, 9] {
        assert!(validate_jobs(
            &jobs[..1],
            Limits {
                concurrency,
                ..Limits::default()
            }
        )
        .is_err());
    }
    for max_bytes in [0, 8 * 1024 * 1024 + 1] {
        assert!(validate_jobs(
            &jobs[..1],
            Limits {
                max_bytes,
                ..Limits::default()
            }
        )
        .is_err());
    }
    assert!(validate_jobs(
        &jobs[..1],
        Limits {
            total_timeout: Duration::ZERO,
            ..Limits::default()
        }
    )
    .is_err());
}
#[tokio::test]
async fn c1_2_reads_real_http_bytes() {
    let (url, task) = server(
        b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\nConnection: close\r\n\r\nabc",
        Duration::ZERO,
    )
    .await;
    let (_tx, rx) = watch::channel(false);
    assert_eq!(
        fetch_one(job(&url), Limits::default(), rx).await.unwrap(),
        b"abc"
    );
    task.await.unwrap();
}
#[tokio::test]
async fn c1_2_rejects_http_errors_and_redirects() {
    for response in [
        b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n".as_slice(),
        b"HTTP/1.1 302 Found\r\nLocation: http://example.com/\r\nContent-Length: 0\r\n\r\n"
            .as_slice(),
    ] {
        let (url, task) = server(response, Duration::ZERO).await;
        let (_tx, rx) = watch::channel(false);
        assert!(matches!(
            fetch_one(job(&url), Limits::default(), rx).await,
            Err(Error::Http(_))
        ));
        task.await.unwrap();
    }
}
#[tokio::test]
async fn c1_2_limits_body_even_without_content_length() {
    let (url, task) = server(
        b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\nabcdef",
        Duration::ZERO,
    )
    .await;
    let (_tx, rx) = watch::channel(false);
    assert_eq!(
        fetch_one(
            job(&url),
            Limits {
                max_bytes: 3,
                ..Limits::default()
            },
            rx
        )
        .await,
        Err(Error::TooLarge)
    );
    task.await.unwrap();
}
#[tokio::test]
async fn c1_2_times_out_waiting_for_headers() {
    let (url, task) = server(
        b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n",
        Duration::from_secs(2),
    )
    .await;
    let (_tx, rx) = watch::channel(false);
    assert_eq!(
        fetch_one(
            job(&url),
            Limits {
                total_timeout: Duration::from_millis(200),
                ..Limits::default()
            },
            rx
        )
        .await,
        Err(Error::Timeout)
    );
    task.abort();
    let _ = task.await;
}
#[tokio::test]
async fn c1_2_pre_cancelled_never_connects() {
    let (_tx, rx) = watch::channel(true);
    assert_eq!(
        fetch_one(job("http://127.0.0.1:1/a"), Limits::default(), rx).await,
        Err(Error::Cancelled)
    );
}
#[tokio::test]
async fn c1_3_publishes_a_new_directory_and_receipts() {
    let tmp = Scratch::new();
    let dest = tmp.0.join("output");
    let (url, task) = server(
        b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\nabc",
        Duration::ZERO,
    )
    .await;
    let (_tx, rx) = watch::channel(false);
    let result = download_batch(vec![job(&url)], &dest, Limits::default(), rx)
        .await
        .unwrap();
    assert_eq!(result[0].bytes, 3);
    assert_eq!(fs::read(dest.join("one.txt")).unwrap(), b"abc");
    assert_eq!(fs::read_dir(&tmp.0).unwrap().count(), 1);
    task.await.unwrap();
}
#[tokio::test]
async fn c1_3_failure_leaves_no_partial_output() {
    let tmp = Scratch::new();
    let dest = tmp.0.join("output");
    let (url, task) = server(
        b"HTTP/1.1 500 Failed\r\nContent-Length: 0\r\n\r\n",
        Duration::ZERO,
    )
    .await;
    let (_tx, rx) = watch::channel(false);
    assert!(
        download_batch(vec![job(&url)], &dest, Limits::default(), rx)
            .await
            .is_err()
    );
    assert!(!dest.exists());
    assert_eq!(fs::read_dir(&tmp.0).unwrap().count(), 0);
    task.abort();
    let _ = task.await;
}
#[tokio::test]
async fn c1_3_existing_destination_is_unchanged() {
    let tmp = Scratch::new();
    let dest = tmp.0.join("output");
    fs::create_dir(&dest).unwrap();
    fs::write(dest.join("keep.txt"), b"keep").unwrap();
    let (_tx, rx) = watch::channel(false);
    assert!(download_batch(
        vec![job("http://127.0.0.1:1/a")],
        &dest,
        Limits::default(),
        rx
    )
    .await
    .is_err());
    assert_eq!(fs::read(dest.join("keep.txt")).unwrap(), b"keep");
    assert_eq!(fs::read_dir(&tmp.0).unwrap().count(), 1);
}

#[test]
fn c1_1_checks_every_timeout_boundary() {
    for bad in [Duration::ZERO, Duration::from_secs(61)] {
        for limits in [
            Limits {
                connect_timeout: bad,
                ..Limits::default()
            },
            Limits {
                idle_timeout: bad,
                ..Limits::default()
            },
            Limits {
                total_timeout: bad,
                ..Limits::default()
            },
        ] {
            assert!(validate_jobs(&[job("http://127.0.0.1/a")], limits).is_err());
        }
    }
}
#[tokio::test]
async fn c1_2_sender_dropped_false_does_not_cancel_or_spin() {
    let (url, task) = server(
        b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\nabc",
        Duration::from_millis(10),
    )
    .await;
    let (tx, rx) = watch::channel(false);
    drop(tx);
    assert_eq!(
        fetch_one(job(&url), Limits::default(), rx).await.unwrap(),
        b"abc"
    );
    task.await.unwrap();
}
#[tokio::test]
async fn c1_2_chunked_body_is_bounded() {
    let (url, task) = server(
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n3\r\nabc\r\n3\r\ndef\r\n0\r\n\r\n",
        Duration::ZERO,
    )
    .await;
    let (_tx, rx) = watch::channel(false);
    assert_eq!(
        fetch_one(
            job(&url),
            Limits {
                max_bytes: 5,
                ..Limits::default()
            },
            rx
        )
        .await,
        Err(Error::TooLarge)
    );
    task.await.unwrap();
}
#[tokio::test]
async fn c1_2_truncated_body_is_not_success() {
    let (url, task) = server(
        b"HTTP/1.1 200 OK\r\nContent-Length: 9\r\nConnection: close\r\n\r\nabc",
        Duration::ZERO,
    )
    .await;
    let (_tx, rx) = watch::channel(false);
    assert!(matches!(
        fetch_one(job(&url), Limits::default(), rx).await,
        Err(Error::Transport(_))
    ));
    task.await.unwrap();
}
#[tokio::test]
async fn c1_2_body_idle_and_whole_response_deadlines_are_distinct() {
    for (idle, total, expected) in [(100, 1500, Error::IdleTimeout), (1500, 100, Error::Timeout)] {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/body", listener.local_addr().unwrap());
        let task = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = [0; 4096];
            let _ = stream.read(&mut request).await;
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\na")
                .await
                .unwrap();
            tokio::time::sleep(Duration::from_secs(3)).await;
            let _ = stream.write_all(b"b").await;
        });
        let (_tx, rx) = watch::channel(false);
        let limits = Limits {
            idle_timeout: Duration::from_millis(idle),
            total_timeout: Duration::from_millis(total),
            ..Limits::default()
        };
        assert_eq!(fetch_one(job(&url), limits, rx).await, Err(expected));
        task.abort();
        let _ = task.await;
    }
}
#[tokio::test]
async fn c1_2_total_deadline_is_not_reset_by_body_progress() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/drip", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut request = [0; 4096];
        let _ = stream.read(&mut request).await;
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 20\r\n\r\n")
            .await
            .unwrap();
        for _ in 0..20 {
            if stream.write_all(b"a").await.is_err() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    });
    let (_tx, rx) = watch::channel(false);
    assert_eq!(
        fetch_one(
            job(&url),
            Limits {
                total_timeout: Duration::from_millis(250),
                idle_timeout: Duration::from_secs(2),
                ..Limits::default()
            },
            rx
        )
        .await,
        Err(Error::Timeout)
    );
    task.abort();
    let _ = task.await;
}

// Single-thread runtime: a serial/blocking queue cannot reach the release barrier.
#[tokio::test]
async fn c1_3_observed_concurrency_is_bounded_and_receipts_keep_input_order() {
    for n in [1, 2, 8] {
        let tmp = Scratch::new();
        let destination = tmp.0.join("output");
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let active = std::sync::Arc::new(AtomicUsize::new(0));
        let peak = std::sync::Arc::new(AtomicUsize::new(0));
        let gate = std::sync::Arc::new(tokio::sync::Semaphore::new(0));
        let (seen_tx, mut seen_rx) = tokio::sync::mpsc::unbounded_channel();
        let a = active.clone();
        let p = peak.clone();
        let g = gate.clone();
        let host = tokio::spawn(async move {
            let mut handlers = tokio::task::JoinSet::new();
            for _ in 0..16 {
                let (mut stream, _) = listener.accept().await.unwrap();
                let count = a.fetch_add(1, Ordering::SeqCst) + 1;
                p.fetch_max(count, Ordering::SeqCst);
                seen_tx.send(()).unwrap();
                let a = a.clone();
                let g = g.clone();
                handlers.spawn(async move {
                    let mut request = [0; 4096];
                    let _ = stream.read(&mut request).await;
                    let permit = g.acquire().await.unwrap();
                    permit.forget();
                    a.fetch_sub(1, Ordering::SeqCst);
                    stream
                        .write_all(
                            b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\nConnection: close\r\n\r\nabc",
                        )
                        .await
                        .unwrap();
                });
            }
            while let Some(result) = handlers.join_next().await {
                result.unwrap();
            }
        });
        let jobs: Vec<_> = (0..16)
            .rev()
            .map(|i| Job {
                name: format!("{i:02}.bin"),
                url: format!("http://{address}/{i}"),
            })
            .collect();
        let names: Vec<_> = jobs.iter().map(|j| j.name.clone()).collect();
        let (_tx, rx) = watch::channel(false);
        let d = destination.clone();
        let batch = tokio::spawn(download_owned(
            jobs,
            d,
            Limits {
                concurrency: n,
                ..Limits::default()
            },
            rx,
        ));
        for _ in 0..n {
            tokio::time::timeout(Duration::from_secs(3), seen_rx.recv())
                .await
                .expect("queue did not execute concurrently")
                .unwrap();
        }
        assert_eq!(peak.load(Ordering::SeqCst), n);
        gate.add_permits(16);
        let results = batch.await.unwrap().unwrap();
        host.await.unwrap();
        assert!(peak.load(Ordering::SeqCst) <= n);
        assert_eq!(
            results.iter().map(|r| r.name.clone()).collect::<Vec<_>>(),
            names
        );
        assert_eq!(fs::read_dir(&destination).unwrap().count(), 16);
    }
}
async fn download_owned(
    jobs: Vec<Job>,
    path: PathBuf,
    limits: Limits,
    rx: watch::Receiver<bool>,
) -> Result<Vec<netfetch_c1::Receipt>, Error> {
    download_batch(jobs, &path, limits, rx).await
}
#[tokio::test]
async fn c1_3_cancel_stops_queue_and_cleans_staging() {
    let tmp = Scratch::new();
    let dest = tmp.0.join("output");
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/wait", listener.local_addr().unwrap());
    let jobs = (0..8)
        .map(|i| Job {
            name: format!("{i}.bin"),
            url: url.clone(),
        })
        .collect();
    let (tx, rx) = watch::channel(false);
    let batch = tokio::spawn(download_owned(
        jobs,
        dest.clone(),
        Limits {
            concurrency: 1,
            ..Limits::default()
        },
        rx,
    ));
    let (_socket, _) = tokio::time::timeout(Duration::from_secs(3), listener.accept())
        .await
        .unwrap()
        .unwrap();
    tx.send(true).unwrap();
    assert_eq!(
        tokio::time::timeout(Duration::from_secs(3), batch)
            .await
            .unwrap()
            .unwrap(),
        Err(Error::Cancelled)
    );
    assert!(!dest.exists());
    assert_eq!(fs::read_dir(&tmp.0).unwrap().count(), 0);
    assert!(
        tokio::time::timeout(Duration::from_millis(100), listener.accept())
            .await
            .is_err()
    );
}
#[tokio::test]
async fn c1_3_dropped_outer_future_cleans_staging_without_file_writers() {
    let tmp = Scratch::new();
    let dest = tmp.0.join("output");
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/wait", listener.local_addr().unwrap());
    let (_tx, rx) = watch::channel(false);
    let batch = tokio::spawn(download_owned(
        vec![job(&url)],
        dest.clone(),
        Limits::default(),
        rx,
    ));
    let (_socket, _) = tokio::time::timeout(Duration::from_secs(3), listener.accept())
        .await
        .unwrap()
        .unwrap();
    batch.abort();
    assert!(batch.await.unwrap_err().is_cancelled());
    assert!(!dest.exists());
    assert_eq!(fs::read_dir(&tmp.0).unwrap().count(), 0);
}
#[tokio::test]
async fn c1_3_bad_plan_and_precancel_create_nothing() {
    let tmp = Scratch::new();
    let dest = tmp.0.join("output");
    let (_tx, rx) = watch::channel(true);
    assert_eq!(
        download_batch(
            vec![job("http://127.0.0.1:1/a")],
            &dest,
            Limits::default(),
            rx
        )
        .await,
        Err(Error::Cancelled)
    );
    assert_eq!(fs::read_dir(&tmp.0).unwrap().count(), 0);
    let (_tx, rx) = watch::channel(false);
    assert!(download_batch(
        vec![job("http://example.com/")],
        &dest,
        Limits::default(),
        rx
    )
    .await
    .is_err());
    assert_eq!(fs::read_dir(&tmp.0).unwrap().count(), 0);
}
#[test]
fn c1_3_real_cli_demo_and_error_contract() {
    let tmp = Scratch::new();
    let dest = tmp.0.join("中文 output");
    let exe = env!("CARGO_BIN_EXE_netfetch-c1");
    let output = std::process::Command::new(exe)
        .arg("demo")
        .arg(&dest)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout).lines().count(), 3);
    assert_eq!(
        fs::read_to_string(dest.join("001.bin")).unwrap(),
        "Rust 异步下载\n"
    );
    let failed = std::process::Command::new(exe)
        .arg("demo")
        .arg(&dest)
        .output()
        .unwrap();
    assert!(!failed.status.success());
    assert!(failed.stdout.is_empty());
    assert!(!failed.stderr.is_empty());
    for args in [
        vec!["demo"],
        vec!["download"],
        vec!["unknown"],
        vec!["--help", "extra"],
    ] {
        let failed = std::process::Command::new(exe).args(args).output().unwrap();
        assert!(!failed.status.success());
        assert!(failed.stdout.is_empty());
    }
}
