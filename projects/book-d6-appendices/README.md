# Book D6 · Appendix Reference Lab

A dependency-free, isolated Rust 2021 workspace for the Rust Book 2018 Appendix A–G reference material.

- Minimum Rust: 1.74
- No crates.io dependencies; all commands run offline
- Not part of the root workspace or the 17 learner projects
- The code lab exercises raw identifiers, operator behavior, operator overloading, derivable traits, and the distinction between `Debug` and `Display`
- Appendix D/E/F/G are taught through reproducible toolchain and edition workflows in `course/BOOK_D6_APPENDICES.md`

From the RustAppLab source root:

```powershell
cargo fmt --manifest-path projects/book-d6-appendices/Cargo.toml --all -- --check
cargo clippy --locked --offline --manifest-path projects/book-d6-appendices/Cargo.toml --workspace --all-targets -- -D warnings
cargo test --locked --offline --manifest-path projects/book-d6-appendices/Cargo.toml --workspace --all-targets
cargo test --locked --offline --manifest-path projects/book-d6-appendices/Cargo.toml --workspace --doc
cargo test --locked --offline --manifest-path projects/book-d6-appendices/Cargo.toml --workspace --all-targets --release
```

The lab is a compact executable companion, not a replacement for the reference tables in the lesson. Do not treat a passing test as proof that a learner can explain every keyword or symbol.
