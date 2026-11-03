use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
fn root() -> std::path::PathBuf {
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    for _ in 0..100 {
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let p = std::env::temp_dir().join(format!("taskpocket-pro-b3-{}-{t}-{n}", std::process::id()));
        match fs::create_dir(&p) {
            Ok(()) => return p,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("cannot create isolated test directory: {error}"),
        }
    }
    panic!("could not allocate a unique B3 test directory")
}
fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_taskpocket-pro"))
        .args(args)
        .output()
        .unwrap()
}
#[test]
fn actual_cli_and_report() {
    let tmp = root();
    let db = tmp.join("我的 任务.tsv");
    let p = db.to_str().unwrap();
    let r = run(&["add", p, "学 Rust"]);
    assert!(
        r.status.success(),
        "{:?}",
        String::from_utf8_lossy(&r.stderr)
    );
    assert_eq!(r.stdout, b"added=1\n");
    assert!(run(&["add", p, "write tests"]).status.success());
    assert!(run(&["done", p, "1"]).status.success());
    let listed = run(&["list", p]);
    assert!(listed.status.success());
    assert_eq!(
        String::from_utf8(listed.stdout).unwrap(),
        "1\tDONE\t学 Rust\n2\tOPEN\twrite tests\n"
    );
    let out = run(&["stats", p]);
    assert_eq!(out.stdout, b"total=2 open=1 done=1\n");
    let tasks = taskcore::load(&db).unwrap();
    assert!(taskcore::markdown(&tasks).contains("学 Rust"));
    assert!(fs::read_to_string(&db).unwrap().contains("1\tDONE"));
    let _ = fs::remove_dir_all(tmp);
}
#[test]
fn corrupt_file_must_not_be_overwritten() {
    let dir = root();
    let db = dir.join("corrupt.tsv");
    fs::write(&db, b"bad\tstate\trow\n").unwrap();
    let before = fs::read(&db).unwrap();
    let out = run(&["add", db.to_str().unwrap(), "New"]);
    assert!(!out.status.success());
    assert_eq!(fs::read(&db).unwrap(), before);
    let _ = fs::remove_dir_all(dir);
}
#[test]
fn invalid_data_does_not_write() {
    let dir = root();
    let db = dir.join("tasks.tsv");
    let p = db.to_str().unwrap();
    assert!(!run(&["add", p, " "]).status.success());
    assert!(!db.exists());
    assert!(!run(&["done", p, "wrong"]).status.success());
    assert!(!db.exists());
    let _ = fs::remove_dir_all(dir);
}
