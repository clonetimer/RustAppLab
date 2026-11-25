use d9_stdlib_lab::{extract_outliers, inspect_readings, transitions};
use std::process::ExitCode;

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments.is_empty() {
        eprintln!("usage: telemetry <u32> ...");
        return ExitCode::FAILURE;
    }
    let readings: Vec<_> = arguments.iter().map(String::as_str).collect();
    let (accepted, good, bad) = inspect_readings(&readings);
    let mut signed: Vec<i32> = accepted.iter().map(|&value| i32::try_from(value).unwrap_or(i32::MAX)).collect();
    let outliers = extract_outliers(&mut signed, 100);
    println!("good={good}; bad={bad}; outliers={outliers:?}; changes={:?}", transitions(&signed));
    ExitCode::SUCCESS
}
