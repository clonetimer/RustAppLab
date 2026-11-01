use labquest::studio;
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};
static COUNTER: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let p = std::env::temp_dir().join(format!(
            "studio-test-{}-{now}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
const REFERENCES: &[(&str, &str)] = &[
    ("unitlab", include_str!("../../unitlab/src/lib.rs")),
    ("quizdeck", include_str!("../../quizdeck/src/lib.rs")),
    ("textscope", include_str!("../../textscope/src/lib.rs")),
    ("ledgerlite", include_str!("../../ledgerlite/src/lib.rs")),
    ("badgeforge", include_str!("../../badgeforge/src/lib.rs")),
    ("batchmeter", include_str!("../../batchmeter/src/lib.rs")),
    ("termgarden", include_str!("../../termgarden/src/lib.rs")),
    ("storedesk", include_str!("../../storedesk/src/lib.rs")),
    ("minikv", include_str!("../../minikv/src/lib.rs")),
    ("netfetch", include_str!("../../netfetch/src/lib.rs")),
    ("servicedesk", include_str!("../../servicedesk/src/lib.rs")),
    ("asyncboard", include_str!("../../asyncboard/src/lib.rs")),
];
fn reference(id: &str) -> String {
    let source = REFERENCES.iter().find(|(name, _)| *name == id).unwrap().1;
    // The basic Studio has no advanced module. Keep the same tested base implementation.
    if id == "termgarden" {
        source.replace("pub mod workbench;\n", "")
    } else {
        source.to_owned()
    }
}
fn red_green(id: &str) {
    let t = Temp::new();
    let p = t.0.join(format!("学员 {id}"));
    studio::init(id, &p).unwrap();
    assert!(!studio::check(&p, 1).unwrap(), "未完成模板不应通过 {id}");
    let s = p.join("src/lib.rs");
    fs::write(&s, reference(id)).unwrap();
    assert!(studio::check(&p, 3).unwrap(), "正确参考实现应全部通过 {id}");
    assert_eq!(fs::read_to_string(s).unwrap(), reference(id));
}
#[test]
fn unitlab_red_to_green() {
    red_green("unitlab");
}
#[test]
fn quizdeck_red_to_green() {
    red_green("quizdeck");
}
#[test]
fn textscope_red_to_green() {
    red_green("textscope");
}
#[test]
fn ledgerlite_red_to_green() {
    red_green("ledgerlite");
}
#[test]
fn badgeforge_red_to_green() {
    red_green("badgeforge");
}
#[test]
fn batchmeter_red_to_green() {
    red_green("batchmeter");
}
#[test]
fn only_completed_milestone_passes_and_reports_are_snapshots() {
    let t = Temp::new();
    let p = t.0.join("partial");
    studio::init("unitlab", &p).unwrap();
    let file = p.join("src/lib.rs");
    let starter = fs::read_to_string(&file).unwrap();
    let start = starter.find("// LESSON 1 BEGIN").unwrap();
    let end = starter.find("// LESSON 1 END").unwrap();
    let mut partial = starter.clone();
    partial.replace_range(
        start..end,
        "// LESSON 1 BEGIN\n celsius * 9.0 / 5.0 + 32.0\n",
    );
    fs::write(&file, &partial).unwrap();
    assert!(studio::check(&p, 1).unwrap());
    assert!(!studio::check(&p, 2).unwrap());
    fs::write(&file, starter).unwrap();
    assert!(!studio::check(&p, 1).unwrap());
    let reports: Vec<_> = fs::read_dir(p.join(".studio-reports")).unwrap().collect();
    assert_eq!(reports.len(), 3);
}
#[test]
fn local_test_deletion_does_not_pass() {
    let t = Temp::new();
    let p = t.0.join("cheat");
    studio::init("unitlab", &p).unwrap();
    fs::write(p.join("tests/milestones.rs"), "").unwrap();
    fs::write(p.join("Cargo.toml"), "not a manifest").unwrap();
    assert!(!studio::check(&p, 1).unwrap());
}
#[test]
fn source_cli_breakage_is_detected() {
    let t = Temp::new();
    let p = t.0.join("bad-cli");
    studio::init("unitlab", &p).unwrap();
    fs::write(p.join("src/lib.rs"), reference("unitlab")).unwrap();
    fs::write(p.join("src/main.rs"), "fn main() {println!(\"wrong\");}").unwrap();
    assert!(!studio::check(&p, 3).unwrap());
}
#[test]
fn relative_unicode_space_and_repeated_init() {
    let t = Temp::new();
    let relative = "我的 学习目录";
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_labquest"))
            .current_dir(&t.0)
            .args(["studio", "start", "unitlab", relative])
            .output()
            .unwrap()
    };
    assert!(run().status.success());
    let file = t.0.join(relative).join("src/lib.rs");
    let before = fs::read(&file).unwrap();
    assert!(!run().status.success());
    assert_eq!(fs::read(file).unwrap(), before);
}
#[test]
fn invalid_parameters_and_marker_fail_without_writes() {
    let t = Temp::new();
    assert!(studio::init("unknown", &t.0.join("x")).is_err());
    assert!(!t.0.join("x").exists());
    let p = t.0.join("unit");
    studio::init("unitlab", &p).unwrap();
    assert!(studio::check(&p, 0).is_err());
    fs::write(p.join(".applab-studio"), "wrong\nunitlab").unwrap();
    assert!(studio::check(&p, 1).is_err());
    assert!(!p.join(".studio-reports").exists());
}
#[test]
fn compile_error_is_not_pass() {
    let t = Temp::new();
    let p = t.0.join("syntax");
    studio::init("unitlab", &p).unwrap();
    fs::write(p.join("src/lib.rs"), "not rust!").unwrap();
    assert!(!studio::check(&p, 1).unwrap());
}
#[cfg(unix)]
#[test]
fn symlink_source_and_report_directory_are_rejected() {
    use std::os::unix::fs::symlink;
    let t = Temp::new();
    let p = t.0.join("linked");
    studio::init("unitlab", &p).unwrap();
    let real = t.0.join("real.rs");
    fs::write(&real, reference("unitlab")).unwrap();
    fs::remove_file(p.join("src/lib.rs")).unwrap();
    symlink(&real, p.join("src/lib.rs")).unwrap();
    assert!(studio::check(&p, 1).is_err());
    fs::remove_file(p.join("src/lib.rs")).unwrap();
    fs::write(p.join("src/lib.rs"), reference("unitlab")).unwrap();
    let external = t.0.join("outside");
    fs::create_dir(&external).unwrap();
    symlink(&external, p.join(".studio-reports")).unwrap();
    assert!(studio::check(&p, 1).is_err());
    assert_eq!(fs::read_dir(external).unwrap().count(), 0);
}
#[test]
fn accidental_infinite_loop_is_bounded() {
    let t = Temp::new();
    let p = t.0.join("loop");
    studio::init("unitlab", &p).unwrap();
    let file = p.join("src/lib.rs");
    let s = fs::read_to_string(&file).unwrap();
    let begin = s.find("// LESSON 1 BEGIN").unwrap();
    let end = s.find("// LESSON 1 END").unwrap();
    let mut bad = s.clone();
    bad.replace_range(begin..end,"// LESSON 1 BEGIN\n let _=celsius; loop {std::thread::sleep(std::time::Duration::from_millis(20));}\n");
    fs::write(file, bad).unwrap();
    let start = std::time::Instant::now();
    assert!(!studio::check(&p, 1).unwrap());
    assert!(start.elapsed() < std::time::Duration::from_secs(45));
}

#[test]
fn termgarden_red_to_green() {
    red_green("termgarden");
}
#[test]
fn storedesk_red_to_green() {
    red_green("storedesk");
}
#[test]
fn new_support_modules_and_samples_are_portable() {
    let t = Temp::new();
    for (id, module) in [("termgarden", "storage.rs"), ("storedesk", "sqlite.rs")] {
        let p = t.0.join(id);
        studio::init(id, &p).unwrap();
        assert!(p.join("src").join(module).is_file());
        assert!(!fs::read_to_string(p.join("Cargo.toml"))
            .unwrap()
            .contains("path ="));
        assert_eq!(
            fs::read_to_string(p.join("src/lib.rs"))
                .unwrap()
                .matches("TODO(")
                .count(),
            3
        );
    }
}
#[test]
fn new_starter_passes_step1_without_later_answers() {
    let t = Temp::new();
    for (id, function) in [
        ("termgarden", "parse_command"),
        ("storedesk", "validate_item"),
    ] {
        let p = t.0.join(id);
        studio::init(id, &p).unwrap();
        let file = p.join("src/lib.rs");
        let mut starter = fs::read_to_string(&file).unwrap();
        let reference = reference(id);
        let function_start = reference.find(&format!("pub fn {function}(")).unwrap();
        let body_start = reference[function_start..].find('{').unwrap() + function_start + 1;
        let mut depth = 1;
        let mut body_end = body_start;
        for (offset, ch) in reference[body_start..].char_indices() {
            if ch == '{' {
                depth += 1;
            } else if ch == '}' {
                depth -= 1;
            }
            if depth == 0 {
                body_end = body_start + offset;
                break;
            }
        }
        let begin = starter.find("// LESSON 1 BEGIN").unwrap();
        let end = starter.find("// LESSON 1 END").unwrap();
        starter.replace_range(begin..end, &reference[body_start..body_end]);
        fs::write(file, starter).unwrap();
        assert!(
            studio::check(&p, 1).unwrap(),
            "第一步不能被后两步阻断：{id}"
        );
        assert!(!studio::check(&p, 2).unwrap(), "第二步仍应未完成：{id}");
    }
}

#[test]
fn minikv_red_to_green() {
    red_green("minikv");
}
#[test]
fn minikv_each_stage_is_independently_reachable() {
    let temp = Temp::new();
    let project = temp.0.join("KV 学习");
    studio::init("minikv", &project).unwrap();
    let source = project.join("src/lib.rs");
    let reference = reference("minikv");
    for (step, signature) in [
        (1, "pub fn validate_key("),
        (2, "pub fn execute("),
        (3, "fn next("),
    ] {
        let begin = reference.find(signature).unwrap();
        let body = reference[begin..].find('{').unwrap() + begin + 1;
        let mut depth = 1;
        let mut end = body;
        for (offset, c) in reference[body..].char_indices() {
            if c == '{' {
                depth += 1;
            } else if c == '}' {
                depth -= 1;
            }
            if depth == 0 {
                end = body + offset;
                break;
            }
        }
        let mut current = fs::read_to_string(&source).unwrap();
        let a = current.find(&format!("// LESSON {step} BEGIN")).unwrap();
        let b = current.find(&format!("// LESSON {step} END")).unwrap();
        current.replace_range(a..b, &reference[body..end]);
        fs::write(&source, current).unwrap();
        assert!(studio::check(&project, step).unwrap());
        if step < 3 {
            assert!(!studio::check(&project, step + 1).unwrap());
        }
    }
}
#[test]
fn minikv_wrong_iterator_is_not_full_pass() {
    let temp = Temp::new();
    let project = temp.0.join("bad-next");
    studio::init("minikv", &project).unwrap();
    let good = reference("minikv");
    let bad = good.replace("key.starts_with(&self.prefix)", "true");
    assert_ne!(bad, good);
    fs::write(project.join("src/lib.rs"), bad).unwrap();
    assert!(studio::check(&project, 2).unwrap());
    assert!(!studio::check(&project, 3).unwrap());
}

#[test]
fn netfetch_studio_red_green() {
    red_green("netfetch");
}
#[test]
fn servicedesk_studio_red_green() {
    red_green("servicedesk");
}

#[test]
fn asyncboard_red_to_green() {
    red_green("asyncboard");
}
#[test]
fn asyncboard_support_runtime_and_await_are_required() {
    let t = Temp::new();
    let p = t.0.join("Async 学员");
    studio::init("asyncboard", &p).unwrap();
    assert!(p.join("src/runtime.rs").is_file());
    assert!(p.join("samples/jobs.tsv").is_file());
    fs::write(p.join("src/lib.rs"), reference("asyncboard")).unwrap();
    assert!(studio::check(&p, 3).unwrap());
    let wrong = reference("asyncboard").replace("    delay.await;", "    let _ = delay;");
    fs::write(p.join("src/lib.rs"), wrong).unwrap();
    assert!(!studio::check(&p, 3).unwrap(), "漏写 await 不应通过");
}

#[test]
fn asyncboard_first_milestone_is_reachable_without_future_milestones() {
    let t = Temp::new();
    let p = t.0.join("Async 第一步");
    studio::init("asyncboard", &p).unwrap();
    let file = p.join("src/lib.rs");
    let mut learner = fs::read_to_string(&file).unwrap();
    let reference = reference("asyncboard");
    let begin = learner.find("pub fn parse_jobs(").unwrap();
    let end = learner.find("/// 学员可以").unwrap();
    let solved_begin = reference.find("pub fn parse_jobs(").unwrap();
    let solved_end = reference.find("/// 学员可以").unwrap();
    learner.replace_range(begin..end, &reference[solved_begin..solved_end]);
    fs::write(file, learner).unwrap();
    assert!(studio::check(&p, 1).unwrap());
    assert!(!studio::check(&p, 2).unwrap());
}
