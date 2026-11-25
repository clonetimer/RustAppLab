use d8_edition2024::select_alarm;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        eprintln!("usage: alarm <reading|missing> <limit>");
        return ExitCode::FAILURE;
    }
    let reading = if args[0] == "missing" {
        None
    } else {
        match args[0].parse::<i32>() {
            Ok(value) => Some(value),
            Err(_) => {
                eprintln!("invalid reading");
                return ExitCode::FAILURE;
            }
        }
    };
    let limit = match args[1].parse::<i32>() {
        Ok(value) => value,
        Err(_) => {
            eprintln!("invalid limit");
            return ExitCode::FAILURE;
        }
    };
    match select_alarm(reading, limit) {
        Some(value) => println!("alarm:{value}"),
        None => println!("no alarm"),
    }
    ExitCode::SUCCESS
}
