use textscope::*;
#[test]
fn step1_unicode() {
    assert_eq!(
        stats("Rust 学习\n"),
        Stats {
            bytes: 12,
            chars: 8,
            lines: 1,
            words: 2
        }
    );
}
#[test]
fn step1_empty_and_crlf() {
    assert_eq!(stats(""), Stats::default());
    assert_eq!(stats("a\r\nb\n").lines, 2);
    assert_eq!(stats("中文连续，Rust!").words, 2);
}
#[test]
fn step2_rank_and_tie() {
    assert_eq!(
        top_words("Rust rust! apple Banana banana", 3),
        vec![
            ("banana".into(), 2),
            ("rust".into(), 2),
            ("apple".into(), 1)
        ]
    );
}
#[test]
fn step2_limits_and_unicode() {
    assert!(top_words("x", 0).is_empty());
    assert_eq!(
        top_words("学习 学习 RUST", 9),
        vec![("学习".into(), 2), ("rust".into(), 1)]
    );
}
#[test]
fn step3_borrowed_lines() {
    let data = "one\nRust 学习\nmore Rust";
    let lines = search(data, "Rust");
    assert_eq!(lines, vec![(2, "Rust 学习"), (3, "more Rust")]);
    assert!(
        std::ptr::eq(lines[0].1.as_ptr(), data[4..].as_ptr()),
        "直接借用原文本，不能泄漏新分配字符串"
    );
}
#[test]
fn step3_edge_cases() {
    assert!(search("abc", "").is_empty());
    assert!(search("Rust", "rust").is_empty());
    assert_eq!(search("一\r\n二", "二"), vec![(2, "二")]);
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
    std::process::Command::new(env!("CARGO_BIN_EXE_textscope"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn step3_cli_unicode_file() {
    let t = CliTemp::new("text");
    let f = t.0.join("中文 file.txt");
    std::fs::write(&f, "Rust 学习\nRust helps\n").unwrap();
    let o = cli(&["find".as_ref(), f.as_os_str(), "Rust".as_ref()]);
    assert!(o.status.success());
    assert_eq!(
        String::from_utf8(o.stdout).unwrap().replace("\r\n", "\n"),
        "1:Rust 学习\n2:Rust helps\n"
    );
}
#[test]
fn step3_cli_bad_input() {
    let t = CliTemp::new("bad-text");
    let f = t.0.join("bad.txt");
    std::fs::write(&f, [0xff]).unwrap();
    assert!(!cli(&["stats".as_ref(), f.as_os_str()]).status.success());
    assert!(!cli(&["find".as_ref()]).status.success());
}
