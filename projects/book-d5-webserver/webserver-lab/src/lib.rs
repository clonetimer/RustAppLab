//! A small, bounded HTTP/1.1 teaching server built only with the Rust standard library.
//!
//! The implementation intentionally omits TLS, keep-alive, request bodies,
//! chunked transfer encoding, and production-grade HTTP semantics.
//!
//! A submitted job must own everything it uses:
//!
//! ```compile_fail
//! use rustapplab_book_d5_webserver::ThreadPool;
//!
//! let pool = ThreadPool::new(2).unwrap();
//! let borrowed = String::from("not owned by the job");
//! pool.execute(|| println!("{borrowed}")).unwrap();
//! ```

use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

const MAX_HEADER_BYTES: usize = 16 * 1024;
const MAX_LINE_BYTES: usize = 4 * 1024;
const MAX_HEADERS: usize = 64;
const READ_TIMEOUT: Duration = Duration::from_secs(3);

/// A parsed HTTP request. Header names are normalized to lowercase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    /// HTTP method, such as GET.
    pub method: String,
    /// Origin-form request target, such as / or /sleep.
    pub target: String,
    /// HTTP protocol version.
    pub version: String,
    /// Normalized header names and trimmed values.
    pub headers: Vec<(String, String)>,
}

/// Errors returned by the deliberately small HTTP request parser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestError {
    /// The complete header exceeded a configured size limit.
    TooLarge,
    /// The client closed the stream before the header terminator.
    Incomplete,
    /// Request bytes were not valid UTF-8.
    InvalidEncoding,
    /// The request line was malformed or outside the supported subset.
    InvalidRequestLine,
    /// A header line was malformed.
    InvalidHeader,
    /// A request body or transfer encoding is not supported.
    UnsupportedBody,
    /// Only HTTP/1.1 is supported by this lab.
    UnsupportedVersion,
    /// A socket read failed.
    Io(io::ErrorKind),
}

/// Parse a complete request header from bytes.
///
/// This parser accepts an HTTP/1.1 request line, CRLF-delimited headers, and
/// an empty body. It rejects oversized input and non-zero Content-Length.
pub fn parse_request(bytes: &[u8]) -> Result<Request, RequestError> {
    if bytes.len() > MAX_HEADER_BYTES {
        return Err(RequestError::TooLarge);
    }

    let text = std::str::from_utf8(bytes).map_err(|_| RequestError::InvalidEncoding)?;
    let header_end = text.find("\r\n\r\n").ok_or(RequestError::Incomplete)?;
    if header_end + 4 != text.len() {
        return Err(RequestError::UnsupportedBody);
    }

    let head = &text[..header_end];
    let mut lines = head.split("\r\n");
    let request_line = lines.next().ok_or(RequestError::InvalidRequestLine)?;
    if request_line.len() > MAX_LINE_BYTES {
        return Err(RequestError::TooLarge);
    }

    let mut parts = request_line.split_whitespace();
    let method = parts.next().ok_or(RequestError::InvalidRequestLine)?;
    let target = parts.next().ok_or(RequestError::InvalidRequestLine)?;
    let version = parts.next().ok_or(RequestError::InvalidRequestLine)?;
    if parts.next().is_some()
        || method.is_empty()
        || !method.bytes().all(|byte| byte.is_ascii_alphabetic())
        || !target.starts_with('/')
    {
        return Err(RequestError::InvalidRequestLine);
    }
    if version != "HTTP/1.1" {
        return Err(RequestError::UnsupportedVersion);
    }

    let mut headers = Vec::new();
    for line in lines {
        if line.len() > MAX_LINE_BYTES {
            return Err(RequestError::TooLarge);
        }
        if headers.len() >= MAX_HEADERS {
            return Err(RequestError::TooLarge);
        }
        let (name, value) = line.split_once(':').ok_or(RequestError::InvalidHeader)?;
        if name.is_empty()
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        {
            return Err(RequestError::InvalidHeader);
        }

        let name = name.to_ascii_lowercase();
        let value = value.trim().to_owned();
        if name == "content-length" {
            let length = value
                .parse::<usize>()
                .map_err(|_| RequestError::InvalidHeader)?;
            if length > 0 {
                return Err(RequestError::UnsupportedBody);
            }
        }
        if name == "transfer-encoding" {
            return Err(RequestError::UnsupportedBody);
        }
        headers.push((name, value));
    }

    Ok(Request {
        method: method.to_owned(),
        target: target.to_owned(),
        version: version.to_owned(),
        headers,
    })
}

