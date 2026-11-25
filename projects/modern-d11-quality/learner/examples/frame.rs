use d11_quality_lab::{decode, encode};
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut arguments = std::env::args().skip(1);
    let Some(payload) = arguments.next() else {
        eprintln!("usage: frame <text> [corrupt]");
        return ExitCode::FAILURE;
    };
    let mut frame = match encode(payload.as_bytes()) {
        Ok(frame) => frame,
        Err(error) => {
            eprintln!("encode rejected: {error:?}");
            return ExitCode::FAILURE;
        }
    };
    match arguments.next().as_deref() {
        Some("corrupt") => {
            let last = frame.len() - 1;
            frame[last] ^= 1;
        }
        None => {}
        _ => {
            eprintln!("unknown option");
            return ExitCode::FAILURE;
        }
    }
    match decode(&frame) {
        Ok(bytes) => {
            println!("decoded_bytes={}; text={}", bytes.len(), String::from_utf8_lossy(bytes));
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("decode rejected: {error:?}");
            ExitCode::FAILURE
        }
    }
}
