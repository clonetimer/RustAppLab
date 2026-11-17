# Book D3 · Object-Oriented Design and Pattern Matching

Rust Book chapters 17–18, implemented as an isolated, dependency-free Workspace.

Read [the Chinese lesson](../../course/BOOK_D3_OOP_PATTERNS.md). This is a reference lab, not a new learner project. Edition 2021, Rust 1.74, no registry dependencies, `publish = false`.

From the RustAppLab source root:

```powershell
cargo test --locked --offline --workspace --all-targets --manifest-path projects/book-d3-oop-patterns/Cargo.toml
cargo test --locked --offline --workspace --doc --manifest-path projects/book-d3-oop-patterns/Cargo.toml
cargo run --locked --offline --manifest-path projects/book-d3-oop-patterns/oop-pattern-lab/Cargo.toml -- patterns
cargo run --locked --offline --manifest-path projects/book-d3-oop-patterns/oop-pattern-lab/Cargo.toml -- oop
cargo run --locked --offline --manifest-path projects/book-d3-oop-patterns/oop-pattern-lab/Cargo.toml -- state
```

Try to predict each result before reading the implementation. In a personal copy, add a new pattern case and a new state transition test. Do not edit the frozen root Workspace or publish the reference crate.
