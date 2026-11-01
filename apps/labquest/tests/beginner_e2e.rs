//! 初学者完整红->绿闭环：以真实 cargo 编译和可执行程序输出作为判断依据。
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_workspace(suffix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let base = std::env::temp_dir().join(format!(
        "rustapplab-beginner-{}-{nanos}-{suffix}",
        std::process::id()
    ));
    fs::create_dir(&base).unwrap();
    base
}

fn command(args: &[&OsStr]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_labquest"))
        .args(args)
        .output()
        .unwrap()
}

fn command_from(dir: &std::path::Path, args: &[&OsStr]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_labquest"))
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn single_component_relative_path_works_like_windows_launcher() {
    let base = temp_workspace("relative");
    let project = base.join("my-first-card");
    let started = command_from(
        &base,
        &[
            "beginner".as_ref(),
            "start".as_ref(),
            "my-first-card".as_ref(),
        ],
    );
    assert!(
        started.status.success(),
        "{} / {}",
        String::from_utf8_lossy(&started.stdout),
        String::from_utf8_lossy(&started.stderr)
    );
    assert!(project.join("src/main.rs").is_file());
    let source = project.join("src/main.rs");
    let old = fs::read_to_string(&source).unwrap();
    fs::write(
        &source,
        old.replace("等待完成第一张任务卡", "今日任务卡 | {name} | {task}"),
    )
    .unwrap();
    let checked = command_from(
        &base,
        &[
            "beginner".as_ref(),
            "check".as_ref(),
            "my-first-card".as_ref(),
        ],
    );
    assert!(
        checked.status.success(),
        "{} / {}",
        String::from_utf8_lossy(&checked.stdout),
        String::from_utf8_lossy(&checked.stderr)
    );
    assert!(project.join("FIRST_WIN.md").exists());
    let _ = fs::remove_dir_all(base);
}

#[test]
fn nested_relative_path_needs_existing_parent_and_does_not_create_it() {
    let base = temp_workspace("nested-relative");
    let missing = command_from(
        &base,
        &[
            "beginner".as_ref(),
            "start".as_ref(),
            "missing/card".as_ref(),
        ],
    );
    assert!(!missing.status.success());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("父目录"));
    assert!(!base.join("missing").exists());
    fs::create_dir(base.join("sub")).unwrap();
    let started = command_from(
        &base,
        &["beginner".as_ref(), "start".as_ref(), "sub/card".as_ref()],
    );
    assert!(
        started.status.success(),
        "{}",
        String::from_utf8_lossy(&started.stderr)
    );
    assert!(base.join("sub/card/src/main.rs").is_file());
    let _ = fs::remove_dir_all(base);
}

#[test]
fn first_win_requires_real_edit_and_dynamic_input() {
    let base = temp_workspace("dynamic");
    let project = base.join("中文 任务卡");
    let init = command(&["beginner".as_ref(), "start".as_ref(), project.as_os_str()]);
    assert!(
        init.status.success(),
        "{}",
        String::from_utf8_lossy(&init.stderr)
    );
    assert!(project.join("跟着做.md").is_file());
    let source = project.join("src/main.rs");
    let before = fs::read_to_string(&source).unwrap();
    assert!(before.contains("TODO"));
    let red = command(&["beginner".as_ref(), "check".as_ref(), project.as_os_str()]);
    assert!(!red.status.success(), "初始工程不能通关");
    assert!(!project.join("FIRST_WIN.md").exists());
    fs::write(
        &source,
        before.replace("等待完成第一张任务卡", "今日任务卡 | 小林 | 写 Rust"),
    )
    .unwrap();
    let cheat = command(&["beginner".as_ref(), "check".as_ref(), project.as_os_str()]);
    assert!(!cheat.status.success(), "硬编码单个输出不能通关");
    fs::write(
        &source,
        before.replace("等待完成第一张任务卡", "今日任务卡 | {name} | {task}"),
    )
    .unwrap();
    let green = command(&["beginner".as_ref(), "check".as_ref(), project.as_os_str()]);
    assert!(
        green.status.success(),
        "{} / {}",
        String::from_utf8_lossy(&green.stderr),
        String::from_utf8_lossy(&green.stdout)
    );
    assert!(project.join("FIRST_WIN.md").is_file());
    let saved = fs::read_to_string(project.join("FIRST_WIN.md")).unwrap();
    let repeat = command(&["beginner".as_ref(), "check".as_ref(), project.as_os_str()]);
    assert!(repeat.status.success());
    assert_eq!(
        fs::read_to_string(project.join("FIRST_WIN.md")).unwrap(),
        saved
    );
    let _ = fs::remove_dir_all(base);
}

