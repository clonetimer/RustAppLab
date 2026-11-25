//! D9: process local sample batches without external dependencies.
use std::alloc::Allocator;
use std::num::ParseIntError;
use std::ops::AsyncFnMut;

pub trait LendingReader {
    type Window<'a>
    where
        Self: 'a;

    fn next(&mut self) -> Option<Self::Window<'_>>;
}

pub struct WindowReader {
    values: Vec<i32>,
    width: usize,
    offset: usize,
}

impl WindowReader {
    pub fn new(values: Vec<i32>, width: usize) -> Self {
        Self {
            values,
            width,
            offset: 0,
        }
    }
}

pub trait Named {
    fn name(&self) -> &str;
}

pub trait Sensor: Named {
    fn value(&self) -> i32;
}

// This local teaching trait intentionally does not promise that its returned
// future is Send. D10 will cover the runtime/thread boundary.
#[allow(async_fn_in_trait)]
pub trait SampleSource {
    async fn sample(&self, key: &str) -> Option<i32>;
}

impl LendingReader for WindowReader {
    type Window<'a> = &'a [i32];

    fn next(&mut self) -> Option<Self::Window<'_>> {
        if self.width == 0 {
            return None;
        }
        let end = self.offset.checked_add(self.width)?;
        let window = self.values.get(self.offset..end)?;
        self.offset += 1;
        Some(window)
    }
}

pub fn owned_minimum(values: &[i32]) -> impl Iterator<Item = i32> + use<> {
    values.iter().copied().min().into_iter()
}

pub fn named(sensor: &dyn Sensor) -> &dyn Named {
    sensor
}

pub async fn collect<S: SampleSource>(source: &S, keys: &[&str]) -> Vec<i32> {
    let mut values = Vec::new();
    for key in keys {
        if let Some(value) = source.sample(key).await {
            values.push(value);
        }
    }
    values
}

pub async fn visit<F>(values: &[i32], mut callback: F)
where
    F: for<'a> AsyncFnMut(&'a i32),
{
    for value in values {
        callback(value).await;
    }
}

pub fn positive(input: Option<&str>) -> Option<u32> {
    match input {
        Some(s) if let Ok(value) = s.parse::<u32>() => (value > 0).then_some(value),
        _ => None,
    }
}

pub fn infallible<T>(result: Result<T, !>) -> T {
    let Ok(value) = result;
    value
}

pub fn append_batch(out: &mut Vec<u32>, readings: &[&str]) -> Result<(), ParseIntError> {
    let before = out.len();
    let mut guarded = std::mem::DropGuard::new(out, move |buffer| {
        buffer.truncate(before);
    });
    for reading in readings {
        guarded.push(reading.parse()?);
    }
    std::mem::DropGuard::dismiss(guarded);
    Ok(())
}

pub fn copy_into<A: Allocator>(values: &[u32], allocator: A) -> Vec<u32, A> {
    let mut result = Vec::new_in(allocator);
    result.extend_from_slice(values);
    result
}
