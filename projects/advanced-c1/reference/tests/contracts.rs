use netfetch_c1::{download_batch, fetch_one, validate_jobs, Error, Job, Limits};
use std::{fs, path::PathBuf, sync::atomic::{AtomicUsize, Ordering}, time::Duration};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpListener, sync::watch};

fn job(url: &str) -> Job { Job { name: "one.txt".into(), url: url.into() } }
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!("c1-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::SeqCst)));
        fs::create_dir(&p).unwrap(); Self(p)
    }
}
impl Drop for Scratch { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }

async fn server(response: &'static [u8], delay: Duration) -> (String, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/test", listener.local_addr().unwrap());
    let handle = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut request = [0u8; 4096]; let _ = stream.read(&mut request).await;
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
    for url in ["https://127.0.0.1/a", "http://example.com/a", "http://localhost/a", "http://192.168.1.1/a", "file:///etc/passwd", "http://u:p@127.0.0.1/a", "http://127.0.0.1/a#frag"] {
        assert!(validate_jobs(&[job(url)], Limits::default()).is_err(), "{url}");
    }
}
#[test]
fn c1_1_rejects_unsafe_and_duplicate_names() {
    for name in ["../x", "CON.txt", "nul", "x/y", "x\\y", "x:", ".", "..", "trailing.", "a b", ""] {
        let mut item = job("http://127.0.0.1/a"); item.name = name.into();
        assert!(validate_jobs(&[item], Limits::default()).is_err(), "{name}");
    }
    let a = job("http://127.0.0.1/a"); let mut b = a.clone(); b.name = "ONE.txt".into();
    assert!(validate_jobs(&[a, b], Limits::default()).is_err());
}
#[test]
fn c1_1_checks_job_count_and_limits() {
    assert!(validate_jobs(&[], Limits::default()).is_err());
    let jobs: Vec<_> = (0..65).map(|i| Job { name: format!("x{i}.txt"), url: "http://127.0.0.1/a".into() }).collect();
    assert!(validate_jobs(&jobs, Limits::default()).is_err());
    validate_jobs(&jobs[..64], Limits::default()).unwrap();
    for concurrency in [0, 9] {
        assert!(validate_jobs(&jobs[..1], Limits {concurrency, ..Limits::default()}).is_err());
    }
    for max_bytes in [0, 8*1024*1024+1] {
        assert!(validate_jobs(&jobs[..1], Limits {max_bytes, ..Limits::default()}).is_err());
    }
    assert!(validate_jobs(&jobs[..1], Limits {total_timeout: Duration::ZERO, ..Limits::default()}).is_err());
}
#[tokio::test]
async fn c1_2_reads_real_http_bytes() {
    let (url, task) = server(b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\nConnection: close\r\n\r\nabc", Duration::ZERO).await;
    let (_tx, rx) = watch::channel(false);
    assert_eq!(fetch_one(job(&url), Limits::default(), rx).await.unwrap(), b"abc");
    task.await.unwrap();
}
#[tokio::test]
async fn c1_2_rejects_http_errors_and_redirects() {
    for response in [b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n".as_slice(), b"HTTP/1.1 302 Found\r\nLocation: http://example.com/\r\nContent-Length: 0\r\n\r\n".as_slice()] {
        let (url, task) = server(response, Duration::ZERO).await;
        let (_tx, rx) = watch::channel(false);
        assert!(matches!(fetch_one(job(&url), Limits::default(), rx).await, Err(Error::Http(_))));
        task.await.unwrap();
    }
}
#[tokio::test]
async fn c1_2_limits_body_even_without_content_length() {
    let (url, task) = server(b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\nabcdef", Duration::ZERO).await;
    let (_tx, rx) = watch::channel(false);
    assert_eq!(fetch_one(job(&url), Limits {max_bytes: 3, ..Limits::default()}, rx).await, Err(Error::TooLarge));
    task.await.unwrap();
}
#[tokio::test]
async fn c1_2_times_out_waiting_for_headers() {
    let (url, task) = server(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n", Duration::from_secs(2)).await;
    let (_tx, rx) = watch::channel(false);
    assert_eq!(fetch_one(job(&url), Limits {total_timeout: Duration::from_millis(200), ..Limits::default()}, rx).await, Err(Error::Timeout));
    task.abort(); let _ = task.await;
}
#[tokio::test]
async fn c1_2_pre_cancelled_never_connects() {
    let (_tx, rx) = watch::channel(true);
    assert_eq!(fetch_one(job("http://127.0.0.1:1/a"), Limits::default(), rx).await, Err(Error::Cancelled));
}
#[tokio::test]
async fn c1_3_publishes_a_new_directory_and_receipts() {
    let tmp = Scratch::new(); let dest = tmp.0.join("output");
    let (url, task) = server(b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\nabc", Duration::ZERO).await;
    let (_tx, rx) = watch::channel(false);
    let result = download_batch(vec![job(&url)], &dest, Limits::default(), rx).await.unwrap();
    assert_eq!(result[0].bytes, 3); assert_eq!(fs::read(dest.join("one.txt")).unwrap(), b"abc");
    assert_eq!(fs::read_dir(&tmp.0).unwrap().count(), 1); task.await.unwrap();
}
#[tokio::test]
async fn c1_3_failure_leaves_no_partial_output() {
    let tmp = Scratch::new(); let dest = tmp.0.join("output");
    let (url, task) = server(b"HTTP/1.1 500 Failed\r\nContent-Length: 0\r\n\r\n", Duration::ZERO).await;
    let (_tx, rx) = watch::channel(false);
    assert!(download_batch(vec![job(&url)], &dest, Limits::default(), rx).await.is_err());
    assert!(!dest.exists()); assert_eq!(fs::read_dir(&tmp.0).unwrap().count(), 0);
    task.abort(); let _ = task.await;
}
#[tokio::test]
async fn c1_3_existing_destination_is_unchanged() {
    let tmp = Scratch::new(); let dest = tmp.0.join("output"); fs::create_dir(&dest).unwrap();
    fs::write(dest.join("keep.txt"), b"keep").unwrap();
    let (_tx, rx) = watch::channel(false);
    assert!(download_batch(vec![job("http://127.0.0.1:1/a")], &dest, Limits::default(), rx).await.is_err());
    assert_eq!(fs::read(dest.join("keep.txt")).unwrap(), b"keep");
    assert_eq!(fs::read_dir(&tmp.0).unwrap().count(), 1);
}
