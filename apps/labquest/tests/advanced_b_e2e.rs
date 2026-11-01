use labquest::advanced_b;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
static NONCE: AtomicU64 = AtomicU64::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "rustapplab-b-{}-{t}-{}",
            std::process::id(),
            NONCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        Self(root)
    }
    fn project(&self) -> PathBuf {
        self.0.join("my-taskpocket-pro")
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn initial_red_then_reference_green_and_stale() {
    let tmp = Scratch::new();
    let p = tmp.project();
    advanced_b::init(&p).unwrap();
    assert!(advanced_b::is_project(&p));
    assert!(advanced_b::init(&p).is_err());
    assert!(!advanced_b::check(&p, 1).unwrap());
    assert!(advanced_b::next(&p).unwrap().contains("B1"));
    fs::write(
        p.join("taskcore/src/lib.rs"),
        include_str!("../../../projects/advanced-b/reference/taskcore/src/lib.rs"),
    )
    .unwrap();
    assert!(advanced_b::check(&p, 3).unwrap());
    assert!(advanced_b::next(&p).unwrap().contains("3/3"));
    let mut src = fs::read_to_string(p.join("taskcore/src/lib.rs")).unwrap();
    src.push_str("\n// personal regression test\n");
    fs::write(p.join("taskcore/src/lib.rs"), src).unwrap();
    assert!(advanced_b::next(&p).unwrap().contains("过期"));
}
#[test]
fn editing_visible_student_tests_does_not_bypass_fixed_contract() {
    let tmp = Scratch::new();
    let p = tmp.project();
    advanced_b::init(&p).unwrap();
    fs::write(
        p.join("taskcore/tests/b1.rs"),
        "#[test]fn fake(){assert!(true);}\n",
    )
    .unwrap();
    assert!(!advanced_b::check(&p, 1).unwrap());
}
#[cfg(unix)]
#[test]
fn symlink_source_rejected_before_running_cargo() {
    use std::os::unix::fs::symlink;
    let tmp = Scratch::new();
    let p = tmp.project();
    advanced_b::init(&p).unwrap();
    fs::remove_file(p.join("taskcore/src/lib.rs")).unwrap();
    symlink("/etc/hosts", p.join("taskcore/src/lib.rs")).unwrap();
    assert!(advanced_b::check(&p, 1).is_err());
}

#[test]
fn stage_one_does_not_require_unfinished_business_methods() {
    let tmp = Scratch::new();
    let p = tmp.project();
    advanced_b::init(&p).unwrap();
    let refcode = include_str!("../../../projects/advanced-b/reference/taskcore/src/lib.rs");
    let begin = refcode.find("pub fn add(").unwrap();
    let end = refcode.find("pub fn load(").unwrap();
    let partial = format!("{}{}{}", &refcode[..begin],
      "pub fn add(_tasks:&mut Vec<Task>,_title:&str)->Result<u64,String>{Err(\"not implemented\".into())}\npub fn complete(_tasks:&mut [Task],_id:u64)->Result<(),String>{Err(\"not implemented\".into())}\n",
      &refcode[end..]);
    fs::write(p.join("taskcore/src/lib.rs"), partial).unwrap();
    assert!(advanced_b::check(&p, 1).unwrap());
    assert!(!advanced_b::check(&p, 2).unwrap());
}
