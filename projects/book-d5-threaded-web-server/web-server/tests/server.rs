use rustapplab_book_d5_web_server::{serve_single, serve_thread_pool};
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread::{self, JoinHandle};

fn start_single(requests: usize) -> (SocketAddr, JoinHandle<io::Result<()>>) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let address = listener.local_addr().unwrap();
    let server = thread::spawn(move || serve_single(listener, requests));
    (address, server)
}

fn start_pool(workers: usize, requests: usize) -> (SocketAddr, JoinHandle<io::Result<()>>) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let address = listener.local_addr().unwrap();
    let server = thread::spawn(move || serve_thread_pool(listener, workers, requests));
    (address, server)
}

fn request(address: SocketAddr, line: &str) -> String {
    let mut stream = TcpStream::connect(address).unwrap();
    write!(stream, "{line}\r\nHost: localhost\r\n\r\n").unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    response
}

#[test]
fn single_threaded_server_serves_a_real_http_request() {
    let (address, server) = start_single(1);
    let response = request(address, "GET / HTTP/1.1");
    server.join().unwrap().unwrap();
    assert!(response.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(response.contains("Hello from the Rust Book server."));
    assert!(response.contains("Connection: close\r\n"));
}

#[test]
fn thread_pool_server_handles_two_real_requests_and_shuts_down() {
    let (address, server) = start_pool(2, 2);
    let first = thread::spawn(move || request(address, "GET / HTTP/1.1"));
    let second = thread::spawn(move || request(address, "GET /sleep HTTP/1.1"));

    let first_response = first.join().unwrap();
    let second_response = second.join().unwrap();
    server.join().unwrap().unwrap();

    assert!(first_response.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(first_response.contains("Hello from the Rust Book server."));
    assert!(second_response.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(second_response.contains("Finished sleeping."));
}

#[test]
fn unknown_routes_return_not_found() {
    let (address, server) = start_single(1);
    let response = request(address, "GET /missing HTTP/1.1");
    server.join().unwrap().unwrap();
    assert!(response.starts_with("HTTP/1.1 404 Not Found\r\n"));
}

#[test]
fn unsupported_methods_return_method_not_allowed() {
    let (address, server) = start_single(1);
    let response = request(address, "POST / HTTP/1.1");
    server.join().unwrap().unwrap();
    assert!(response.starts_with("HTTP/1.1 405 Method Not Allowed\r\n"));
}

#[test]
fn malformed_request_lines_return_bad_request() {
    let (address, server) = start_single(1);
    let response = request(address, "BROKEN");
    server.join().unwrap().unwrap();
    assert!(response.starts_with("HTTP/1.1 400 Bad Request\r\n"));
}
