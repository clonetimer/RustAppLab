use std::process::Command;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_cargo-applab-map"))
        .args(args)
        .output()
        .expect("teaching binary must start")
}

#[test]
fn direct_execution_prints_one_chapter() {
    let output = run(&["profiles"]);
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout).unwrap().starts_with("14.1 "));
    assert!(output.stderr.is_empty());
}

#[test]
fn cargo_style_invocation_uses_same_path() {
    let output = run(&["applab-map", "workspace"]);
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout).unwrap().starts_with("14.3 "));
    assert!(output.stderr.is_empty());
}

#[test]
fn wrong_topic_has_nonzero_exit_and_no_stdout() {
    let output = run(&["applab-map", "not-a-topic"]);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr).unwrap().contains("未知主题"));
}

#[test]
fn trailing_args_are_rejected() {
    let output = run(&["profiles", "install"]);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
}
