use std::mem::MaybeUninit;
fn main() {
    let mut data = [MaybeUninit::<u32>::uninit()];
    data[0].write(42);
    // SAFETY: the sole element was initialized above.
    assert_eq!(unsafe { data[0].assume_init() }, 42);
    println!("ok");
}
