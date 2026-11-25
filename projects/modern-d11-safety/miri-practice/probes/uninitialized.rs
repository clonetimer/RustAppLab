//! MIRI-ONLY intentional UB. Do not run natively.
use std::mem::MaybeUninit;
fn main() {
    let data = [MaybeUninit::<u32>::uninit()];
    // Deliberately assume initialization without writing the element.
    let _ = std::hint::black_box(unsafe { data[0].assume_init() });
}
