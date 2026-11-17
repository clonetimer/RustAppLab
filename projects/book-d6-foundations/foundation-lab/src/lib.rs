//! # D6-B Foundations Gap Lab
//!
//! Small examples for the remaining practical gaps in Rust Book chapters 1–13.
//!
//! A public function can be documented with a runnable example:
//!
//! ```
//! use book_d6_foundation_lab::Rectangle;
//!
//! let room = Rectangle::new(4, 5);
//! assert_eq!(room.area(), 20);
//! ```
//!
//! A fallible parser returns `Result`; callers must handle the error instead
//! of pretending the result is already a string:
//!
//! ```compile_fail
//! use book_d6_foundation_lab::parse_non_empty;
//!
//! let name: &str = parse_non_empty("Ada");
//! ```

use std::collections::HashMap;
use std::ffi::OsStr;
use std::fmt;
use std::io::{self, BufRead};
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// A rectangle used to practice struct fields, ownership, and method syntax.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rectangle {
    pub width: u32,
    pub height: u32,
}

impl Rectangle {
    /// Construct a rectangle from its width and height.
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Construct a square using the same size for both dimensions.
    pub const fn square(size: u32) -> Self {
        Self::new(size, size)
    }

    /// Return the area using a wider integer to avoid u32 multiplication overflow.
    pub fn area(self) -> u64 {
        u64::from(self.width) * u64::from(self.height)
    }

    /// Return whether this rectangle can fully contain another rectangle.
    pub fn can_hold(self, other: Self) -> bool {
        self.width >= other.width && self.height >= other.height
    }
}

/// The result of comparing a guess with the hidden number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuessOutcome {
    TooSmall,
    Correct,
    TooLarge,
}

/// Compare a guess with a secret value.
pub fn compare_guess(secret: u32, guess: u32) -> GuessOutcome {
    match guess.cmp(&secret) {
        std::cmp::Ordering::Less => GuessOutcome::TooSmall,
        std::cmp::Ordering::Equal => GuessOutcome::Correct,
        std::cmp::Ordering::Greater => GuessOutcome::TooLarge,
    }
}

/// Parse the numeric part of a guessing-game input.
pub fn parse_guess(input: &str) -> Result<u32, std::num::ParseIntError> {
    input.trim().parse()
}

/// Demonstrate `if let` when only one enum variant needs special handling.
pub fn describe_optional_name(name: Option<&str>) -> &'static str {
    if let Some(name) = name {
        if name.trim().is_empty() {
            "empty name"
        } else {
            "name provided"
        }
    } else {
        "name missing"
    }
}

/// Count whitespace-separated words with the HashMap entry API.
pub fn word_frequencies(input: &str) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for word in input.split_whitespace() {
        *counts.entry(word.to_lowercase()).or_insert(0) += 1;
    }
    counts
}

/// Error returned when a user-provided string is empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidationError;

impl fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("value must not be empty")
    }
}

impl std::error::Error for ValidationError {}

/// Validate external input without panicking.
pub fn parse_non_empty(input: &str) -> Result<&str, ValidationError> {
    if input.trim().is_empty() {
        Err(ValidationError)
    } else {
        Ok(input.trim())
    }
}

/// Assert a programmer invariant; do not use this for ordinary user input.
pub fn require_non_empty(input: &str) -> &str {
    assert!(
        !input.trim().is_empty(),
        "programmer invariant: empty value"
    );
    input
}

/// Derive an application config directory from an injected home path.
pub fn config_dir_from(home: Option<&OsStr>) -> Option<PathBuf> {
    home.map(|value| PathBuf::from(value).join(".rustapplab"))
}

/// Count non-empty lines using buffered I/O and iterator error propagation.
pub fn count_non_empty_lines<R: BufRead>(reader: R) -> io::Result<usize> {
    reader.lines().try_fold(0usize, |count, line| {
        let line = line?;
        Ok(count + usize::from(!line.trim().is_empty()))
    })
}

/// Sum values with an explicit loop.
pub fn sum_with_loop(values: &[u64]) -> u64 {
    let mut total = 0;
    for value in values {
        total += *value;
    }
    total
}

/// Sum values with iterator adapters.
pub fn sum_with_iterator(values: &[u64]) -> u64 {
    values.iter().copied().sum()
}

/// Timing results for an illustrative loop-versus-iterator comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BenchmarkResult {
    pub loop_total: u64,
    pub iterator_total: u64,
    pub loop_elapsed: Duration,
    pub iterator_elapsed: Duration,
}

