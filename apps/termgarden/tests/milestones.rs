use termgarden::*;

use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};
static COUNT: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let p = std::env::temp_dir().join(format!(
            "termgarden-test-{}-{stamp}-{}",
            std::process::id(),
            COUNT.fetch_add(1, Ordering::Relaxed)
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
fn cli(args: &[&std::ffi::OsStr]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_termgarden"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn step1_commands_and_unicode() {
    assert_eq!(
        parse_command("add 学完 Rust 所有权").unwrap(),
        Event::Add("学完 Rust 所有权".into())
    );
    for (s, e) in [
        (" n ", Event::Next),
        ("prev", Event::Prev),
        ("x", Event::Toggle),
        ("filter", Event::Filter),
        ("quit", Event::Quit),
    ] {
        assert_eq!(parse_command(s).unwrap(), e);
    }
}
#[test]
fn step1_bad_inputs() {
    for s in [
        "",
        "add",
        "add   ",
        "next extra",
        "delete all",
        "add a\tb",
        "add a\u{1b}[2J",
    ] {
        assert!(parse_command(s).is_err(), "{s:?}");
    }
    assert!(parse_command(&format!("add {}", "好".repeat(81))).is_err());
}
#[test]
fn step2_state_navigation_filter_and_undo() {
    let mut g = Garden::default();
    apply_event(&mut g, Event::Next).unwrap();
    assert_eq!(g.selected, None);
    apply_event(&mut g, Event::Add("第一步".into())).unwrap();
    apply_event(&mut g, Event::Add("第二步".into())).unwrap();
    apply_event(&mut g, Event::Prev).unwrap();
    assert_eq!(g.selected, Some(1));
    apply_event(&mut g, Event::Toggle).unwrap();
    assert!(g.tasks[0].done);
    apply_event(&mut g, Event::Filter).unwrap();
    assert_eq!(g.selected, Some(2));
    apply_event(&mut g, Event::Toggle).unwrap();
    assert_eq!(g.selected, None);
    apply_event(&mut g, Event::Filter).unwrap();
    assert_eq!(g.selected, Some(1));
    apply_event(&mut g, Event::Toggle).unwrap();
    assert!(!g.tasks[0].done);
    apply_event(&mut g, Event::Prev).unwrap();
    assert_eq!(g.selected, Some(2));
    assert!(!apply_event(&mut g, Event::Quit).unwrap());
}
#[test]
fn step2_errors_do_not_mutate() {
    let mut g = Garden::default();
    let before = g.clone();
    assert!(apply_event(&mut g, Event::Toggle).is_err());
    assert_eq!(g, before);
    assert!(apply_event(&mut g, Event::Add(" ".into())).is_err());
    assert_eq!(g, before);
    g.next_id = u64::MAX;
    let before = g.clone();
    assert!(apply_event(&mut g, Event::Add("still safe".into())).is_err());
    assert_eq!(g, before);
}
#[test]
fn step3_render_is_pure_and_input_sensitive() {
    let g = Garden {
        tasks: vec![
            Task {
                id: 8,
                title: "代码评审".into(),
                done: true,
            },
            Task {
                id: 9,
                title: "自己的项目".into(),
                done: false,
            },
        ],
        selected: Some(9),
        only_open: false,
        next_id: 10,
    };
    let before = g.clone();
    let out = render(&g);
    assert!(out.contains("done=1/2 growth=sprout view=all"));
    assert!(out.contains("  [x] #8 代码评审"));
    assert!(out.contains("> [ ] #9 自己的项目"));
    assert_eq!(g, before);
    let filtered = Garden {
        only_open: true,
        ..g
    };
    let out = render(&filtered);
    assert!(!out.contains("代码评审"));
    assert!(out.contains("view=open"));
}
#[test]
fn step3_empty_and_bloom_views() {
    assert!(render(&Garden::default()).contains("(empty)"));
    let mut g = Garden::default();
    for i in 0..3 {
        apply_event(&mut g, Event::Add(format!("作品 {i}"))).unwrap();
        apply_event(&mut g, Event::Toggle).unwrap();
    }
    assert!(render(&g).contains("growth=bloom"));
    assert_eq!(renderer("board").unwrap().draw(&g), render(&g));
    assert!(renderer("compact")
        .unwrap()
        .draw(&g)
        .contains("3/3 completed"));
    assert!(renderer("unknown").is_err());
}
#[test]
fn step3_cli_replay_persists() {
    let t = Temp::new();
    let p = t.0.join("我的 花园.tasks");
    let script = t.0.join("commands.txt");
    fs::write(&script, "add 写单元测试\nadd 完成 CLI\np\nx\nq\n").unwrap();
    let output = cli(&["replay".as_ref(), p.as_os_str(), script.as_os_str()]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let saved = storage::load(&p).unwrap();
    assert_eq!(saved.tasks.len(), 2);
    assert!(saved.tasks[0].done);
    assert!(String::from_utf8_lossy(&output.stdout).contains("done=1/2"));
}
#[test]
fn step3_cli_bad_script_never_saves_partial_state() {
    let t = Temp::new();
    let p = t.0.join("garden.tasks");
    let script = t.0.join("bad.txt");
    fs::write(&script, "add should not persist\nnot-a-command\n").unwrap();
    assert!(
        !cli(&["replay".as_ref(), p.as_os_str(), script.as_os_str()])
            .status
            .success()
    );
    assert!(!p.exists());
}
