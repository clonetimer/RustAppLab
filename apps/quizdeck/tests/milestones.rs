use quizdeck::*;
#[test]
fn step1_comparison() {
    assert!(is_correct("cargo", " Cargo "));
    assert!(is_correct("北京", "北京"));
    assert!(!is_correct("北京", "上海"));
}
#[test]
fn step1_not_substring() {
    assert!(!is_correct("cargo", "cargo build"));
    assert!(!is_correct("5", ""));
    assert!(!is_correct("é", "É"));
}
#[test]
fn step2_bom_crlf_and_owned_data() {
    let cards = {
        let source = String::from("\u{feff} 工具 \t cargo \r\n\r\n首都\t北京\r\n");
        parse_deck(&source).unwrap()
    };
    assert_eq!(cards.len(), 2);
    assert_eq!(
        cards[0],
        Card {
            question: "工具".into(),
            answer: "cargo".into()
        }
    );
}
#[test]
fn step2_invalid() {
    for data in ["", "q", "q\t", "q\ta\textra"] {
        assert!(parse_deck(data).is_err(), "应该拒绝 {data:?}");
    }
}
#[test]
fn step3_score_and_missed() {
    let cards = parse_deck("工具\tcargo\n2+3\t5\n").unwrap();
    let result = grade(&cards, &[" Cargo ".into(), "4".into()]).unwrap();
    assert_eq!(
        result,
        Score {
            total: 2,
            correct: 1,
            missed: vec!["2+3".into()]
        }
    );
}
#[test]
fn step3_count_mismatch() {
    let cards = parse_deck("Q\tA").unwrap();
    assert!(grade(&cards, &[]).is_err());
    assert_eq!(grade(&cards, &["a".into()]).unwrap().correct, 1);
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
    std::process::Command::new(env!("CARGO_BIN_EXE_quizdeck"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn step3_cli_real_deck() {
    let t = CliTemp::new("deck");
    let d = t.0.join("题库 空格.tsv");
    let a = t.0.join("answers.txt");
    std::fs::write(&d, "首都\t北京\r\n工具\tcargo\r\n").unwrap();
    std::fs::write(&a, "北京\n Cargo \n").unwrap();
    let o = cli(&["check".as_ref(), d.as_os_str(), a.as_os_str()]);
    assert!(o.status.success());
    assert_eq!(String::from_utf8(o.stdout).unwrap().trim(), "score=2/2");
}
#[test]
fn step3_cli_interactive_and_bad_deck() {
    use std::io::Write;
    let t = CliTemp::new("interactive");
    let d = t.0.join("deck.tsv");
    std::fs::write(&d, "工具\tcargo\n").unwrap();
    let mut c = std::process::Command::new(env!("CARGO_BIN_EXE_quizdeck"))
        .arg("play")
        .arg(&d)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    c.stdin.take().unwrap().write_all(b"cargo\n").unwrap();
    let o = c.wait_with_output().unwrap();
    assert!(o.status.success());
    assert!(String::from_utf8(o.stdout).unwrap().contains("score=1/1"));
    std::fs::write(&d, "bad line").unwrap();
    assert!(!cli(&["play".as_ref(), d.as_os_str()]).status.success());
}