fn read_request<R: BufRead>(reader: &mut R) -> Result<Request, RequestError> {
    let mut bytes = Vec::new();

    loop {
        let mut line = Vec::new();
        let limit = (MAX_LINE_BYTES + 1) as u64;
        let count = reader
            .by_ref()
            .take(limit)
            .read_until(b'\n', &mut line)
            .map_err(|error| RequestError::Io(error.kind()))?;

        if count == 0 {
            return Err(RequestError::Incomplete);
        }
        if line.len() > MAX_LINE_BYTES || bytes.len().saturating_add(line.len()) > MAX_HEADER_BYTES
        {
            return Err(RequestError::TooLarge);
        }

        let blank_line = line == b"\r\n";
        bytes.extend_from_slice(&line);
        if blank_line {
            break;
        }
    }

    parse_request(&bytes)
}

#[derive(Debug, PartialEq, Eq)]
struct Response {
    status: u16,
    reason: &'static str,
    content_type: &'static str,
    body: Vec<u8>,
    extra_headers: Vec<(&'static str, &'static str)>,
}

impl Response {
    fn new(status: u16, reason: &'static str, content_type: &'static str, body: Vec<u8>) -> Self {
        Self {
            status,
            reason,
            content_type,
            body,
            extra_headers: Vec::new(),
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut head = format!(
            "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n",
            self.status,
            self.reason,
            self.content_type,
            self.body.len()
        );
        for (name, value) in &self.extra_headers {
            head.push_str(&format!("{name}: {value}\r\n"));
        }
        head.push_str("\r\n");

        let mut bytes = head.into_bytes();
        bytes.extend_from_slice(&self.body);
        bytes
    }
}

fn route(request: &Request) -> Response {
    if request.method != "GET" {
        let mut response = Response::new(
            405,
            "Method Not Allowed",
            "text/plain; charset=utf-8",
            b"only GET is supported\n".to_vec(),
        );
        response.extra_headers.push(("Allow", "GET"));
        return response;
    }

    let path = request.target.split('?').next().unwrap_or("/");
    match path {
        "/" => Response::new(
            200,
            "OK",
            "text/html; charset=utf-8",
            include_str!("../samples/index.html").as_bytes().to_vec(),
        ),
        "/sleep" => {
            // This deliberately slow route makes worker-pool concurrency observable.
            thread::sleep(Duration::from_millis(200));
            Response::new(
                200,
                "OK",
                "text/plain; charset=utf-8",
                b"slow route finished\n".to_vec(),
            )
        }
        _ => Response::new(
            404,
            "Not Found",
            "text/plain; charset=utf-8",
            b"not found\n".to_vec(),
        ),
    }
}

fn error_response(error: RequestError) -> Response {
    match error {
        RequestError::TooLarge => Response::new(
            431,
            "Request Header Fields Too Large",
            "text/plain; charset=utf-8",
            b"request headers too large\n".to_vec(),
        ),
        RequestError::Io(io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock) => Response::new(
            408,
            "Request Timeout",
            "text/plain; charset=utf-8",
            b"request timed out\n".to_vec(),
        ),
        _ => Response::new(
            400,
            "Bad Request",
            "text/plain; charset=utf-8",
            b"bad request\n".to_vec(),
        ),
    }
}

fn handle_connection(mut stream: TcpStream) {
    let _ = stream.set_read_timeout(Some(READ_TIMEOUT));
    let request = {
        let mut reader = BufReader::new(&stream);
        read_request(&mut reader)
    };
    let response = match request {
        Ok(request) => route(&request),
        Err(error) => error_response(error),
    };
    let bytes = response.to_bytes();
    let _ = stream.write_all(&bytes);
    let _ = stream.flush();
}

/// Handle a finite number of connections sequentially.
///
/// This is the single-threaded baseline for comparing the worker-pool server.
pub fn serve_sequential(listener: TcpListener, max_connections: usize) -> io::Result<()> {
    for stream in listener.incoming().take(max_connections) {
        handle_connection(stream?);
    }
    Ok(())
}

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Shutdown,
}

