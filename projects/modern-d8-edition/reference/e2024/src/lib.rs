//! Reference solution for the three learner contracts.
use std::cell::RefCell;

pub fn select_alarm(reading: Option<i32>, limit: i32) -> Option<i32> {
    if let Some(value) = reading
        && value >= limit
        && value >= 0
    {
        Some(value)
    } else {
        None
    }
}

pub fn ensure_threshold(slot: &RefCell<Option<u32>>, default: u32) -> u32 {
    if let Some(value) = *slot.borrow() {
        value
    } else {
        *slot.borrow_mut() = Some(default);
        default
    }
}

#[macro_export]
macro_rules! legacy_kind {
    ($value:expr_2021) => {
        "expr"
    };
    (const $value:expr) => {
        "const"
    };
}
