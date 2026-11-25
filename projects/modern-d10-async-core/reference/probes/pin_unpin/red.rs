use std::marker::PhantomPinned;
use std::pin::Pin;
fn main() {
    let mut value = PhantomPinned;
    let _pinned = Pin::new(&mut value);
}
