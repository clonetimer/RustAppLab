use std::env;
use std::io::{self, Write};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use book_d6_foundation_lab::{
    benchmark_sums, compare_guess, config_dir_from, parse_guess, secret_from_seed,
    word_frequencies, GuessOutcome,
};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("hello") => {
            println!("Hello, world!");
            ExitCode::SUCCESS
        }
        Some("words") if args.len() == 2 => {
            let mut entries: Vec<_> = word_frequencies(&args[1]).into_iter().collect();
            entries.sort_by(|left, right| left.0.cmp(&right.0));
            for (word, count) in entries {
                println!("{word}: {count}");
            }
            ExitCode::SUCCESS
        }
        Some("config") => {
            let home = env::var_os("RUSTAPPLAB_HOME")
                .or_else(|| env::var_os("HOME"))
                .or_else(|| env::var_os("USERPROFILE"));
            match config_dir_from(home.as_deref()) {
                Some(path) => {
                    println!("{}", path.display());
                    ExitCode::SUCCESS
                }
                None => {
                    eprintln!("No home directory is available; set RUSTAPPLAB_HOME.");
                    ExitCode::FAILURE
                }
            }
        }
        Some("guess") => run_guessing_game(),
        Some("bench") if args.len() == 2 => run_benchmark(&args[1]),
        _ => {
            eprintln!("Usage: foundation-lab <hello|guess|words <text>|config|bench <items>>");
            ExitCode::from(2)
        }
    }
}

fn run_guessing_game() -> ExitCode {
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let secret = secret_from_seed(seed);

    println!("Guess the number (1..=100). Type q to quit.");
    loop {
        print!("Your guess: ");
        if io::stdout().flush().is_err() {
            return ExitCode::FAILURE;
        }

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => return ExitCode::SUCCESS,
            Ok(_) if input.trim().eq_ignore_ascii_case("q") => return ExitCode::SUCCESS,
            Ok(_) => match parse_guess(&input) {
                Ok(guess @ 1..=100) => match compare_guess(secret, guess) {
                    GuessOutcome::TooSmall => println!("Too small."),
                    GuessOutcome::Correct => {
                        println!("Correct!");
                        return ExitCode::SUCCESS;
                    }
                    GuessOutcome::TooLarge => println!("Too large."),
                },
                Ok(_) => println!("Enter a number from 1 to 100."),
                Err(_) => println!("That is not a valid number."),
            },
            Err(error) => {
                eprintln!("Could not read input: {error}");
                return ExitCode::FAILURE;
            }
        }
    }
}

fn run_benchmark(input: &str) -> ExitCode {
    let count = match input.parse::<usize>() {
        Ok(count) if count > 0 => count,
        _ => {
            eprintln!("The benchmark item count must be a positive integer.");
            return ExitCode::from(2);
        }
    };
    let values: Vec<u64> = (0..count as u64).collect();
    let result = benchmark_sums(&values, 100);
    println!("loop total: {}", result.loop_total);
    println!("iterator total: {}", result.iterator_total);
    println!("loop elapsed: {:?}", result.loop_elapsed);
    println!("iterator elapsed: {:?}", result.iterator_elapsed);
    println!("Timing is illustrative; no implementation is guaranteed to win.");
    ExitCode::SUCCESS
}
