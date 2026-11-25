use d8_edition2024::{ensure_threshold, legacy_kind, select_alarm};
use std::cell::RefCell;

#[test]
fn step1_accepts_only_valid_alarms() {
    assert_eq!(select_alarm(Some(45), 40), Some(45));
    assert_eq!(select_alarm(Some(40), 40), Some(40));
    assert_eq!(select_alarm(Some(39), 40), None);
    assert_eq!(select_alarm(None, 40), None);
    assert_eq!(select_alarm(Some(-1), -2), None);
    assert_eq!(select_alarm(Some(0), 0), Some(0));
    assert_eq!(select_alarm(Some(i32::MAX), i32::MAX), Some(i32::MAX));
}

#[test]
fn step2_retains_or_initializes_threshold() {
    let slot = RefCell::new(None);
    assert_eq!(ensure_threshold(&slot, 42), 42);
    assert_eq!(*slot.borrow(), Some(42));
    assert_eq!(ensure_threshold(&slot, 10), 42);
    assert_eq!(*slot.borrow(), Some(42));

    let zero = RefCell::new(Some(0));
    assert_eq!(ensure_threshold(&zero, 99), 0);
    assert_eq!(*zero.borrow(), Some(0));
}

#[test]
fn step3_preserves_legacy_macro_dispatch() {
    assert_eq!(legacy_kind!(42), "expr");
    assert_eq!(legacy_kind!(const { 2 + 2 }), "const");
    assert_eq!(legacy_kind!(const { 0 }), "const");
    assert_eq!(legacy_kind!({ 7 }), "expr");
}
