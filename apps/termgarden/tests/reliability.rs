use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
};
use termgarden::{apply_event, storage, Event, Garden};
static COUNT: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "term-reliability-{}-{}",
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
#[test]
fn storage_rejects_invalid_state() {
    for text in [
        "wrong",
        "TERMGARDEN\t1\n0\topen\ta\n",
        "TERMGARDEN\t1\n1\topen\ta\n1\tdone\tb\n",
        "TERMGARDEN\t1\n1\tother\ta\n",
        "TERMGARDEN\t1\n1\topen\tx\u{1b}y\n",
    ] {
        assert!(storage::decode(text).is_err(), "{text:?}");
    }
}
#[test]
fn saving_twice_preserves_ids_and_utf8() {
    let t = Temp::new();
    let path = t.0.join("my 中文.tasks");
    let mut g = Garden::default();
    apply_event(&mut g, Event::Add("书写自己的案例".into())).unwrap();
    storage::save(&path, &g).unwrap();
    apply_event(&mut g, Event::Toggle).unwrap();
    storage::save(&path, &g).unwrap();
    let reloaded = storage::load(&path).unwrap();
    assert_eq!(reloaded.tasks, g.tasks);
    assert_eq!(reloaded.next_id, 2);
    assert_eq!(fs::read_dir(&t.0).unwrap().count(), 1);
}
#[test]
fn invalid_existing_file_is_not_overwritten_by_cli() {
    let t = Temp::new();
    let p = t.0.join("private.txt");
    fs::write(&p, "private\n").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_termgarden"))
        .args(["play".as_ref(), p.as_os_str()])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert_eq!(fs::read_to_string(p).unwrap(), "private\n");
}
#[test]
fn interactive_session_and_eof_exit() {
    let t = Temp::new();
    let p = t.0.join("garden.tasks");
    let mut child = Command::new(env!("CARGO_BIN_EXE_termgarden"))
        .args(["play".as_ref(), p.as_os_str()])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all("add 真实交互\nunknown\nx\n".as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("未执行"));
    let g = storage::load(&p).unwrap();
    assert_eq!(g.tasks.len(), 1);
    assert!(g.tasks[0].done);
}
#[cfg(unix)]
#[test]
fn no_symlink_overwrite() {
    let t = Temp::new();
    let original = t.0.join("original");
    fs::write(&original, "keep").unwrap();
    let link = t.0.join("link");
    std::os::unix::fs::symlink(&original, &link).unwrap();
    assert!(storage::load(&link).is_err());
    assert!(storage::save(&link, &Garden::default()).is_err());
    assert_eq!(fs::read_to_string(original).unwrap(), "keep");
}
