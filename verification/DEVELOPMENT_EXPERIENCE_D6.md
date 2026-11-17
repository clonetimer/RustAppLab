# RustAppLab · D6 开发经验记录

Date: 2026-11-17  
Scope: D6-A appendix reference and D6-B foundations gap lab

## 1. Keep reference labs isolated from learner products

D6-A and D6-B are reference workspaces, not additional learner projects. They remain outside the root Cargo workspace and do not change the frozen 17-package learner set. Each lab declares Edition 2021, Rust 1.74 MSRV, `publish = false`, and no registry dependencies.

**Rule:** a coverage reference lab must be independently buildable and testable without silently changing the learner product inventory. Register every reference workspace in structural validation, release checks, declared-MSRV checks, source packaging, learner-kit packaging, POSIX/PowerShell verification, and regression tests.

## 2. Make interactive examples deterministic at the test boundary

The guessing game uses an injected comparison function and a small time-seeded xorshift helper only for the interactive demonstration. Unit tests exercise fixed secret/guess pairs; they never wait for wall-clock randomness or assert a particular random output.

The time-seeded helper is explicitly documented as non-cryptographic. Do not present a small PRNG as suitable for security-sensitive randomness. When a future lesson requires a production RNG, introduce the `rand` crate only with a pinned lockfile and a reproducible offline dependency strategy.

## 3. Do not mutate process environment in unit tests

The config-path lesson uses `config_dir_from(Option<&OsStr>)`. The CLI reads environment variables, but the core path decision is a pure function. This avoids tests racing over global process environment and makes Windows/POSIX path behavior easier to isolate.

**Rule:** inject environment-derived values into pure functions; reserve direct `std::env` access for the application boundary.

## 4. Demonstrate panic versus Result with an explicit decision boundary

Ordinary invalid input is represented as `Result`; panic is reserved for a broken programmer invariant. A panic test may use `catch_unwind` to observe behavior, but the lesson must not imply that catching arbitrary panics is a general recovery strategy.

## 5. Benchmark equivalent work and never assert a performance winner

Loop and iterator versions receive the same slice and repeat count, and the test checks equal results only. Timing is an opt-in release-mode observation using `Instant` and `black_box`; no CI test asserts that one form is faster.

**Rule:** CI validates semantic equivalence and that the benchmark runs. Performance claims require repeated measurements, release builds, controlled inputs, and a report of variability—not a single wall-clock assertion.

## 6. Preserve I/O errors while using iterator adapters

The buffered line-count example uses `BufRead::lines` with `try_fold`. Each line read can fail, and `?` propagates the error. Do not convert an I/O error into an empty line or silently skip it unless the application contract explicitly permits that behavior.

## 7. Keep Appendix keyword and symbol references edition-aware

D6-A now includes the complete current Strict/Reserved/Weak keyword sets and explains the Edition 2018/2024 changes. The older Chinese book is retained as the audit chapter baseline, while current token classification is linked to the Rust Reference.

**Rule:** never copy a historical keyword table forward without checking the current Rust Reference and Edition Guide. Distinguish current syntax from historical Edition behavior.