/// Failure modes when creating or submitting work to a thread pool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PoolError {
    /// A pool with no workers cannot make progress.
    ZeroWorkers,
    /// The operating system could not create a worker thread.
    ThreadSpawn(String),
    /// All worker receivers have been disconnected.
    ShuttingDown,
}

impl std::fmt::Display for PoolError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZeroWorkers => write!(formatter, "worker count must be greater than zero"),
            Self::ThreadSpawn(message) => write!(formatter, "could not create worker: {message}"),
            Self::ShuttingDown => write!(formatter, "thread pool is shutting down"),
        }
    }
}

impl std::error::Error for PoolError {}

struct Worker {
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Message>>>) -> io::Result<Self> {
        let thread = thread::Builder::new()
            .name(format!("book-d5-worker-{id}"))
            .spawn(move || loop {
                let message = {
                    let guard = match receiver.lock() {
                        Ok(guard) => guard,
                        Err(_) => break,
                    };
                    guard.recv()
                };

                match message {
                    Ok(Message::NewJob(job)) => job(),
                    Ok(Message::Shutdown) | Err(_) => break,
                }
            })?;

        Ok(Self {
            thread: Some(thread),
        })
    }
}

/// A fixed-size worker pool using an MPSC job queue.
///
/// Dropping the pool queues one shutdown message per worker, then joins every
/// worker. Pending jobs are ahead of shutdown messages in the FIFO queue and
/// therefore finish before workers exit.
pub struct ThreadPool {
    sender: Sender<Message>,
    workers: Vec<Worker>,
}

impl ThreadPool {
    /// Create a pool with exactly size worker threads.
    pub fn new(size: usize) -> Result<Self, PoolError> {
        if size == 0 {
            return Err(PoolError::ZeroWorkers);
        }

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            match Worker::new(id, Arc::clone(&receiver)) {
                Ok(worker) => workers.push(worker),
                Err(error) => {
                    for _ in 0..workers.len() {
                        let _ = sender.send(Message::Shutdown);
                    }
                    for worker in &mut workers {
                        if let Some(thread) = worker.thread.take() {
                            let _ = thread.join();
                        }
                    }
                    return Err(PoolError::ThreadSpawn(error.to_string()));
                }
            }
        }

        Ok(Self { sender, workers })
    }

    /// Number of worker threads owned by this pool.
    pub fn worker_count(&self) -> usize {
        self.workers.len()
    }

    /// Submit one owned, one-shot job to the worker queue.
    pub fn execute<F>(&self, job: F) -> Result<(), PoolError>
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender
            .send(Message::NewJob(Box::new(job)))
            .map_err(|_| PoolError::ShuttingDown)
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Send every termination message before joining any worker. Joining
        // inside this loop can deadlock if a different worker receives it.
        for _ in &self.workers {
            let _ = self.sender.send(Message::Shutdown);
        }
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                let _ = thread.join();
            }
        }
    }
}

