//! Edit only these three tasks; the initial business checks deliberately fail.
use std::cell::RefCell;

pub fn select_alarm(_reading: Option<i32>, _limit: i32) -> Option<i32> {
    // TODO(1): use an edition-2024 let chain to accept valid alarm readings.
    None
}

pub fn ensure_threshold(_slot: &RefCell<Option<u32>>, default: u32) -> u32 {
    // TODO(2): read the current value or initialize the missing one with if let.
    default
}

#[macro_export]
macro_rules! legacy_kind {
    // TODO(3): retain the old macro dispatch rule while compiling in edition 2024.
    ($value:expr) => {
        "expr"
    };
    (const $value:expr) => {
        "const"
    };
}
