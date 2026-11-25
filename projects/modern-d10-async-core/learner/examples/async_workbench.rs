use d10_async_core::runtime::run_until;
use d10_async_core::*;
use std::time::Duration;

fn main() {
    let limit = Duration::from_secs(3);
    let (joined, polls) = run_until(join_pair(YieldOnce::new(7), YieldOnce::new(9)), limit);
    println!("joined={:?}; polls={polls}", joined.expect("join"));
    let tasks: Vec<BoxTask<'_, i32>> = vec![
        Box::pin(YieldOnce::new(8)),
        Box::pin(async { 3 }),
        Box::pin(async { YieldOnce::new(5).await }),
    ];
    println!("ordered={:?}", run_until(join_many(tasks), limit).0.expect("join_many"));
    println!("race={:?}", run_until(race_pair(YieldOnce::new(1), std::future::ready(2)), limit).0.expect("race"));
    let (sender, receiver) = signal();
    let producer = std::thread::spawn(move || sender.send(42).expect("send"));
    println!("signal={:?}", run_until(receiver, limit).0.expect("signal"));
    producer.join().expect("producer");
    println!("pinned={}", run_until(pin_probe(31), limit).0.expect("pin"));
}