#[test]
fn refuses_existing_folder_without_overwrite() {
    let base = temp_workspace("protect");
    let project = base.join("first-card");
    fs::create_dir(&project).unwrap();
    fs::write(project.join("珍贵资料.txt"), "do not touch").unwrap();
    let res = command(&["beginner".as_ref(), "start".as_ref(), project.as_os_str()]);
    assert!(!res.status.success());
    assert_eq!(
        fs::read_to_string(project.join("珍贵资料.txt")).unwrap(),
        "do not touch"
    );
    assert!(!project.join("src/main.rs").exists());
    let _ = fs::remove_dir_all(base);
}

#[test]
fn syntax_error_explains_next_step_and_does_not_issue_badge() {
    let base = temp_workspace("syntax");
    let project = base.join("card");
    assert!(
        command(&["beginner".as_ref(), "start".as_ref(), project.as_os_str()])
            .status
            .success()
    );
    fs::write(
        project.join("src/main.rs"),
        "fn main() { println!(\"bad\" }\n",
    )
    .unwrap();
    let res = command(&["beginner".as_ref(), "check".as_ref(), project.as_os_str()]);
    assert!(!res.status.success());
    let msg = String::from_utf8_lossy(&res.stdout);
    assert!(msg.contains("编译没有通过"));
    assert!(msg.contains("分号") || msg.contains("println"));
    assert!(!project.join("FIRST_WIN.md").exists());
    let _ = fs::remove_dir_all(base);
}

#[test]
fn three_tier_hints_work_without_answer_first() {
    for tier in ["1", "2", "3"] {
        let res = command(&["beginner".as_ref(), "hint".as_ref(), tier.as_ref()]);
        assert!(res.status.success());
    }
    assert!(
        !command(&["beginner".as_ref(), "hint".as_ref(), "4".as_ref()])
            .status
            .success()
    );
}

#[test]
fn endless_loop_returns_feedback_instead_of_hanging_forever() {
    let base = temp_workspace("timeout");
    let project = base.join("card");
    assert!(
        command(&["beginner".as_ref(), "start".as_ref(), project.as_os_str()])
            .status
            .success()
    );
    fs::write(
        project.join("src/main.rs"),
        "fn main() { loop { std::hint::spin_loop(); } }\n",
    )
    .unwrap();
    let res = command(&["beginner".as_ref(), "check".as_ref(), project.as_os_str()]);
    assert!(!res.status.success());
    assert!(String::from_utf8_lossy(&res.stdout).contains("超过 4 秒"));
    let _ = fs::remove_dir_all(base);
}

#[test]
fn verbose_stderr_does_not_turn_correct_work_into_a_timeout() {
    let base = temp_workspace("verbose");
    let project = base.join("card");
    labquest::beginner::init(&project).unwrap();
    let source = project.join("src/main.rs");
    let original = fs::read_to_string(&source).unwrap();
    fs::write(&source, original.replace(
        "println!(\"等待完成第一张任务卡\");",
        "eprint!(\"{}\", \"x\".repeat(192 * 1024)); println!(\"今日任务卡 | {name} | {task}\");",
    )).unwrap();
    let result = command(&["beginner".as_ref(), "check".as_ref(), project.as_os_str()]);
    assert!(
        result.status.success(),
        "{} / {}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(project.join("FIRST_WIN.md").is_file());
    let _ = fs::remove_dir_all(base);
}

#[test]
fn output_flood_is_rejected_without_a_badge() {
    let base = temp_workspace("output-limit");
    let project = base.join("card");
    labquest::beginner::init(&project).unwrap();
    fs::write(
        project.join("src/main.rs"),
        "fn main() { print!(\"{}\", \"x\".repeat(2 * 1024 * 1024)); }",
    )
    .unwrap();
    let result = command(&["beginner".as_ref(), "check".as_ref(), project.as_os_str()]);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stdout).contains("1 MiB"));
    assert!(!project.join("FIRST_WIN.md").exists());
    let _ = fs::remove_dir_all(base);
}

#[test]
fn learner_build_is_native_even_when_parent_has_a_cross_target() {
    let base = temp_workspace("parent-target");
    let project = base.join("card");
    labquest::beginner::init(&project).unwrap();
    let path = project.join("src/main.rs");
    let source = fs::read_to_string(&path)
        .unwrap()
        .replace("等待完成第一张任务卡", "今日任务卡 | {name} | {task}");
    fs::write(path, source).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_labquest"))
        .args(["beginner".as_ref(), "check".as_ref(), project.as_os_str()])
        .env("CARGO_BUILD_TARGET", "not-a-real-target")
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{} / {}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    let _ = fs::remove_dir_all(base);
}
