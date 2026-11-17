# Book D4 · Advanced Features

Rust Book chapter 19 in the Chinese Rust 2018 edition (rust.bootcss.com), mapped by topic to chapter 20 of the current English Rust Book. Five units cover unsafe Rust, advanced traits, advanced types, advanced functions/closures, and macros.

Read the course lesson at course/BOOK_D4_ADVANCED_FEATURES.md. This is a reference lab, not a new learner project. Edition 2021, Rust 1.74, no registry dependencies, publish = false. The procedural-macro crate uses only the stable proc_macro API; it intentionally avoids syn/quote so the lab stays offline and exposes token mechanics.

From the RustAppLab source root:

~~~powershell
cargo fmt --manifest-path projects/book-d4-advanced-features/Cargo.toml --all -- --check
cargo clippy --locked --offline --manifest-path projects/book-d4-advanced-features/Cargo.toml --workspace --all-targets -- -D warnings
cargo test --locked --offline --manifest-path projects/book-d4-advanced-features/Cargo.toml --workspace --all-targets
cargo test --locked --offline --manifest-path projects/book-d4-advanced-features/Cargo.toml --workspace --doc
cargo run --locked --offline --manifest-path projects/book-d4-advanced-features/advanced-lab/Cargo.toml -- unsafe
cargo run --locked --offline --manifest-path projects/book-d4-advanced-features/advanced-lab/Cargo.toml -- traits
cargo run --locked --offline --manifest-path projects/book-d4-advanced-features/advanced-lab/Cargo.toml -- types
cargo run --locked --offline --manifest-path projects/book-d4-advanced-features/advanced-lab/Cargo.toml -- functions
cargo run --locked --offline --manifest-path projects/book-d4-advanced-features/advanced-lab/Cargo.toml -- macros
~~~

The unsafe examples use valid, bounded inputs only. The intentionally unsafe global-counter API is documented as single-thread-only and is never used by concurrent code. No example attempts to trigger undefined behavior.
