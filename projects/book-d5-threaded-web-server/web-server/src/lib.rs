//! A loopback-only teaching server for Rust Book chapter 20.
//!
//! A small executable example:
//!
//! ~~~
//! use rustapplab_book_d5_web_server::ThreadPool;
//!
//! let pool = ThreadPool::new(2).unwrap();
//! pool.execute(|| assert_eq!(2 + 2, 4)).unwrap();
//! drop(pool); // drains queued work and joins both workers
//! ~~~
//!
//! Jobs must be transferable to worker threads. An Rc-capturing closure is
//! rejected at compile time:
//!
//! ~~~compile_fail
//! use std::rc::Rc;
//! use rustapplab_book_d5_web_server::ThreadPool;
//!
//! let pool = ThreadPool::new(1).unwrap();
//! let value = Rc::new(7);
//! pool.execute(move || println!("{value}")).unwrap();
//! ~~~

mod thread_pool;

pub use thread_pool::ThreadPool;

use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

const MAX_REQUEST_LINE_BYTES: usize = 1024;
const READ_TIMEOUT: Duration = Duration::from_secs(2);
const SLEEP_ROUTE_DELAY: Duration = Duration::from_millis(150);

/// Serves exactly max_requests TCP connections on a single thread.
pub fn serve_single(listener: TcpListener, max_requests: usize) -> io::Result<()> {
    validate_request_count(max_requests)?;

    for incoming in listener.incoming().take(max_requests) {
        handle_connection(incoming?)?;
    }
    Ok(())
}

/// Serves exactly max_requests connections through a fixed-size worker pool.
///
/// When the accept loop ends, dropping the pool queues termination messages
/// and joins every worker before this function returns.
pub fn serve_thread_pool(
    listener: TcpListener,
    worker_count: usize,
    max_requests: usize,
) -> io::Result<()> {
    validate_request_count(max_requests)?;
    let pool = ThreadPool::new(worker_count)?;

    for incoming in listener.incoming().take(max_requests) {
        let stream = incoming?;
        pool.execute(move || {
            if let Err(error) = handle_connection(stream) {
                eprintln!("request handling failed: {error}");
            }
        })?;
    }

    drop(pool);
    Ok(())
}

fn validate_request_count(max_requests: usize) -> io::Result<()> {
    if max_requests == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "request count must be greater than zero for a graceful finite run",
        ));
    }
    Ok(())
}

fn handle_connection(stream: TcpStream) -> io::Result<()> {
    stream.set_read_timeout(Some(READ_TIMEOUT))?;
    let mut reader = BufReader::new(stream);
    let request_line = match read_request_line(&mut reader) {
        Ok(Some(line)) => line,
        Ok(None) => return Ok(()),
        Err(error) if error.kind() == io::ErrorKind::InvalidData => {
            return write_response(reader.get_mut(), "400 Bad Request", "Bad Request\n");
        }
        Err(error) => return Err(error),
    };

    let mut parts = request_line.split_whitespace();
    let method = parts.next();
    let path = parts.next();
    let version = parts.next();
    if method.is_none()
        || path.is_none()
        || version.is_none()
        || parts.next().is_some()
        || !matches!(version, Some("HTTP/1.0" | "HTTP/1.1"))
    {
        return write_response(reader.get_mut(), "400 Bad Request", "Bad Request\n");
    }

    if method != Some("GET") {
        return write_response(
            reader.get_mut(),
            "405 Method Not Allowed",
            "Method Not Allowed\n",
        );
    }

    let (status, body) = match path.unwrap() {
        "/" => ("200 OK", "Hello from the Rust Book server.\n"),
        "/sleep" => {
            thread::sleep(SLEEP_ROUTE_DELAY);
            ("200 OK", "Finished sleeping.\n")
        }
        _ => ("404 Not Found", "Not Found\n"),
    };

    write_response(reader.get_mut(), status, body)
}

fn read_request_line(reader: &mut BufReader<TcpStream>) -> io::Result<Option<String>> {
    let mut bytes = Vec::with_capacity(64);

    loop {
        let (chunk, found_newline) = {
            let available = reader.fill_buf()?;
            if available.is_empty() {
                if bytes.is_empty() {
                    return Ok(None);
                }
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "request line ended before newline",
                ));
            }

            let newline = available.iter().position(|byte| *byte == b'\n');
            let length = newline.map_or(available.len(), |index| index + 1);
            if bytes.len() + length > MAX_REQUEST_LINE_BYTES {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "request line exceeds the configured limit",
                ));
            }
            (available[..length].to_vec(), newline.is_some())
        };

        let consumed = chunk.len();
        bytes.extend_from_slice(&chunk);
        reader.consume(consumed);
        if found_newline {
            break;
        }
    }

    bytes.pop();
    if bytes.last() == Some(&b'\r') {
        bytes.pop();
    }
    String::from_utf8(bytes)
        .map(Some)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "request line is not UTF-8"))
}

fn write_response(stream: &mut TcpStream, status: &str, body: &str) -> io::Result<()> {
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    )?;
    stream.flush()
}
