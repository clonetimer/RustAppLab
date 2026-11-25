trait Source { async fn sample(&self) -> u32; }
fn inspect(_: &dyn Source) {}
fn main() {}
