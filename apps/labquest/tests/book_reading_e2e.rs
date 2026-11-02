//! Reading is navigation, never an execution or progress operation.
use std::{
    collections::BTreeMap,
    ffi::OsStr,
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
        let root = std::env::temp_dir().join(format!("reading-{}-{stamp}", std::process::id()));
        fs::create_dir(&root).unwrap();
        Self(root)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn call(args: &[&OsStr]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_labquest"))
        .args(args)
        .output()
        .unwrap()
}
fn reading(root: &Path) -> Output {
    call(&["learn".as_ref(), "reading".as_ref(), root.as_os_str()])
}
fn text(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}
fn snapshot(root: &Path) -> BTreeMap<PathBuf, Option<Vec<u8>>> {
    fn walk(base: &Path, dir: &Path, entries: &mut BTreeMap<PathBuf, Option<Vec<u8>>>) {
        for item in fs::read_dir(dir).unwrap() {
            let path = item.unwrap().path();
            let key = path.strip_prefix(base).unwrap().to_path_buf();
            if path.is_dir() {
                entries.insert(key, None);
                walk(base, &path, entries);
            } else {
                entries.insert(key, Some(fs::read(path).unwrap()));
            }
        }
    }
    let mut entries = BTreeMap::new();
    walk(root, root, &mut entries);
    entries
}
fn assert_readonly(root: &Path, title: &str, concept: &str, check: &str) {
    let before = snapshot(root);
    let out = reading(root);
    let body = text(&out);
    assert!(out.status.success(), "{body}");
    for expected in [
        title,
        concept,
        "https://rust.bootcss.com/",
        "https://doc.rust-lang.org/",
        "自己的输入",
        check,
        "1.37",
        "2018",
        "2021",
        "1.74",
    ] {
        assert!(body.contains(expected), "missing {expected}: {body}");
    }
    assert_eq!(
        before,
        snapshot(root),
        "reading must not modify files or create reports/target"
    );
}
#[test]
fn reading_overview_explains_versions_and_navigation() {
    let out = call(&["learn".as_ref(), "reading".as_ref()]);
    let body = text(&out);
    assert!(out.status.success(), "{body}");
    for term in [
        "对照阅读",
        "FirstCard",
        "TaskPocket",
        "PageMint",
        "进阶",
        "1.37",
        "2018",
        "2021",
        "1.74",
        "Cargo.toml",
        "不自动迁移",
    ] {
        assert!(body.contains(term), "{term}: {body}");
    }
}
#[test]
fn firstcard_and_all_studio_mappings_are_readonly() {
    let t = Temp::new();
    let first = t.0.join("我的 第一张卡");
    labquest::beginner::init(&first).unwrap();
    assert_readonly(&first, "FirstCard", "Cargo", "learn check");
    for (id, concept) in [
        ("unitlab", "Result"),
        ("quizdeck", "借用"),
        ("textscope", "字符串"),
        ("ledgerlite", "Result"),
        ("badgeforge", "trait"),
        ("termgarden", "枚举"),
        ("batchmeter", "线程"),
        ("minikv", "借用"),
        ("netfetch", "Result"),
        ("servicedesk", "测试"),
        ("storedesk", "Result"),
        ("asyncboard", "async"),
    ] {
        let root = t.0.join(format!("中文 空格 {id}"));
        labquest::studio::init(id, &root).unwrap();
        fs::create_dir(root.join(".studio-reports")).unwrap();
        fs::write(root.join(".studio-reports/keep.md"), "历史记录 & <保留>").unwrap();
        // Invalid Rust and a build script demonstrate that navigation must not compile.
        fs::write(root.join("src/lib.rs"), "this is deliberately not Rust").unwrap();
        fs::write(
            root.join("build.rs"),
            "compile_error!(\"must never execute\");",
        )
        .unwrap();
        assert_readonly(&root, id, concept, "learn check");
    }
}
#[test]
fn independent_and_advanced_projects_get_their_own_reading() {
    let t = Temp::new();
    for id in ["taskpocket", "pagemint"] {
        let root = t.0.join(format!("独立 {id}"));
        let out = call(&[
            "app".as_ref(),
            "init".as_ref(),
            id.as_ref(),
            root.as_os_str(),
        ]);
        assert!(out.status.success(), "{}", text(&out));
        assert_readonly(
            &root,
            if id == "taskpocket" {
                "TaskPocket"
            } else {
                "PageMint"
            },
            "测试",
            "learn check",
        );
        assert!(text(&call(&[
            "learn".as_ref(),
            "lesson".as_ref(),
            root.as_os_str()
        ]))
        .contains("learn reading"));
    }
    for (command, title, concept) in [
        ("advanced", "进阶 A", "RefCell"),
        ("advanced-b", "进阶 B", "工作空间"),
    ] {
        let root = t.0.join(format!("进阶 工程 {command}"));
        let out = call(&[command.as_ref(), "start".as_ref(), root.as_os_str()]);
        assert!(out.status.success(), "{}", text(&out));
        assert_readonly(&root, title, concept, &format!("{command} check"));
        assert!(text(&call(&[
            "learn".as_ref(),
            "lesson".as_ref(),
            root.as_os_str()
        ]))
        .contains("learn reading"));
    }
}
#[test]
fn unknown_missing_and_forged_projects_are_rejected_without_writes() {
    let t = Temp::new();
    for root in [t.0.clone(), t.0.join("不存在")] {
        let before = snapshot(&t.0);
        let out = reading(&root);
        assert!(!out.status.success());
        assert!(text(&out).contains("目录") || text(&out).contains("工程"));
        assert_eq!(before, snapshot(&t.0));
    }
    let root = t.0.join("伪造");
    fs::create_dir(&root).unwrap();
    fs::write(
        root.join(".applab-studio"),
        "RustAppLab Studio v1\nunknown\n",
    )
    .unwrap();
    assert!(!reading(&root).status.success());
}
#[test]
fn generated_cards_share_precise_links_and_escape_html() {
    let t = Temp::new();
    let root = t.0.join("借用 & 实验");
    labquest::studio::init("minikv", &root).unwrap();
    let md = fs::read_to_string(root.join("TASK_CARDS.md")).unwrap();
    let html = fs::read_to_string(root.join("TASK_CARDS.html")).unwrap();
    let out = reading(&root);
    assert!(out.status.success(), "{}", text(&out));
    let cn = "https://rust.bootcss.com/ch04-02-references-and-borrowing.html";
    let official = "https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html";
    for body in [&md, &html, &text(&out)] {
        for expected in ["对照阅读", cn, official, "1.37", "1.74", "Cargo.toml"] {
            assert!(body.contains(expected), "{expected}: {body}");
        }
    }
    assert!(md.contains(&format!("]({cn})")));
    assert!(html.contains(&format!("href=\"{cn}\"")));
    assert!(html.contains("引用 &amp; 借用"));
    assert!(!html.contains("引用 & 借用"));
    assert!(html.find("对照阅读").unwrap() < html.find("</main>").unwrap());
    let before = snapshot(&root);
    assert!(labquest::studio::init("minikv", &root).is_err());
    assert_eq!(before, snapshot(&root));
    let lesson = call(&["learn".as_ref(), "lesson".as_ref(), root.as_os_str()]);
    assert!(lesson.status.success(), "{}", text(&lesson));
    assert!(text(&lesson).contains("learn reading"));
}
