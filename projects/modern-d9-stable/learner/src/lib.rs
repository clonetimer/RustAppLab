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
        // TODO(1): lend a window of the requested width, then advance one slot.
        let _ = (&self.values, self.width, self.offset);
        None
    }
}

pub fn owned_minimum(_values: &[i32]) -> impl Iterator<Item = i32> + use<> {
    // TODO(2): return an owned minimum without retaining the input borrow.
    None::<i32>.into_iter()
}

pub fn named(_sensor: &dyn Sensor) -> &dyn Named {
    // TODO(3): upcast without allocating or constructing a new object.
    todo!()
}

pub async fn collect<S: SampleSource>(_source: &S, _keys: &[&str]) -> Vec<i32> {
    // TODO(4): await each source lookup; skip missing keys and preserve order.
    Vec::new()
}

pub async fn visit<F>(_values: &[i32], _callback: F)
where
    F: for<'a> AsyncFnMut(&'a i32),
{
    // TODO(5): call and await the lending async closure for every value.
}

pub fn positive(_input: Option<&str>) -> Option<u32> {
    // TODO(6): use a match if-let guard; accept only a positive parsed value.
    None
}

pub fn infallible<T>(_result: Result<T, !>) -> T {
    // TODO(7): use the uninhabited error type, not unwrap or panic.
    todo!()
}

pub fn append_batch(_out: &mut Vec<u32>, _readings: &[&str]) -> Result<(), ParseIntError> {
    // TODO(8): append all readings, or roll back on the first parse failure.
    Ok(())
}

pub fn copy_into<A: Allocator>(_values: &[u32], _allocator: A) -> Vec<u32, A> {
    // TODO(9): preserve the caller-selected allocator and copy all values.
    todo!()
}
