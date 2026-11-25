use d9_stable_lab::{append_batch, copy_into, infallible, positive};
use std::alloc::System;
use std::process::ExitCode;

fn main() -> ExitCode {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    if arguments.is_empty() {
        eprintln!("usage: batch <positive integer> ...");
        return ExitCode::FAILURE;
    }
    let readings: Vec<_> = arguments.iter().map(String::as_str).collect();
    let mut buffer = vec![100];
    if append_batch(&mut buffer, &readings).is_err() {
        eprintln!("batch rejected; retained={buffer:?}");
        return ExitCode::FAILURE;
    }
    let count = readings.iter().filter(|text| positive(Some(text)).is_some()).count();
    let count = infallible(Ok::<_, !>(count));
    let copied = copy_into(&buffer, System);
    println!("accepted={count}; stored={copied:?}");
    ExitCode::SUCCESS
}
