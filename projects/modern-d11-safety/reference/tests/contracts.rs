use d11_safety_lab::{checked_read, initialized_prefix};

#[test]
fn step1_checked_read_has_no_out_of_bounds_access() {
    let bytes = [0, 7, 255];
    for index in 0..=bytes.len() {
        assert_eq!(checked_read(&bytes, index), bytes.get(index).copied());
    }
    assert_eq!(checked_read(&bytes, usize::MAX), None);
    assert_eq!(checked_read(&[], 0), None);
    assert_eq!(checked_read(&bytes[1..], 0), Some(7));
}

#[test]
fn step2_every_array_slot_is_initialized_before_read() {
    let values = [1, u32::MAX, 3];
    assert_eq!(initialized_prefix::<0>(&[]), Some([]));
    assert_eq!(initialized_prefix::<1>(&values), Some([1]));
    assert_eq!(initialized_prefix::<2>(&values), Some([1, u32::MAX]));
    assert_eq!(initialized_prefix::<3>(&values), Some(values));
    assert_eq!(initialized_prefix::<4>(&values), None);
    assert_eq!(initialized_prefix::<1>(&[]), None);
    assert_eq!(initialized_prefix::<2>(&values[1..]), Some([u32::MAX, 3]));
}
