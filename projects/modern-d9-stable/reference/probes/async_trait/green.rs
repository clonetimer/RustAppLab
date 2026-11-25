trait Source { async fn sample(&self) -> u32; }
struct Local; impl Source for Local { async fn sample(&self) -> u32 { 7 } }
async fn inspect<S: Source>(source: &S) -> u32 { source.sample().await }
fn main() { let local = Local; let _future = inspect(&local); println!("ok"); }
