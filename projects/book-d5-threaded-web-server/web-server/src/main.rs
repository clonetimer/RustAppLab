use std::env;
use std::net::TcpListener;
use std::process::ExitCode;
use std::str::FromStr;

use rustapplab_book_d5_web_server::{serve_single, serve_thread_pool};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Config {
    port: u16,
    workers: Option<usize>,
    requests: usize,
}

fn parse_value<T>(value: Option<String>, default: T, label: &str) -> Result<T, String>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    match value {
        Some(value) => value
            .parse()
            .map_err(|error| format!("{label}无效：{error}")),
        None => Ok(default),
    }
}

fn parse_config(args: impl IntoIterator<Item = String>) -> Result<Config, String> {
    let mut args = args.into_iter();
    let mode = args
        .next()
        .ok_or_else(|| "用法：book_d5 <single|pool> [port] [workers] [requests]".to_string())?;
    let port = parse_value(args.next(), 7878_u16, "端口")?;

    let (workers, requests) = match mode.as_str() {
        "single" => (None, parse_value(args.next(), 2_usize, "请求数")?),
        "pool" => {
            let workers = parse_value(args.next(), 4_usize, "工作线程数")?;
            let requests = parse_value(args.next(), 2_usize, "请求数")?;
            (Some(workers), requests)
        }
        _ => return Err("模式只能是 single 或 pool".into()),
    };

    if workers == Some(0) {
        return Err("工作线程数必须大于零".into());
    }
    if requests == 0 {
        return Err("请求数必须大于零，以便演示有序关闭".into());
    }
    if args.next().is_some() {
        return Err("参数过多".into());
    }

    Ok(Config {
        port,
        workers,
        requests,
    })
}

fn run(config: Config) -> std::io::Result<()> {
    // Deliberately bind loopback only; this teaching server is not Internet-facing.
    let listener = TcpListener::bind(("127.0.0.1", config.port))?;
    let address = listener.local_addr()?;
    println!(
        "Listening on http://{address}; requests={}",
        config.requests
    );

    match config.workers {
        Some(workers) => serve_thread_pool(listener, workers, config.requests),
        None => serve_single(listener, config.requests),
    }
}

fn main() -> ExitCode {
    match parse_config(env::args().skip(1)) {
        Ok(config) => match run(config) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("Book D5 server failed: {error}");
                ExitCode::FAILURE
            }
        },
        Err(error) => {
            eprintln!("{error}");
            eprintln!("示例：book_d5 single 7878 2");
            eprintln!("示例：book_d5 pool 7878 4 2");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_config, Config};

    #[test]
    fn single_mode_has_safe_loopback_demo_defaults() {
        assert_eq!(
            parse_config(["single".to_string()]).unwrap(),
            Config {
                port: 7878,
                workers: None,
                requests: 2,
            }
        );
    }

    #[test]
    fn pool_mode_parses_port_worker_count_and_request_count() {
        assert_eq!(
            parse_config([
                "pool".to_string(),
                "0".to_string(),
                "6".to_string(),
                "12".to_string()
            ])
            .unwrap(),
            Config {
                port: 0,
                workers: Some(6),
                requests: 12,
            }
        );
    }

    #[test]
    fn invalid_modes_counts_ports_and_extra_arguments_are_rejected() {
        for args in [
            vec!["unknown".to_string()],
            vec!["single".to_string(), "not-a-port".to_string()],
            vec!["pool".to_string(), "7878".to_string(), "0".to_string()],
            vec!["single".to_string(), "7878".to_string(), "0".to_string()],
            vec!["single".to_string(), "7878".to_string(), "1".to_string(), "extra".to_string()],
        ] {
            assert!(parse_config(args).is_err());
        }
    }
}
