//! D9-B: practical modern standard-library features; no registry dependencies.
use std::ffi::{CString, OsStr};
use std::io::{self, BufRead};
use std::path::Path;
use std::sync::{LazyLock, RwLock, RwLockWriteGuard};
use std::time::Duration;

pub fn inspect_readings(readings: &[&str]) -> (Vec<u32>, usize, usize) {
    let (mut good, mut bad) = (0, 0);
    let values = readings
        .iter()
        .filter_map(|text| {
            text.parse::<u32>()
                .inspect(|_| good += 1)
                .inspect_err(|_| bad += 1)
                .ok()
        })
        .collect();
    (values, good, bad)
}

pub fn calibrate<const N: usize>(mut values: [u32; N], offsets: [u32; N]) -> [u32; N] {
    for (value, offset) in values.each_mut().into_iter().zip(offsets) {
        *value = value.saturating_add(offset);
    }
    values
}

pub fn empty_slots<const N: usize>() -> [Option<String>; N] {
    [const { None }; N]
}

pub fn lookup<F: FnOnce() -> Vec<i32>>(cache: &LazyLock<Vec<i32>, F>, indices: &[usize]) -> Vec<Option<i32>> {
    indices
        .iter()
        .map(|&index| LazyLock::force(cache).get(index).copied())
        .collect()
}

pub fn timing_error(actual: Duration, expected: Duration) -> Duration {
    actual.abs_diff(expected)
}

pub fn after_prefix<R: BufRead>(mut input: R) -> io::Result<String> {
    input.skip_until(b':')?;
    let mut remaining = String::new();
    input.read_to_string(&mut remaining)?;
    Ok(remaining)
}

pub fn c_label(input: &str) -> Option<String> {
    let encoded: CString = input.parse().ok()?;
    String::try_from(encoded).ok()
}

pub fn extract_outliers(values: &mut Vec<i32>, limit: i32) -> Vec<i32> {
    values.extract_if(.., |value| *value > limit).collect()
}

pub fn corrected(value: u32, offset: i32) -> Option<u32> {
    value.checked_sub_signed(offset)
}

pub fn file_label(path: &Path) -> Option<&OsStr> {
    path.file_prefix()
}

pub fn update_and_read<R>(lock: &RwLock<Vec<u32>>, value: u32, observe: impl FnOnce(&[u32]) -> R) -> R {
    let mut writer = lock.write().expect("teaching lock not poisoned");
    writer.push(value);
    let reader = RwLockWriteGuard::downgrade(writer);
    observe(&reader)
}

pub fn fixed_pair(values: &[i32]) -> Option<&[i32; 2]> {
    values.as_array::<2>()
}

pub fn transitions(values: &[i32]) -> Vec<i64> {
    values
        .array_windows::<2>()
        .map(|[first, second]| i64::from(*second) - i64::from(*first))
        .collect()
}

pub fn mask_info(mask: u32) -> (u32, u32) {
    (mask.bit_width(), mask.isolate_lowest_one())
}

pub fn token_spans(text: &str) -> Vec<(usize, usize)> {
    text.split_whitespace()
        .map(|token| {
            let range = text.substr_range(token).expect("token comes from text");
            (range.start, range.end)
        })
        .collect()
}

pub fn consume_boxed<const N: usize>(values: Box<[String; N]>) -> Vec<String> {
    values.into_iter().collect()
}

pub fn update_two(values: &mut [i32], first: usize, second: usize) -> bool {
    let Ok([a, b]) = values.get_disjoint_mut([first, second]) else {
        return false;
    };
    *a = a.saturating_add(1);
    *b = b.saturating_add(1);
    true
}
