use batchmeter::*;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
static COUNT: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "batchmeter-test-{}-{}",
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
fn step1_real_file() {
    let t = Temp::new();
    let p = t.0.join("中文 路径.txt");
    fs::write(&p, "一\r\n二\n").unwrap();
    assert_eq!(
        analyze(&p).unwrap(),
        FileStats {
            path: p,
            bytes: 9,
            lines: 2
        }
    );
}
#[test]
fn step1_errors() {
    let t = Temp::new();
    assert!(analyze(&t.0.join("missing")).is_err());
    let p = t.0.join("bad");
    fs::write(&p, [0xff]).unwrap();
    assert!(analyze(&p).is_err());
}
#[test]
fn step2_totals() {
    let rows = [
        FileStats {
            path: PathBuf::from("a"),
            bytes: 4,
            lines: 2,
        },
        FileStats {
            path: PathBuf::from("b"),
            bytes: 8,
            lines: 1,
        },
    ];
    assert_eq!(
        aggregate(&rows).unwrap(),
        Totals {
            files: 2,
            bytes: 12,
            lines: 3
        }
    );
    assert_eq!(aggregate(&[]).unwrap(), Totals::default());
}
#[test]
fn step2_overflow() {
    let r = FileStats {
        path: PathBuf::from("x"),
        bytes: u64::MAX,
        lines: 1,
    };
    assert!(aggregate(&[r.clone(), r]).is_err());
}
#[test]
fn step3_parallel_matches_serial() {
    let t = Temp::new();
    let mut paths = Vec::new();
    for i in 0..17 {
        let p = t.0.join(format!("{i}.txt"));
        fs::write(&p, format!("第 {i} 行\n重复\n")).unwrap();
        paths.push(p);
    }
    let expected = analyze_many(&paths, 1).unwrap();
    for n in [2, 4, 16] {
        assert_eq!(analyze_many(&paths, n).unwrap(), expected);
    }
    assert_eq!(expected.len(), 17);
    assert_eq!(expected[0].path, paths[0]);
}
#[test]
fn step3_empty_missing_workers() {
    assert!(analyze_many(&[], 0).is_err());
    assert!(analyze_many(&[], 17).is_err());
    assert!(analyze_many(&[], 2).unwrap().is_empty());
    let t = Temp::new();
    assert!(analyze_many(&[t.0.join("missing")], 2).is_err());
}

// 最后一个里程碑还执行真正的 CLI；不能只让库测试通过却破坏软件入口。
struct CliTemp(std::path::PathBuf);
impl CliTemp {
    fn new(label: &str) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("applab-cli-{}-{now}-{label}", std::process::id()));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for CliTemp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn cli(args: &[&std::ffi::OsStr]) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_batchmeter"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn step3_cli_same_order() {
    let t = CliTemp::new("batch");
    let a = t.0.join("中文 a.txt");
    let b = t.0.join("b.txt");
    std::fs::write(&a, "x\ny\n").unwrap();
    std::fs::write(&b, "z\n").unwrap();
    let one = cli(&["scan".as_ref(), "1".as_ref(), a.as_os_str(), b.as_os_str()]);
    let many = cli(&["scan".as_ref(), "4".as_ref(), a.as_os_str(), b.as_os_str()]);
    assert!(one.status.success());
    assert!(many.status.success());
    assert_eq!(one.stdout, many.stdout);
    assert!(String::from_utf8(many.stdout)
        .unwrap()
        .contains("files=2 bytes=6 lines=3"));
}
#[test]
fn step3_cli_missing_is_error() {
    let t = CliTemp::new("missing");
    let a = t.0.join("missing");
    assert!(!cli(&["scan".as_ref(), "2".as_ref(), a.as_os_str()])
        .status
        .success());
    assert!(!cli(&["scan".as_ref(), "0".as_ref(), a.as_os_str()])
        .status
        .success());
}
