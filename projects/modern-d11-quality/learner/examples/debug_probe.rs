//! Debugger target: inspect the two actual function arguments before comparison.
use std::hint::black_box;

#[inline(never)]
fn diagnose_frame(expected: u32, actual: u32) -> bool {
    black_box(expected) == black_box(actual)
}

fn main() {
    let valid = diagnose_frame(42, 43);
    println!("checksum_matches={valid}");
}
