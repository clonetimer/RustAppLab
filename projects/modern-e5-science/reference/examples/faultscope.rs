use e5_faultscope::{evaluate, train};
use e5_faultscope::synthetic::{extract, grouped_windows};
use std::collections::BTreeSet;
use std::hint::black_box;
use std::time::Instant;

fn main() {
    let (train_windows, test_windows) = grouped_windows();
    let train_groups: BTreeSet<_> = train_windows.iter().map(|row| row.group).collect();
    let test_groups: BTreeSet<_> = test_windows.iter().map(|row| row.group).collect();
    assert!(train_groups.is_disjoint(&test_groups));
    let training = extract(&train_windows).expect("train features");
    let testing = extract(&test_windows).expect("held-out features");
    let fit = train(&training, 400, 0.1).expect("complete training TODO");
    let predictions: Vec<_> = testing.iter().map(|row| (row.label, fit.model.probability(row.x).unwrap())).collect();
    let result = evaluate(&predictions).expect("complete evaluation TODO");
    let repeats = 2000_u64;
    let start = Instant::now();
    let mut checksum = 0.0;
    for _ in 0..repeats {
        for row in &testing {
            checksum += black_box(fit.model.probability(black_box(row.x)).unwrap());
        }
    }
    let elapsed = start.elapsed().as_nanos();
    println!("backend=cpu-f64");
    println!("train_groups={}", train_groups.len());
    println!("test_groups={}", test_groups.len());
    println!("train_windows={}", training.len());
    println!("test_windows={}", testing.len());
    println!("loss_before={:.12}", fit.loss_before);
    println!("loss_after={:.12}", fit.loss_after);
    println!("tp={}", result.tp);
    println!("tn={}", result.tn);
    println!("fp={}", result.fp);
    println!("fn={}", result.fn_);
    println!("accuracy={:.12}", result.accuracy().unwrap());
    println!("inference_count={}", repeats * testing.len() as u64);
    println!("elapsed_ns={elapsed}");
    println!("checksum={:.12}", black_box(checksum));
}
