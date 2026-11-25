trait A {} trait B: A {} struct Probe; impl A for Probe {} impl B for Probe {}
fn upcast(value: &dyn B) -> &dyn A { value }
fn main() { let value = Probe; let _ = upcast(&value); println!("ok"); }
