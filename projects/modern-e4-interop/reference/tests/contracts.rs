use e4_signal_bridge::*;

#[test]
fn step1_classification_boundaries() {
    assert_eq!(e4_abi_version(), 1);
    for (value, expected) in [(-10, 0), (10, 1), (19, 1), (20, 2), (100_000, 2)] {
        assert_eq!(e4_classify(value, 10, 20), expected);
    }
    assert_eq!(classify(0, 5, 5), Err(BAD_INPUT));
    assert_eq!(classify(i32::MIN, 10, 20), Err(BAD_INPUT));
    assert_eq!(classify(0, -100_001, 20), Err(BAD_INPUT));
}

#[test]
fn step2_summary_validates_and_bounds() {
    assert_eq!(summarize(&[-4, 10, 19, 20, 31], 10, 20), Ok(Stats {
        count: 5, min: -4, max: 31, sum: 76, high_count: 2,
    }));
    assert_eq!(summarize(&[], 10, 20), Err(BAD_LENGTH));
    assert_eq!(summarize(&[1; MAX_SAMPLES + 1], 10, 20), Err(BAD_LENGTH));
    assert_eq!(summarize(&[10, i32::MAX], 10, 20), Err(BAD_INPUT));
    assert_eq!(summarize(&[LIMIT; MAX_SAMPLES], 10, 20).unwrap().sum, 409_600_000);
}

#[test]
fn step3_ffi_caller_ownership_and_failure_atomicity() {
    let samples = [-4, 10, 19, 20, 31];
    let sentinel = Stats { count: 99, min: -1, max: -1, sum: -1, high_count: 99 };
    let mut out = sentinel;
    // SAFETY: valid local allocations, disjoint, no concurrent access.
    unsafe {
        assert_eq!(e4_summarize(samples.as_ptr(), 5, 10, 20, &mut out), 0);
        assert_eq!(out, summarize(&samples, 10, 20).unwrap());
        out = sentinel;
        assert_eq!(e4_summarize(samples.as_ptr(), 5, 20, 20, &mut out), BAD_INPUT);
        assert_eq!(out, sentinel);
        assert_eq!(e4_summarize(std::ptr::null(), 5, 10, 20, &mut out), BAD_POINTER);
        assert_eq!(e4_summarize(samples.as_ptr(), 5, 10, 20, std::ptr::null_mut()), BAD_POINTER);
        assert_eq!(e4_summarize(samples.as_ptr(), 0, 10, 20, &mut out), BAD_LENGTH);
        assert_eq!(e4_summarize(samples.as_ptr(), 4097, 10, 20, &mut out), BAD_LENGTH);
        assert_eq!(out, sentinel);
    }
}
