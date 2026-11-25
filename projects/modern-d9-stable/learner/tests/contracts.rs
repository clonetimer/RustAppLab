use d9_stable_lab::*;
use std::alloc::{Global, System};
use std::future::{Future, ready};
use std::task::{Context, Poll, Waker};

// A single poll is valid only for these explicitly ready, local test futures.
// Never use this helper as a general-purpose async runtime.
fn ready_only<F: Future>(future: F) -> F::Output {
    let mut future = std::pin::pin!(future);
    let mut cx = Context::from_waker(Waker::noop());
    match future.as_mut().poll(&mut cx) {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("the test fixture must not wait for external I/O"),
    }
}

#[test]
fn step1_lends_overlapping_windows_and_stops() {
    let mut reader = WindowReader::new(vec![10, 20, 30, 40], 2);
    assert_eq!(reader.next(), Some(&[10, 20][..]));
    assert_eq!(reader.next(), Some(&[20, 30][..]));
    assert_eq!(reader.next(), Some(&[30, 40][..]));
    assert_eq!(reader.next(), None);
    assert_eq!(reader.next(), None);
    assert_eq!(WindowReader::new(vec![1], 0).next(), None);
    assert_eq!(WindowReader::new(vec![1], usize::MAX).next(), None);
    assert_eq!(WindowReader::new(vec![7], 1).next(), Some(&[7][..]));
}

#[test]
fn step2_minimum_does_not_capture_the_input_borrow() {
    let mut values = vec![8, -4, 3, -4];
    let result = owned_minimum(&values);
    values.clear();
    assert_eq!(result.collect::<Vec<_>>(), [-4]);
    assert_eq!(owned_minimum(&[]).next(), None);
    let result = {
        let temporary = vec![21, 10];
        owned_minimum(&temporary)
    };
    assert_eq!(result.collect::<Vec<_>>(), [10]);
}

struct Probe(&'static str, i32);
impl Named for Probe {
    fn name(&self) -> &str {
        self.0
    }
}
impl Sensor for Probe {
    fn value(&self) -> i32 {
        self.1
    }
}

#[test]
fn step3_upcasting_preserves_the_original_object() {
    let a = Probe("thermal", 42);
    let b = Probe("power", -5);
    for source in [&a as &dyn Sensor, &b as &dyn Sensor] {
        let name = named(source);
        assert_eq!(name.name(), source.name());
        assert!(std::ptr::eq(name, source as &dyn Named));
    }
}

struct Local;
impl SampleSource for Local {
    async fn sample(&self, key: &str) -> Option<i32> {
        ready(match key {
            "a" => Some(7),
            "b" => Some(-2),
            _ => None,
        })
        .await
    }
}

#[test]
fn step4_async_trait_collects_in_order() {
    assert_eq!(ready_only(collect(&Local, &["a", "missing", "b", "a"])), [7, -2, 7]);
    assert!(ready_only(collect(&Local, &[])).is_empty());
    assert!(ready_only(collect(&Local, &["unknown"])).is_empty());
}

#[test]
fn step5_async_closure_borrows_captured_state() {
    let mut output = Vec::new();
    ready_only(visit(&[3, -1, 9], async |value| {
        output.push(ready(*value * 2).await);
    }));
    assert_eq!(output, [6, -2, 18]);
    ready_only(visit(&[], async |value| {
        output.push(*value);
    }));
    assert_eq!(output, [6, -2, 18]);
}

#[test]
fn step6_match_guard_handles_parse_and_fallback() {
    for (input, expected) in [
        (Some("17"), Some(17)),
        (Some("1"), Some(1)),
        (Some("4294967295"), Some(u32::MAX)),
        (Some("0"), None),
        (Some("-1"), None),
        (Some("not-a-number"), None),
        (Some("4294967296"), None),
        (None, None),
    ] {
        assert_eq!(positive(input), expected);
    }
}

#[test]
fn step7_never_type_is_infallible_not_a_runtime_error() {
    assert_eq!(infallible(Ok::<_, !>(123)), 123);
    let value = String::from("owned sample");
    assert_eq!(infallible(Ok::<_, !>(value)), "owned sample");
    let converted: Result<u32, std::convert::Infallible> = Ok(5);
    assert_eq!(infallible(converted), 5);
}

#[test]
fn step8_drop_guard_preserves_prior_data_on_failure() {
    let mut buffer = vec![99, 0];
    assert!(append_batch(&mut buffer, &["2", "bad", "3"]).is_err());
    assert_eq!(buffer, [99, 0]);
    assert!(append_batch(&mut buffer, &["bad"]).is_err());
    assert_eq!(buffer, [99, 0]);
    append_batch(&mut buffer, &["2", "3"]).unwrap();
    append_batch(&mut buffer, &[]).unwrap();
    assert_eq!(buffer, [99, 0, 2, 3]);
    assert!(append_batch(&mut buffer, &["1", "4294967296"]).is_err());
    assert_eq!(buffer, [99, 0, 2, 3]);
}

#[test]
fn step9_allocator_is_generic_and_preserves_values() {
    let a: Vec<u32, System> = copy_into(&[3, 0, u32::MAX], System);
    assert_eq!(&*a, &[3, 0, u32::MAX]);
    let b: Vec<u32, Global> = copy_into(&[9, 8], Global);
    assert_eq!(&*b, &[9, 8]);
    let empty: Vec<u32, System> = copy_into(&[], System);
    assert!(empty.is_empty());
    let boxed = Box::new_in(42, System);
    assert_eq!(*boxed, 42);
}
