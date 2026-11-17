use std::process::Command;

fn run(topic: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_book_d3"))
        .arg(topic)
        .output()
        .expect("Book D3 CLI must start")
}

#[test]
fn pattern_demo_exposes_literal_range_and_destructuring_results() {
    let output = run("patterns");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("zero"));
    assert!(stdout.contains("small-positive"));
    assert!(stdout.contains("y-axis"));
    assert!(stdout.contains("move:2,5"));
    assert!(stdout.contains("valid-id:42"));
}

#[test]
fn oop_demo_exposes_encapsulation_and_dynamic_dispatch() {
    let output = run("oop");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("average=15.0"));
    assert!(stdout.contains("components=2"));
    assert!(stdout.contains("[button:save]|[text:ready]"));
}

#[test]
fn state_demo_exposes_review_and_approval_transitions() {
    let output = run("state");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("draft-visible=true"));
    assert!(stdout.contains("review-visible=true"));
    assert!(stdout.contains("published=true"));
    assert!(stdout.contains("content=A Rust state-pattern example."));
}

#[test]
fn unknown_topic_fails_without_fake_stdout() {
    let output = run("unknown");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr).unwrap().contains("主题只能是"));
}
