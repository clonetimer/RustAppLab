use labquest::advanced;
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let p =
            std::env::temp_dir().join(format!("applab-advanced-test-{}-{n}", std::process::id()));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn reference() -> String {
    include_str!("../../termgarden/src/workbench.rs")
        .replace(
            "use crate::{Garden, Renderer, Task};",
            "use termgarden::{Garden, Renderer, Task};",
        )
        .replace("crate::storage::", "termgarden::storage::")
}
fn mask(mut code: String, signature: &str) -> String {
    let start = code.find(signature).unwrap();
    let open = code[start..].find('{').unwrap() + start;
    let mut depth = 1;
    let mut end = open + 1;
    for (i, ch) in code[open + 1..].char_indices() {
        if ch == '{' {
            depth += 1;
        }
        if ch == '}' {
            depth -= 1;
        }
        if depth == 0 {
            end = open + 1 + i;
            break;
        }
    }
    code.replace_range(open + 1..end, " todo!(\"stage not implemented\") ");
    code
}
fn install(root: &Path, code: &str) {
    fs::write(root.join("src/lib.rs"), code).unwrap();
}
#[test]
fn creates_a_standalone_project_without_advanced_answer_and_refuses_overwrite() {
    let temp = Temp::new();
    let p = temp.0.join("my 中文 project");
    advanced::init(&p).unwrap();
    assert!(p.join("START.html").is_file());
    assert!(p.join("DESIGN_REVIEW.md").is_file());
    assert!(!fs::read_to_string(p.join("support/termgarden/src/lib.rs"))
        .unwrap()
        .contains("pub mod workbench"));
    assert!(fs::read_to_string(p.join("src/lib.rs"))
        .unwrap()
        .contains("todo!"));
    assert!(advanced::init(&p).is_err());
    assert!(!p.join(".advanced-reports").exists());
}
#[test]
fn starter_and_deleted_learner_tests_do_not_pass() {
    let temp = Temp::new();
    let p = temp.0.join("p");
    advanced::init(&p).unwrap();
    fs::remove_dir_all(p.join("tests")).unwrap();
    assert!(!advanced::check(&p, 1).unwrap());
    assert!(advanced::check(&p, 4).is_err());
}
#[test]
fn stage_one_can_pass_without_later_steps_and_full_implementation_passes() {
    let temp = Temp::new();
    let p = temp.0.join("p");
    advanced::init(&p).unwrap();
    let mut partial = reference();
    for signature in [
        "pub fn edit<T, F>",
        "pub fn rename_where<P, F>",
        "pub fn undo(",
        "pub fn spawn_report<R>",
    ] {
        partial = mask(partial, signature);
    }
    install(&p, &partial);
    assert!(advanced::check(&p, 1).unwrap());
    assert!(advanced::next(&p).unwrap().contains("第 2 步"));
    assert!(!advanced::check(&p, 2).unwrap());
    install(&p, &reference());
    assert!(advanced::next(&p).unwrap().contains("旧报告失效"));
    assert!(advanced::check(&p, 3).unwrap());
    assert!(advanced::next(&p).unwrap().contains("3/3历史通过记录"));
    fs::write(p.join("src/personal.rs"), "// my next change").unwrap();
    assert!(advanced::next(&p).unwrap().contains("旧报告失效"));
}
#[test]
fn broken_undo_is_caught_at_stage_two() {
    let temp = Temp::new();
    let p = temp.0.join("p");
    advanced::init(&p).unwrap();
    let code = reference().replace(
        "session.garden = session.undo.pop().expect(\"checked nonempty\");",
        "session.undo.pop();",
    );
    assert_ne!(code, reference());
    install(&p, &code);
    assert!(advanced::check(&p, 1).unwrap());
    assert!(!advanced::check(&p, 2).unwrap());
}
#[cfg(unix)]
#[test]
fn linked_sources_are_rejected_before_execution() {
    let temp = Temp::new();
    let p = temp.0.join("p");
    advanced::init(&p).unwrap();
    let original = p.join("src/lib.rs");
    fs::remove_file(&original).unwrap();
    std::os::unix::fs::symlink("/etc/passwd", &original).unwrap();
    assert!(advanced::check(&p, 1).is_err());
}
#[test]
fn lessons_are_read_only_and_distinct() {
    assert!(advanced::lesson(1).unwrap().contains("Weak"));
    assert!(advanced::lesson(2).unwrap().contains("FnOnce"));
    assert!(advanced::lesson(3).unwrap().contains("Send"));
    assert!(advanced::lesson(0).is_err());
    let temp = Temp::new();
    let p = temp.0.join("p");
    advanced::init(&p).unwrap();
    let before = fs::read(p.join("src/lib.rs")).unwrap();
    labquest::run(&[
        "learn".into(),
        "lesson".into(),
        p.as_os_str().into(),
        "1".into(),
    ])
    .unwrap();
    assert_eq!(fs::read(p.join("src/lib.rs")).unwrap(), before);
    assert!(!p.join(".advanced-reports").exists());
}
