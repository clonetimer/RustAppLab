use futures_util::{StreamExt, stream};
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let values = stream::once(async { 7 });
    tokio::pin!(values);
    assert_eq!(values.next().await, Some(7));
    assert_eq!(values.next().await, None);
    println!("ok");
}
