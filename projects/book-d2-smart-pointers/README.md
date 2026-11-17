# Book D2 · Smart Pointers

Rust Book chapter 15, implemented as an isolated, dependency-free Workspace.

See [the Chinese lesson](../../course/BOOK_D2_SMART_POINTERS.md). This workspace is not a member of the frozen 17-package root workspace. Edition 2021, Rust 1.74, no registry dependencies, `publish = false`.

From the RustAppLab source root:

```powershell
cargo test --locked --offline --workspace --all-targets --manifest-path projects/book-d2-smart-pointers/Cargo.toml
cargo test --locked --offline --workspace --doc --manifest-path projects/book-d2-smart-pointers/Cargo.toml
cargo run --locked --offline --manifest-path projects/book-d2-smart-pointers/smart-pointer-lab/Cargo.toml -- box
cargo run --locked --offline --manifest-path projects/book-d2-smart-pointers/smart-pointer-lab/Cargo.toml -- weak
```

Try each CLI topic before reading the implementation. Then change a test in a personal copy, predict the ownership/borrow-count change, and run the tests again. This reference workspace demonstrates behavior; it is not a score for learner mastery.
