trait A {} trait B {} fn unrelated(value: &dyn A) -> &dyn B { value }
fn main() {}
