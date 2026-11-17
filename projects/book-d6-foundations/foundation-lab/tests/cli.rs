use std::io::Write;
use std::process::{Command, Stdio};

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_foundation-lab"))
        .args(args)
        .output()
        .expect("foundation-lab should start")
}

fn run_with_env(args: &[&str], key: &str, value: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_foundation-lab"))
        .args(args)
        .env(key, value)
        .output()
        .expect("foundation-lab should start")
}

fn run_with_input(args: &[&str], input: &[u8]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_foundation-lab"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("foundation-lab should start");
    child
        .stdin
        .take()
        .expect("stdin should be piped")
        .write_all(input)
        .expect("input should be written");
    child.wait_with_output().expect("foundation-lab should exit")
}

#[test]
fn hello_command_prints_the_first_rust_program() {
    let output = run(&["hello"]);
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "Hello, world!\n");
}

#[test]
fn words_command_prints_sorted_hashmap_results() {
    let output = run(&["words", "Rust rust Ownership"]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "ownership: 1\nrust: 2\n"
    );
}

#[test]
fn invalid_command_returns_a_nonzero_exit_code_and_usage() {
    let output = run(&["unknown"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("Usage: foundation-lab"));
}

#[test]
fn config_command_uses_explicit_environment_override() {
    let output = run_with_env(&["config"], "RUSTAPPLAB_HOME", "D:/Rust AppLab Home");
    assert!(output.status.success());
    let path = String::from_utf8(output.stdout).unwrap();
    assert!(path.contains("Rust AppLab Home"));
    assert!(path.trim_end().ends_with(".rustapplab"));
}

#[test]
fn benchmark_command_compares_equal_totals() {
    let output = run(&["bench", "10"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("loop total: 45"));
    assert!(stdout.contains("iterator total: 45"));
    assert!(stdout.contains("no implementation is guaranteed to win"));
}

#[test]
fn guessing_game_accepts_the_quit_command() {
    let output = run_with_input(&["guess"], b"q\n");
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("Guess the number (1..=100)"));
}
