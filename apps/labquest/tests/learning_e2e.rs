//! Exercise the learner-facing CLI, not just generated strings.
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let p = std::env::temp_dir().join(format!("learning-e2e-{}-{stamp}", std::process::id()));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn call(args: &[&std::ffi::OsStr]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_labquest"))
        .args(args)
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap()
}
fn learn(root: &Path, cmd: &str, tail: &[&str]) -> Output {
    let mut args = vec!["learn".as_ref(), cmd.as_ref(), root.as_os_str()];
    args.extend(tail.iter().map(|s| std::ffi::OsStr::new(*s)));
    call(&args)
}
fn text(out: &Output) -> String {
    format!(
        "{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}
fn patch_step(root: &Path, reference: &str, step: usize) {
    let path = root.join("src/lib.rs");
    let source = fs::read_to_string(&path).unwrap();
    let begin = format!("// LESSON {step} BEGIN");
    let end = format!("// LESSON {step} END");
    let a = source.find(&begin).unwrap();
    let b = source.find(&end).unwrap();
    let ra = reference.find(&begin).unwrap();
    let rb = reference.find(&end).unwrap();
    fs::write(
        path,
        format!("{}{}{}", &source[..a], &reference[ra..rb], &source[b..]),
    )
    .unwrap();
}
#[test]
fn unitlab_guided_loop_does_not_hide_failure_or_skip_regressions() {
    let t = Temp::new();
    let p = t.0.join("我的 UnitLab");
    labquest::studio::init("unitlab", &p).unwrap();
    let initial = fs::read(p.join("src/lib.rs")).unwrap();
    for command in ["lesson", "next"] {
        assert!(learn(&p, command, &[]).status.success());
    }
    assert!(learn(&p, "hint", &["1"]).status.success());
    assert_eq!(fs::read(p.join("src/lib.rs")).unwrap(), initial);
    let failed = learn(&p, "check", &[]);
    assert!(!failed.status.success());
    assert!(text(&failed).contains("学习反馈：第 1 步"));
    let reference = include_str!("../../unitlab/src/lib.rs");
    patch_step(&p, reference, 1);
    let pass1 = learn(&p, "check", &[]);
    assert!(pass1.status.success(), "{}", text(&pass1));
    assert!(text(&learn(&p, "lesson", &[])).contains("第 2 步"));
    patch_step(&p, reference, 2);
    let pass2 = learn(&p, "check", &[]);
    assert!(pass2.status.success(), "{}", text(&pass2));
    assert!(text(&pass2).contains("第 3 步"));
    // Step 3 is still the template: navigate to 3, never erroneously to 2.
    let failed3 = learn(&p, "check", &[]);
    assert!(!failed3.status.success());
    assert!(text(&learn(&p, "next", &[])).contains("第一项失败在第 3 步"));
    patch_step(&p, reference, 3);
    let done = learn(&p, "check", &[]);
    assert!(done.status.success(), "{}", text(&done));
    assert!(text(&done).contains("studio start quizdeck"));
    // Change previously passed behavior. A fresh check MUST catch the regression.
    fs::write(
        p.join("src/lib.rs"),
        reference.replace("celsius * 9.0 / 5.0 + 32.0", "celsius * 9.0 / 5.0 + 1.0"),
    )
    .unwrap();
    let broken = learn(&p, "check", &[]);
    assert!(!broken.status.success());
    assert!(text(&learn(&p, "next", &[])).contains("第一项失败在第 1 步"));
}
#[test]
fn compilation_failure_reports_real_code_and_learner_location() {
    let t = Temp::new();
    let p = t.0.join("bad");
    labquest::studio::init("unitlab", &p).unwrap();
    let file = p.join("src/lib.rs");
    let code = fs::read_to_string(&file)
        .unwrap()
        .replace("    0.0\n", "    0.0;\n");
    fs::write(&file, &code).unwrap();
    let failed = learn(&p, "check", &["2"]);
    assert!(!failed.status.success());
    let msg = text(&failed);
    assert!(msg.contains("E0308"), "{msg}");
    assert!(msg.contains("分号"));
    assert!(msg.contains("src/lib.rs"));
    assert!(msg.contains("自己的"));
    assert_eq!(fs::read_to_string(&file).unwrap(), code);
    assert!(text(&learn(&p, "next", &[])).contains("不是把学习进度重置"));
    let reports = fs::read_dir(p.join(".studio-reports")).unwrap();
    assert!(reports
        .map(|f| fs::read_to_string(f.unwrap().path()).unwrap())
        .any(|s| s.contains("### 教学解释") && s.contains("E0308")));
}
#[test]
fn quizdeck_first_step_and_textscope_first_step_are_isolated() {
    let t = Temp::new();
    for (id, reference) in [
        ("quizdeck", include_str!("../../quizdeck/src/lib.rs")),
        ("textscope", include_str!("../../textscope/src/lib.rs")),
    ] {
        let p = t.0.join(id);
        labquest::studio::init(id, &p).unwrap();
        assert!(!learn(&p, "check", &["1"]).status.success());
        patch_step(&p, reference, 1);
        let out = learn(&p, "check", &["1"]);
        assert!(out.status.success(), "{}", text(&out));
        assert!(text(&learn(&p, "lesson", &[])).contains("第 2 步"));
        assert!(!learn(&p, "check", &["2"]).status.success());
        assert!(text(&learn(&p, "next", &[])).contains("第一项失败在第 2 步"));
    }
}
#[test]
fn firstcard_can_check_without_learning_platform_internals() {
    let t = Temp::new();
    let p = t.0.join("卡片 space");
    labquest::beginner::init(&p).unwrap();
    assert!(!learn(&p, "check", &[]).status.success());
    let file = p.join("src/main.rs");
    let source = fs::read_to_string(&file).unwrap();
    fs::write(
        &file,
        source.replace(
            "println!(\"等待完成第一张任务卡\");",
            "println!(\"今日任务卡 | {} | {}\", name, task);",
        ),
    )
    .unwrap();
    let out = learn(&p, "check", &[]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(text(&out).contains("studio start unitlab"));
}
#[test]
fn utf16_power_shell_error_file_works_through_cli() {
    let t = Temp::new();
    let file = t.0.join("错误 日志.txt");
    let mut bytes = vec![0xff, 0xfe];
    for c in "error[E0382]: moved\n --> src/main.rs:4:2\n".encode_utf16() {
        bytes.extend_from_slice(&c.to_le_bytes());
    }
    fs::write(&file, bytes).unwrap();
    let out = call(&["learn".as_ref(), "diagnose".as_ref(), file.as_os_str()]);
    assert!(out.status.success());
    assert!(text(&out).contains("所有权"));
    assert!(text(&out).contains("src/main.rs:4:2"));
}
#[test]
fn malformed_commands_do_not_start_checks() {
    let t = Temp::new();
    let p = t.0.join("p");
    labquest::studio::init("unitlab", &p).unwrap();
    for (cmd, args) in [
        ("check", vec!["0"]),
        ("hint", vec!["4"]),
        ("lesson", vec!["9"]),
    ] {
        assert!(!learn(&p, cmd, &args).status.success());
    }
    assert!(!p.join(".studio-reports").exists());
}

fn systems_guided_loop(id: &str, reference: &str, next_product: &str) {
    let t = Temp::new();
    let p = t.0.join(format!("自己的 {id}"));
    labquest::studio::init(id, &p).unwrap();
    let before = fs::read(p.join("src/lib.rs")).unwrap();
    let card = fs::read_to_string(p.join("TASK_CARDS.md")).unwrap();
    assert!(p.join("TASK_CARDS.html").is_file());
    assert!(fs::read_to_string(p.join("START.html"))
        .unwrap()
        .contains("href=\"TASK_CARDS.html\""));
    assert_eq!(card.matches("<details>").count(), 9);
    for step in ["1", "2", "3"] {
        let out = learn(&p, "lesson", &[step]);
        assert!(out.status.success());
        assert!(text(&out).contains("先预测"));
        assert!(text(&out).contains("行附近"));
    }
    assert_eq!(before, fs::read(p.join("src/lib.rs")).unwrap());
    assert!(!p.join(".studio-reports").exists());
    let red = learn(&p, "check", &["1"]);
    assert!(!red.status.success());
    assert!(text(&red).contains("学习反馈：第 1 步"));
    if id == "minikv" {
        // MiniKV reference predates lesson markers. Extract only its validation body.
        let start = reference.find("pub fn validate_key(").unwrap();
        let body = start + reference[start..].find('{').unwrap() + 1;
        let end = body + reference[body..].find("\n}").unwrap();
        let path = p.join("src/lib.rs");
        let mut starter = fs::read_to_string(&path).unwrap();
        let a = starter.find("// LESSON 1 BEGIN").unwrap();
        let b = starter.find("// LESSON 1 END").unwrap();
        starter.replace_range(
            a..b,
            &format!("// LESSON 1 BEGIN\n{}\n", &reference[body..end]),
        );
        fs::write(path, starter).unwrap();
    } else {
        patch_step(&p, reference, 1);
    }
    let first = learn(&p, "check", &["1"]);
    assert!(first.status.success(), "{id}: {}", text(&first));
    assert!(text(&learn(&p, "next", &[])).contains("第 2 步"));
    let second = learn(&p, "check", &["2"]);
    assert!(!second.status.success(), "{id}: missing stage 2 must fail");
    assert!(text(&learn(&p, "next", &[])).contains("第一项失败在第 2 步"));
    // Maintainer fixture, not a learner achievement record.
    fs::write(p.join("src/lib.rs"), reference).unwrap();
    let complete = learn(&p, "check", &["all"]);
    assert!(complete.status.success(), "{id}: {}", text(&complete));
    assert!(text(&learn(&p, "next", &[])).contains(next_product));
    fs::write(
        p.join("src/lib.rs"),
        format!("{reference}\n// a learner edit\n"),
    )
    .unwrap();
    assert!(text(&learn(&p, "next", &[])).contains("最近报告与当前源码不同"));
}
#[test]
fn batchmeter_systems_guidance_is_real_and_step_isolated() {
    systems_guided_loop(
        "batchmeter",
        include_str!("../../batchmeter/src/lib.rs"),
        "studio start minikv",
    );
}
#[test]
fn minikv_systems_guidance_is_real_and_step_isolated() {
    systems_guided_loop(
        "minikv",
        include_str!("../../minikv/src/lib.rs"),
        "studio start netfetch",
    );
}
#[test]
fn netfetch_systems_guidance_is_real_and_step_isolated() {
    systems_guided_loop(
        "netfetch",
        include_str!("../../netfetch/src/lib.rs"),
        "studio start servicedesk",
    );
}
#[test]
fn servicedesk_systems_guidance_is_real_and_step_isolated() {
    systems_guided_loop(
        "servicedesk",
        include_str!("../../servicedesk/src/lib.rs"),
        "studio start storedesk",
    );
}

// These references lack lesson markers; transplant only the current function body.
fn patch_function(root: &Path, reference: &str, name: &str, step: usize) {
    let start = reference.find(&format!("fn {name}(")).unwrap();
    let a = start + reference[start..].find('{').unwrap() + 1;
    let b = a + reference[a..].find("\n}").unwrap();
    let path = root.join("src/lib.rs");
    let mut source = fs::read_to_string(&path).unwrap();
    let mark = format!("// LESSON {step} BEGIN");
    let sa = source.find(&mark).unwrap();
    let sb = source.find(&format!("// LESSON {step} END")).unwrap();
    source.replace_range(sa..sb, &format!("{mark}\n{}\n    ", &reference[a..b]));
    if name == "save_item" {
        source = source.replace("pub mod sqlite;", "pub mod sqlite;\nuse sqlite::Param;");
    }
    fs::write(path, source).unwrap();
}
fn reliability_loop(id: &str, reference: &str, functions: [&str; 3], next: &str) {
    let t = Temp::new();
    let root = t.0.join(format!("我的 {id}"));
    labquest::studio::init(id, &root).unwrap();
    let initial = fs::read(root.join("src/lib.rs")).unwrap();
    for n in ["1", "2", "3"] {
        let out = learn(&root, "lesson", &[n]);
        assert!(out.status.success(), "{}", text(&out));
        assert!(text(&out).contains("行附近"));
    }
    assert_eq!(initial, fs::read(root.join("src/lib.rs")).unwrap());
    for (i, function) in functions.into_iter().enumerate() {
        let n = (i + 1).to_string();
        assert!(
            !learn(&root, "check", &[&n]).status.success(),
            "unimplemented {id}/{n}"
        );
        patch_function(&root, reference, function, i + 1);
        let out = learn(&root, "check", &[&n]);
        assert!(out.status.success(), "{id}/{n}: {}", text(&out));
    }
    assert!(text(&learn(&root, "next", &[])).contains(next));
    // A plausible but wrong completion must fail an actual check, not a source-string lint.
    let wrong = if id == "asyncboard" {
        reference.replace("    delay.await;", "    let _ = delay;")
    } else {
        reference
            .replacen("db.transaction(|tx| {", "(|| { let tx = &*db;", 1)
            .replacen(
                "    })\n}\n\n/// Supplied",
                "    })()\n}\n\n/// Supplied",
                1,
            )
    };
    fs::write(root.join("src/lib.rs"), wrong).unwrap();
    assert!(text(&learn(&root, "next", &[])).contains("最近报告与当前源码不同"));
    let rejected = learn(&root, "check", &["3"]);
    assert!(
        !rejected.status.success(),
        "{id} incorrect implementation must fail"
    );
    assert!(
        text(&rejected).contains("学习反馈：第 3 步"),
        "{}",
        text(&rejected)
    );
}
#[test]
fn storedesk_guidance_rejects_partial_commits() {
    reliability_loop(
        "storedesk",
        include_str!("../../storedesk/src/lib.rs"),
        ["validate_item", "save_item", "apply_changes"],
        "studio start asyncboard",
    );
}
#[test]
fn asyncboard_guidance_rejects_skipped_await() {
    reliability_loop(
        "asyncboard",
        include_str!("../../asyncboard/src/lib.rs"),
        ["parse_jobs", "render_report", "complete"],
        "app init taskpocket",
    );
}
