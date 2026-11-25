use futures_util::{StreamExt, stream};
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut values = stream::once(async { 7 });
    assert_eq!(values.next().await, Some(7));
}
