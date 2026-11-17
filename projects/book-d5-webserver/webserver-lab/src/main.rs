use std::env;
use std::io::{self, BufRead};
use std::net::TcpListener;
use std::process::ExitCode;
use std::sync::mpsc;
use std::thread;

use rustapplab_book_d5_webserver::{serve, ThreadPool};

const DEFAULT_ADDRESS: &str = "127.0.0.1:7878";
const DEFAULT_WORKERS: usize = 4;

#[derive(Debug, PartialEq, Eq)]
enum Config {
    Help,
    Serve { address: String, workers: usize },
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Config, String> {
    let mut args = args.into_iter();
    let first = args.next();

    if first.as_deref() == Some("--help") {
        if args.next().is_some() {
            return Err("--help cannot be combined with other arguments".into());
        }
        return Ok(Config::Help);
    }

    let address = first.unwrap_or_else(|| DEFAULT_ADDRESS.to_owned());
    let workers = match args.next() {
        Some(value) => value
            .parse::<usize>()
            .map_err(|_| "worker count must be a positive integer".to_string())?,
        None => DEFAULT_WORKERS,
    };
    if workers == 0 {
        return Err("worker count must be greater than zero".into());
    }
    if args.next().is_some() {
        return Err("usage: book_d5 [ADDR] [WORKERS]".into());
    }

    Ok(Config::Serve { address, workers })
}

fn run(args: impl IntoIterator<Item = String>) -> Result<(), String> {
    match parse_args(args)? {
        Config::Help => {
            println!("Usage: book_d5 [ADDR] [WORKERS]");
            println!("Default address: {DEFAULT_ADDRESS}; default workers: {DEFAULT_WORKERS}");
            println!("Type stop and press Enter in the server terminal to shut down gracefully.");
            Ok(())
        }
        Config::Serve { address, workers } => {
            let listener = TcpListener::bind(&address)
                .map_err(|error| format!("could not bind {address}: {error}"))?;
            let actual_address = listener
                .local_addr()
                .map_err(|error| format!("could not read bound address: {error}"))?;
            let pool = ThreadPool::new(workers).map_err(|error| error.to_string())?;

            println!("Book D5 listening on http://{actual_address}");
            println!("Routes: GET /, GET /sleep; other paths return 404.");
            println!("Type stop and press Enter to drain jobs and shut down.");

            let (shutdown_tx, shutdown_rx) = mpsc::channel();
            let _input_thread = thread::spawn(move || {
                let stdin = io::stdin();
                for line in stdin.lock().lines() {
                    match line {
                        Ok(line) if line.trim().eq_ignore_ascii_case("stop") => {
                            let _ = shutdown_tx.send(());
                            break;
                        }
                        Ok(_) => {}
                        Err(_) => break,
                    }
                }
            });

            serve(listener, pool, shutdown_rx)
                .map_err(|error| format!("server stopped with an error: {error}"))
        }
    }
}

fn main() -> ExitCode {
    match run(env::args().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Book D5: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_args, Config};

    #[test]
    fn defaults_to_localhost_and_four_workers() {
        assert_eq!(
            parse_args(Vec::<String>::new()).unwrap(),
            Config::Serve {
                address: "127.0.0.1:7878".into(),
                workers: 4
            }
        );
    }

    #[test]
    fn accepts_a_custom_address_and_worker_count() {
        assert_eq!(
            parse_args(["127.0.0.1:0".into(), "2".into()]).unwrap(),
            Config::Serve {
                address: "127.0.0.1:0".into(),
                workers: 2
            }
        );
    }

    #[test]
    fn rejects_zero_workers() {
        assert!(parse_args(["127.0.0.1:7878".into(), "0".into()]).is_err());
    }

    #[test]
    fn rejects_invalid_worker_count() {
        assert!(parse_args(["127.0.0.1:7878".into(), "many".into()]).is_err());
    }

    #[test]
    fn rejects_extra_arguments() {
        assert!(parse_args(["127.0.0.1:7878".into(), "2".into(), "extra".into()]).is_err());
    }
}
