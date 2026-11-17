# Book D5 · Multithreaded Web Server

A dependency-free reference workspace for Rust Book (Chinese Edition 2018) chapter 20. It builds a small HTTP/1.1 server directly on the Rust standard library, then adds a fixed-size worker pool and an orderly shutdown path.

This is a teaching implementation, not a production web server. It deliberately supports a narrow subset: HTTP/1.1, GET requests, no request body, no keep-alive, no TLS, and bounded headers. It does not replace Axum/Tokio or the existing ServiceDesk projects.

- Edition 2021; minimum Rust 1.74
- No crates.io dependencies; commands run offline
- publish = false; isolated from the root workspace and frozen RC scope

From the RustAppLab source root:

~~~powershell
cargo fmt --manifest-path projects/book-d5-webserver/Cargo.toml --all -- --check
cargo clippy --locked --offline --manifest-path projects/book-d5-webserver/Cargo.toml --workspace --all-targets -- -D warnings
cargo test --locked --offline --manifest-path projects/book-d5-webserver/Cargo.toml --workspace --all-targets
cargo test --locked --offline --manifest-path projects/book-d5-webserver/Cargo.toml --workspace --doc
cargo run --locked --offline --manifest-path projects/book-d5-webserver/webserver-lab/Cargo.toml -- 127.0.0.1:7878 4
~~~

Open http://127.0.0.1:7878/, /sleep, or /missing. In the server terminal, type stop and press Enter. The listener stops accepting requests, queued jobs finish, and worker threads are joined before the process exits.

The library exposes both serve_sequential (finite single-threaded baseline) and serve (fixed worker pool plus shutdown channel), so the concurrency change can be compared without duplicating the HTTP parser.

Read the lesson at course/BOOK_D5_WEB_SERVER.md. Tests cover request parsing boundaries, response framing, real TCP exchanges, bounded worker concurrency, queue draining, and shutdown. Passing tests validate the reference implementation; learners should still complete the lesson exercises and record their own observations.
