use std::alloc::System;
fn main() { let value=Box::new_in(7,System); assert_eq!(*value,7); println!("ok"); }
