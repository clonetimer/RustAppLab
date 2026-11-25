use std::future::Future; use std::task::{Context, Poll, Waker};
fn main() { let mut output = Vec::new(); let mut callback = async || { output.push(1); }; { let mut future = std::pin::pin!(callback()); let mut cx = Context::from_waker(Waker::noop()); assert!(matches!(future.as_mut().poll(&mut cx), Poll::Ready(()))); } assert_eq!(output, [1]); println!("ok"); }
