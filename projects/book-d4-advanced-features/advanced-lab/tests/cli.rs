use std::process::Command;

fn run(topic: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_book_d4"))
        .arg(topic)
        .output()
        .expect("Book D4 CLI must start")
}

#[test]
fn unsafe_demo_exposes_bounded_raw_pointer_and_union_examples() {
    let output = run("unsafe");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("raw-sum=10"));
    assert!(stdout.contains("split-left=[1, 2]"));
    assert!(stdout.contains("split-right=[3, 4]"));
    assert!(stdout.contains("ascii=SAT"));
    assert!(stdout.contains("union-integer=42"));
    assert!(stdout.contains("ffi-identity=17"));
}

#[test]
fn traits_demo_exposes_associated_types_and_qualified_calls() {
    let output = run("traits");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("point=6,8"));
    assert!(stdout.contains("measurement=12"));
    assert!(stdout.contains("qualified=trait name"));
}

#[test]
fn types_demo_exposes_newtype_and_unsized_reference_examples() {
    let output = run("types");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("meters=12"));
    assert!(stdout.contains("display-len=4"));
}

#[test]
fn functions_demo_exposes_function_pointers_and_returned_closures() {
    let output = run("functions");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("function-pointer=5"));
    assert!(stdout.contains("returned-closure=12"));
    assert!(stdout.contains("boxed-closure=30"));
}

#[test]
fn macros_demo_exposes_all_procedural_macro_forms() {
    let output = run("macros");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("derive=Packet"));
    assert!(stdout.contains("attribute=telemetry"));
    assert!(stdout.contains("function-macro=3"));
    assert!(stdout.contains("declarative-macro=(1, \"two\")"));
}

#[test]
fn unknown_topic_fails_without_fake_stdout() {
    let output = run("unknown");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("主题只能是"));
}
