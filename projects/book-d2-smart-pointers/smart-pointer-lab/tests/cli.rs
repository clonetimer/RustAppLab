use std::process::Command;

fn run(topic: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_book_d2"))
        .arg(topic)
        .output()
        .expect("Book D2 CLI must start")
}

#[test]
fn box_demo_exercises_indirection_and_trait_object() {
    let output = run("box");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("sum=6"));
    assert!(stdout.contains("boxed=42"));
    assert!(stdout.contains("draw=[owned by Box]"));
}

#[test]
fn deref_demo_exercises_coercion() {
    let output = run("deref");
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), "Hello, Rust!");
}

#[test]
fn drop_demo_runs_destructor() {
    let output = run("drop");
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), "drops=1");
}

#[test]
fn rc_demo_reports_shared_owners() {
    let output = run("rc");
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout).unwrap().contains("owners=2"));
}

#[test]
fn refcell_demo_reports_runtime_borrow_conflict() {
    let output = run("refcell");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("value=2"));
    assert!(stdout.contains("conflict_handled=true"));
}

#[test]
fn weak_demo_releases_parent_without_dangling_reference() {
    let output = run("weak");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("parent_before_drop=root"));
    assert!(stdout.contains("parent_after_drop=none"));
}

#[test]
fn unknown_topic_fails_without_fake_stdout() {
    let output = run("unknown");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr).unwrap().contains("主题只能是"));
}
