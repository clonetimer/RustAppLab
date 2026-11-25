fn valid() -> Result<u32, !> { Ok(5) }
fn main() { let Ok(value) = valid(); assert_eq!(value,5); println!("ok"); }
