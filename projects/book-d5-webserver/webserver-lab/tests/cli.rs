use std::process::Command;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_book_d5"))
        .args(args)
        .output()
        .expect("Book D5 CLI must start")
}

#[test]
fn help_is_available_without_binding_a_port() {
    let output = run(&["--help"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Usage: book_d5 [ADDR] [WORKERS]"));
}

#[test]
fn zero_worker_count_fails_before_starting_the_server() {
    let output = run(&["127.0.0.1:0", "0"]);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("worker count must be greater than zero"));
}

#[test]
fn extra_arguments_are_rejected() {
    let output = run(&["127.0.0.1:0", "2", "extra"]);
    assert!(!output.status.success());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("usage: book_d5 [ADDR] [WORKERS]"));
}
