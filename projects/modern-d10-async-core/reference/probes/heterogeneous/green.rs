use std::future::Future;
use std::pin::Pin;
fn main() {
    let jobs: Vec<Pin<Box<dyn Future<Output = i32>>>> =
        vec![Box::pin(async { 1 }), Box::pin(async { 2 })];
    assert_eq!(jobs.len(), 2);
    println!("ok");
}
