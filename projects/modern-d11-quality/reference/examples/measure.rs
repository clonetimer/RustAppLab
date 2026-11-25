//! A small measurement experiment, not a statistical performance verdict.
use d11_quality_lab::{count_iter, count_loop};
use std::hint::black_box;
use std::time::Instant;

fn median_ns(f: fn(&[u8], u8) -> usize, input: &[u8], rounds: usize) -> (u128, usize) {
    let mut measurements = Vec::with_capacity(5);
    let mut checked = 0usize;
    for _ in 0..5 {
        let start = Instant::now();
        let mut sum = 0usize;
        for _ in 0..rounds {
            sum += black_box(f(black_box(input), black_box(127)));
        }
        measurements.push(start.elapsed().as_nanos());
        checked = sum;
    }
    measurements.sort_unstable();
    (measurements[2], checked)
}

fn main() {
    let rounds: usize = std::env::args().nth(1).map_or(1000, |n| n.parse().expect("positive rounds"));
    assert!((1..=10_000).contains(&rounds));
    let values: Vec<u8> = (0..4096).map(|i| (i % 256) as u8).collect();
    let expected = 16 * 129 * rounds;
    // Warm up both algorithms before measuring. Do not assert a speed ratio.
    assert_eq!(black_box(count_loop(&values, 127)), 16 * 129);
    assert_eq!(black_box(count_iter(&values, 127)), 16 * 129);
    let (loop_ns, loop_count) = median_ns(count_loop, &values, rounds);
    let (iter_ns, iter_count) = median_ns(count_iter, &values, rounds);
    assert_eq!(loop_count, expected);
    assert_eq!(iter_count, expected);
    println!("size={}; rounds={rounds}; samples=5; result={expected}", values.len());
    println!("loop_median_ns={loop_ns}");
    println!("iterator_median_ns={iter_ns}");
}
