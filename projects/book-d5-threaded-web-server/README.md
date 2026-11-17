# Book D5 · Multithreaded Web Server

A dependency-free reference lab for the Chinese Rust Book 2018 chapter 20 (the current English Book's chapter 21). It implements the three original sections in sequence: a single-threaded TCP/HTTP server, a fixed-size worker pool, and graceful pool shutdown.

Read the course lesson at course/BOOK_D5_THREADPOOL_WEB_SERVER.md. Edition 2021, Rust 1.74, no registry dependencies, publish = false. The workspace is deliberately separate from the frozen 17-package root workspace.

From the RustAppLab source root:

~~~powershell
cargo fmt --manifest-path projects/book-d5-threaded-web-server/Cargo.toml --all -- --check
cargo clippy --locked --offline --manifest-path projects/book-d5-threaded-web-server/Cargo.toml --workspace --all-targets -- -D warnings
cargo test --locked --offline --manifest-path projects/book-d5-threaded-web-server/Cargo.toml --workspace --all-targets
cargo test --locked --offline --manifest-path projects/book-d5-threaded-web-server/Cargo.toml --workspace --doc
cargo run --locked --offline --manifest-path projects/book-d5-threaded-web-server/web-server/Cargo.toml -- single 7878 1
cargo run --locked --offline --manifest-path projects/book-d5-threaded-web-server/web-server/Cargo.toml -- pool 7878 4 2
~~~

The CLI binds only to 127.0.0.1. The final argument is a positive request count; after that many connections the server stops accepting, drops the pool, sends termination messages, and joins its workers. This finite run makes the shutdown sequence observable and testable. It is a teaching server, not a production HTTP implementation: one bounded request line, a small fixed route set, one response per connection, and no TLS, keep-alive, chunked bodies, or external exposure.
