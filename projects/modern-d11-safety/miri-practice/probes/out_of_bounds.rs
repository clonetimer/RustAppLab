//! MIRI-ONLY intentional UB. Do not run natively.
fn main() {
    let data = [10u8, 20];
    // Deliberately read one past this local array. This violates the safety contract.
    let _ = std::hint::black_box(unsafe { *data.as_ptr().add(data.len()) });
}
