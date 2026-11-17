use std::process::Command;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_foundation-lab"))
        .args(args)
        .output()
        .expect("foundation-lab should start")
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
