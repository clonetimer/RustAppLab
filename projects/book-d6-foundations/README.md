# Book D6-B · Foundations Gap Lab

A dependency-free reference workspace for the remaining practical gaps in Rust Book chapters 1–13.

- Edition 2021; minimum Rust 1.74
- No registry dependencies; offline builds
- Not part of the root workspace or the 17 learner products
- One small CLI and library connect the original concepts to executable tests
- Cargo 1.3 review: package / crate / bin / lib, manifests and lockfiles, check / build / run, debug / release
- Compound types from 3.2: `summarize_scores([u32; 3]) -> (u64, u32)`, mixed tuples, unit, arrays and bounds checks
- Unit, CLI and compound-type integration tests, plus runnable, `should_panic` and `compile_fail` doctests

Follow the [Chinese lesson and three-step Cargo exercise](../../course/BOOK_D6_FOUNDATIONS.md).
The [variables and modules lesson](../../course/BOOK_D6_LANGUAGE_BASICS.md) covers the separate 3.1 and 7.2–7.5 exercises.

Continue with [ownership, receivers and slices](../../course/BOOK_OWNERSHIP_PRACTICE.md),
[collections and error propagation](../../course/BOOK_COLLECTIONS_ERRORS.md), and
[generics and lifetimes](../../course/BOOK_GENERICS_LIFETIMES.md).
Each module has focused integration tests and paired runnable/compiler-error examples.
The [scoped concept audit](../../course/BOOK_CONCEPT_AUDIT.md) records remaining teaching targets separately from reference-code test results.

## Cargo vocabulary in this workspace

| Term | Concrete location or behavior |
|---|---|
| Workspace | This directory's `Cargo.toml` groups the `foundation-lab` member; it is separate from RustAppLab's root workspace. |
| Package | `foundation-lab/Cargo.toml` describes the package named `book-d6-foundation-lab`. Select it with `-p book-d6-foundation-lab`. |
| Library crate | `foundation-lab/src/lib.rs` exposes reusable functions; Rust code imports it as `book_d6_foundation_lab`. |
| Binary crate | `foundation-lab/src/main.rs` contains the CLI entry point. Its explicit target name is `foundation-lab`; select it with `--bin foundation-lab`. |
| `Cargo.toml` | The workspace manifest sets members and profiles; the package manifest sets the package, library and binary targets. |
| `Cargo.lock` | Cargo records the resolved package versions here, even in this dependency-free workspace. Keep it under version control; do not edit it by hand. |

From the RustAppLab source root, check types, build an executable, then run it:

```powershell
cargo check --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab
cargo build --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab
cargo run --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab --bin foundation-lab -- hello
cargo run --release --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab --bin foundation-lab -- hello
```

`check` does not produce a runnable executable; `build` does; `run` builds when needed and starts the binary. The `--` separates Cargo options from the CLI's `hello` argument. Without a target-directory override, the two executable paths are under this workspace's `target/debug/` and `target/release/` (with `.exe` on Windows). This workspace sets development optimization to `0` and release optimization to `2`; the two modes should print the same greeting.

## Compound-type practice

`summarize_scores` accepts exactly three `u32` scores and returns `(total, highest)` as `(u64, u32)`. It widens each score before addition, so three `u32::MAX` values still produce the exact total in debug and release builds. It does not validate a 0–100 grading scale.

The function's rustdoc examples show mixed-type tuples, destructuring, `.0` / `.1`, the unit value `()`, repeated arrays such as `[0_u32; 3]`, and the difference between `scores[index]` and `scores.get(index)`. Two `compile_fail` examples reject the wrong array length and a floating-point element in a `[u32; 3]` array.

```powershell
cargo test --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab --test compound_types
cargo test --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab --doc summarize_scores
```

## Full verification

From the RustAppLab source root:

```powershell
cargo fmt --manifest-path projects/book-d6-foundations/Cargo.toml --all -- --check
cargo clippy --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml --workspace --all-targets -- -D warnings
cargo test --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml --workspace --all-targets
cargo test --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml --workspace --doc
cargo test --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml --workspace --all-targets --release
```

Passing these checks is evidence about the reference implementation. Learner mastery still needs the exercises and explanations in the lesson.

Try the runnable exercises:

```powershell
cargo run --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab -- hello
cargo run --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab -- words "rust ownership rust"
cargo run --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab -- guess
cargo run --locked --offline --manifest-path projects/book-d6-foundations/Cargo.toml -p book-d6-foundation-lab -- bench 100000
```

The guessing-game target is generated by a small time-seeded xorshift routine solely to keep the lab dependency-free and reproducible in offline CI. It is not cryptographically secure and must not be used for security-sensitive randomness.