/// Compare equivalent implementations without asserting which one is faster.
pub fn benchmark_sums(values: &[u64], repeats: usize) -> BenchmarkResult {
    let repeats = repeats.max(1);

    let start = Instant::now();
    let mut loop_total = 0;
    for _ in 0..repeats {
        loop_total = std::hint::black_box(sum_with_loop(std::hint::black_box(values)));
    }
    let loop_elapsed = start.elapsed();

    let start = Instant::now();
    let mut iterator_total = 0;
    for _ in 0..repeats {
        iterator_total = std::hint::black_box(sum_with_iterator(std::hint::black_box(values)));
    }
    let iterator_elapsed = start.elapsed();

    BenchmarkResult {
        loop_total,
        iterator_total,
        loop_elapsed,
        iterator_elapsed,
    }
}

/// Produce a small deterministic pseudo-random number in 1..=100 from a seed.
///
/// This xorshift helper is for the dependency-free learning game only. It is
/// not a cryptographically secure random-number generator.
pub fn secret_from_seed(seed: u64) -> u32 {
    let mut value = seed;
    value ^= value << 13;
    value ^= value >> 7;
    value ^= value << 17;
    (value % 100 + 1) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    #[test]
    fn rectangle_methods_calculate_area_and_containment() {
        let outer = Rectangle::new(8, 7);
        let inner = Rectangle::new(5, 6);
        assert_eq!(outer.area(), 56);
        assert!(outer.can_hold(inner));
        assert!(!inner.can_hold(outer));
        assert_eq!(Rectangle::square(4), Rectangle::new(4, 4));
    }

    #[test]
    fn guess_comparison_covers_all_three_outcomes() {
        assert_eq!(compare_guess(42, 10), GuessOutcome::TooSmall);
        assert_eq!(compare_guess(42, 42), GuessOutcome::Correct);
        assert_eq!(compare_guess(42, 90), GuessOutcome::TooLarge);
    }

    #[test]
    fn guess_parser_handles_valid_and_invalid_input() {
        assert_eq!(parse_guess(" 42 ").unwrap(), 42);
        assert!(parse_guess("forty-two").is_err());
    }

    #[test]
    fn if_let_example_distinguishes_option_cases() {
        assert_eq!(describe_optional_name(None), "name missing");
        assert_eq!(describe_optional_name(Some("")), "empty name");
        assert_eq!(describe_optional_name(Some("Ada")), "name provided");
    }

    #[test]
    fn hashmap_entry_counts_case_insensitive_words() {
        let counts = word_frequencies("Rust rust Ownership ownership");
        assert_eq!(counts.get("rust"), Some(&2));
        assert_eq!(counts.get("ownership"), Some(&2));
    }

    #[test]
    fn result_validation_returns_errors_for_external_input() {
        assert_eq!(parse_non_empty("  hello ").unwrap(), "hello");
        assert_eq!(parse_non_empty("   "), Err(ValidationError));
    }

    #[test]
    fn panic_is_reserved_for_broken_programmer_invariants() {
        assert_eq!(require_non_empty("valid"), "valid");
        let result = catch_unwind(AssertUnwindSafe(|| require_non_empty(" ")));
        assert!(result.is_err());
    }

    #[test]
    fn config_path_is_testable_without_mutating_process_environment() {
        let path = config_dir_from(Some(OsStr::new("home/learner"))).unwrap();
        assert_eq!(path, PathBuf::from("home/learner").join(".rustapplab"));
        assert!(config_dir_from(None).is_none());
    }

    #[test]
    fn buffered_line_count_propagates_io_and_ignores_blank_lines() {
        let input = Cursor::new(b"one\n\n two \nthree\n");
        assert_eq!(count_non_empty_lines(input).unwrap(), 3);
    }

    struct FailingReader;

    impl std::io::Read for FailingReader {
        fn read(&mut self, _buffer: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "synthetic read failure",
            ))
        }
    }

    impl BufRead for FailingReader {
        fn fill_buf(&mut self) -> io::Result<&[u8]> {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "synthetic read failure",
            ))
        }

        fn consume(&mut self, _amount: usize) {}
    }

    #[test]
    fn buffered_line_count_propagates_io_failures() {
        let error = count_non_empty_lines(FailingReader).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert_eq!(error.to_string(), "synthetic read failure");
    }

    #[test]
    fn loop_and_iterator_sums_produce_identical_results() {
        let values = [1, 2, 3, 4, 5];
        assert_eq!(sum_with_loop(&values), 15);
        assert_eq!(sum_with_iterator(&values), 15);
    }

    #[test]
    fn benchmark_compares_equal_work_without_assuming_a_winner() {
        let values: Vec<u64> = (0..100).collect();
        let result = benchmark_sums(&values, 3);
        assert_eq!(result.loop_total, result.iterator_total);
        assert_eq!(result.loop_total, 4950);
    }

    #[test]
    fn seeded_guess_is_repeatable_and_in_range() {
        let first = secret_from_seed(12345);
        assert_eq!(first, secret_from_seed(12345));
        assert!((1..=100).contains(&first));
    }

    #[test]
    fn zero_seed_still_produces_a_valid_secret() {
        assert_eq!(secret_from_seed(0), 1);
    }
}
