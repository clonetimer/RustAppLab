use std::marker::PhantomPinned;
fn main() {
    let value = Box::pin(PhantomPinned);
    let original = value.as_ref().get_ref() as *const PhantomPinned;
    let moved = value;
    assert_eq!(original, moved.as_ref().get_ref() as *const PhantomPinned);
    println!("ok");
}
