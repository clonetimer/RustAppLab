//! Identical source, compiled inside two crates with different editions.
use std::cell::RefCell;

struct Probe<'a>(&'a RefCell<Vec<&'static str>>);

impl Probe<'_> {
    fn missing(&self) -> Option<u8> {
        None
    }
}

impl Drop for Probe<'_> {
    fn drop(&mut self) {
        self.0.borrow_mut().push("drop");
    }
}

pub fn drop_order() -> Vec<&'static str> {
    let events = RefCell::new(Vec::new());
    if let Some(value) = Probe(&events).missing() {
        assert_eq!(value, 0);
        events.borrow_mut().push("some");
    } else {
        events.borrow_mut().push("else");
    }
    events.into_inner()
}

macro_rules! classify {
    ($value:expr) => {
        "expr"
    };
    (const $value:expr) => {
        "const"
    };
}

pub fn macro_kind() -> &'static str {
    classify!(const { 2 + 2 })
}