/// Accept connections concurrently until the shutdown channel receives a value
/// or its sender is dropped. On return, the pool drains queued jobs and joins.
pub fn serve(listener: TcpListener, pool: ThreadPool, shutdown: Receiver<()>) -> io::Result<()> {
    listener.set_nonblocking(true)?;

    loop {
        match shutdown.try_recv() {
            Ok(()) | Err(TryRecvError::Disconnected) => break,
            Err(TryRecvError::Empty) => {}
        }

        match listener.accept() {
            Ok((stream, _address)) => {
                pool.execute(move || handle_connection(stream))
                    .map_err(|error| {
                        io::Error::new(io::ErrorKind::BrokenPipe, error.to_string())
                    })?;
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                match shutdown.recv_timeout(Duration::from_millis(10)) {
                    Ok(()) | Err(RecvTimeoutError::Disconnected) => break,
                    Err(RecvTimeoutError::Timeout) => {}
                }
            }
            Err(error) => return Err(error),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::{Shutdown, SocketAddr};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Condvar;

    fn valid_request() -> &'static [u8] {
        b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"
    }

    fn request(address: SocketAddr, bytes: &[u8]) -> String {
        let mut stream = TcpStream::connect(address).expect("connect to test server");
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        stream.write_all(bytes).unwrap();
        stream.shutdown(Shutdown::Write).unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        response
    }

    fn start_server() -> (SocketAddr, Sender<()>, JoinHandle<io::Result<()>>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let (shutdown_tx, shutdown_rx) = mpsc::channel();
        let server =
            thread::spawn(move || serve(listener, ThreadPool::new(2).unwrap(), shutdown_rx));
        (address, shutdown_tx, server)
    }

    #[test]
    fn parser_extracts_request_line_and_normalizes_headers() {
        let parsed = parse_request(valid_request()).unwrap();
        assert_eq!(parsed.method, "GET");
        assert_eq!(parsed.target, "/");
        assert_eq!(parsed.version, "HTTP/1.1");
        assert_eq!(parsed.headers[0], ("host".into(), "localhost".into()));
    }

    struct OneByteReader<'a> {
        remaining: &'a [u8],
    }

    impl<'a> Read for OneByteReader<'a> {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            if output.is_empty() || self.remaining.is_empty() {
                return Ok(0);
            }
            output[0] = self.remaining[0];
            self.remaining = &self.remaining[1..];
            Ok(1)
        }
    }

    #[test]
    fn request_reader_handles_crlf_split_across_read_boundaries() {
        let source = OneByteReader {
            remaining: valid_request(),
        };
        let mut reader = BufReader::new(source);
        let parsed = read_request(&mut reader).unwrap();
        assert_eq!(parsed.method, "GET");
        assert_eq!(parsed.target, "/");
        assert_eq!(parsed.headers[0], ("host".into(), "localhost".into()));
    }

    #[test]
    fn parser_rejects_malformed_request_line() {
        assert_eq!(
            parse_request(b"GET / HTTP/1.1 EXTRA\r\n\r\n"),
            Err(RequestError::InvalidRequestLine)
        );
    }

    #[test]
    fn parser_rejects_malformed_header() {
        assert_eq!(
            parse_request(b"GET / HTTP/1.1\r\nnot-a-header\r\n\r\n"),
            Err(RequestError::InvalidHeader)
        );
    }

    #[test]
    fn parser_rejects_non_utf8_bytes() {
        assert_eq!(
            parse_request(b"GET / HTTP/1.1\r\nX-Test: \xff\r\n\r\n"),
            Err(RequestError::InvalidEncoding)
        );
    }

    #[test]
    fn parser_rejects_unsupported_protocol_version() {
        assert_eq!(
            parse_request(b"GET / HTTP/1.0\r\n\r\n"),
            Err(RequestError::UnsupportedVersion)
        );
    }

    #[test]
    fn parser_rejects_non_empty_request_bodies() {
        assert_eq!(
            parse_request(b"POST / HTTP/1.1\r\nContent-Length: 4\r\n\r\ndata"),
            Err(RequestError::UnsupportedBody)
        );
    }

    #[test]
    fn parser_rejects_oversized_headers() {
        let oversized = vec![b'a'; MAX_HEADER_BYTES + 1];
        assert_eq!(parse_request(&oversized), Err(RequestError::TooLarge));
    }

    #[test]
    fn parser_rejects_too_many_headers() {
        let mut request = String::from("GET / HTTP/1.1\r\n");
        for index in 0..=MAX_HEADERS {
            request.push_str(&format!("X-{index}: value\r\n"));
        }
        request.push_str("\r\n");
        assert_eq!(
            parse_request(request.as_bytes()),
            Err(RequestError::TooLarge)
        );
    }

    #[test]
    fn root_route_returns_the_sample_page() {
        let response = route(&parse_request(valid_request()).unwrap());
        assert_eq!(response.status, 200);
        assert!(String::from_utf8(response.body)
            .unwrap()
            .contains("Hello from Rust"));
    }

    #[test]
    fn unknown_route_returns_not_found() {
        let request = parse_request(b"GET /missing HTTP/1.1\r\n\r\n").unwrap();
        assert_eq!(route(&request).status, 404);
    }

    #[test]
    fn non_get_route_returns_method_not_allowed() {
        let request = parse_request(b"POST / HTTP/1.1\r\nContent-Length: 0\r\n\r\n").unwrap();
        let response = route(&request);
        assert_eq!(response.status, 405);
        assert_eq!(response.extra_headers, vec![("Allow", "GET")]);
    }

    #[test]
    fn response_content_length_counts_body_bytes() {
        let response = Response::new(200, "OK", "text/plain", "卫星".as_bytes().to_vec());
        let bytes = String::from_utf8(response.to_bytes()).unwrap();
        assert!(bytes.contains(&format!("Content-Length: {}\r\n", "卫星".len())));
    }

    #[test]
    fn zero_worker_pool_is_rejected() {
        assert!(matches!(ThreadPool::new(0), Err(PoolError::ZeroWorkers)));
    }

    #[test]
    fn dropping_pool_drains_all_queued_jobs_and_joins_workers() {
        let completed = Arc::new(AtomicUsize::new(0));
        {
            let pool = ThreadPool::new(3).unwrap();
            for _ in 0..12 {
                let completed = Arc::clone(&completed);
                pool.execute(move || {
                    completed.fetch_add(1, Ordering::SeqCst);
                })
                .unwrap();
            }
        }
        assert_eq!(completed.load(Ordering::SeqCst), 12);
    }

    #[test]
    fn fixed_pool_runs_jobs_on_multiple_workers() {
        let pool = ThreadPool::new(2).unwrap();
        let gate = Arc::new((Mutex::new(false), Condvar::new()));
        let active = Arc::new(AtomicUsize::new(0));
        let peak = Arc::new(AtomicUsize::new(0));
        let (started_tx, started_rx) = mpsc::channel();

        for _ in 0..2 {
            let gate = Arc::clone(&gate);
            let active = Arc::clone(&active);
            let peak = Arc::clone(&peak);
            let started_tx = started_tx.clone();
            pool.execute(move || {
                let current = active.fetch_add(1, Ordering::SeqCst) + 1;
                peak.fetch_max(current, Ordering::SeqCst);
                started_tx.send(()).unwrap();

                let (lock, condition) = &*gate;
                let released = lock.lock().unwrap();
                let _released = condition
                    .wait_timeout_while(released, Duration::from_secs(2), |ready| !*ready)
                    .unwrap()
                    .0;
                active.fetch_sub(1, Ordering::SeqCst);
            })
            .unwrap();
        }
        drop(started_tx);

        let first_started = started_rx.recv_timeout(Duration::from_secs(2)).is_ok();
        let second_started_before_release = started_rx.recv_timeout(Duration::from_secs(1)).is_ok();

        let (lock, condition) = &*gate;
        *lock.lock().unwrap() = true;
        condition.notify_all();
        drop(pool);

        assert!(first_started, "at least one worker must start a job");
        assert!(
            second_started_before_release,
            "the second worker must start while the first job is held"
        );
        assert_eq!(peak.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn real_tcp_server_returns_success_and_not_found_responses() {
        let (address, shutdown, server) = start_server();
        let success = request(address, b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n");
        assert!(success.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(success.contains("Hello from Rust"));

        let missing = request(address, b"GET /missing HTTP/1.1\r\nHost: localhost\r\n\r\n");
        assert!(missing.starts_with("HTTP/1.1 404 Not Found\r\n"));

        shutdown.send(()).unwrap();
        server.join().unwrap().unwrap();
    }

    #[test]
    fn real_tcp_server_returns_431_for_oversized_request_headers() {
        let (address, shutdown, server) = start_server();
        let mut oversized = b"GET / HTTP/1.1\r\nX-Test: ".to_vec();
        oversized.extend(std::iter::repeat(b'a').take(MAX_LINE_BYTES));
        oversized.extend_from_slice(b"\r\n\r\n");

        let response = request(address, &oversized);
        assert!(response.starts_with("HTTP/1.1 431 Request Header Fields Too Large\r\n"));

        shutdown.send(()).unwrap();
        server.join().unwrap().unwrap();
    }

    #[test]
    fn real_tcp_server_returns_bad_request_for_malformed_input() {
        let (address, shutdown, server) = start_server();
        let response = request(address, b"not a request\r\n\r\n");
        assert!(response.starts_with("HTTP/1.1 400 Bad Request\r\n"));
        shutdown.send(()).unwrap();
        server.join().unwrap().unwrap();
    }

    #[test]
    fn sequential_baseline_handles_its_connection_budget() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || serve_sequential(listener, 1));
        let response = request(address, b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n");
        assert!(response.starts_with("HTTP/1.1 200 OK\r\n"));
        server.join().unwrap().unwrap();
    }
}
