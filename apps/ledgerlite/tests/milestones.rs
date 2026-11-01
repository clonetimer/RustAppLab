use ledgerlite::*;
#[test]
fn step1_exact_decimal() {
    assert_eq!(
        parse_cents("0.1").unwrap() + parse_cents("0.2").unwrap(),
        30
    );
    assert_eq!(parse_cents("12").unwrap(), 1200);
    assert_eq!(parse_cents("001.05").unwrap(), 105);
}
#[test]
fn step1_bad_money() {
    for s in [
        "-1",
        "NaN",
        "1.234",
        "1.",
        ".1",
        "1e2",
        "",
        "18446744073709551615",
    ] {
        assert!(parse_cents(s).is_err(), "应该拒绝 {s}");
    }
}
#[test]
fn step2_records() {
    let records =
        parse_entries("\u{feff}2024-02-29\t 餐饮 \t12.50\r\n2026-10-28\t交通\t3\r\n").unwrap();
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].cents, 1250);
    assert_eq!(records[0].category, "餐饮");
    assert!(parse_entries("").unwrap().is_empty());
}
#[test]
fn step2_bad_records() {
    for s in [
        "2023-02-29\tfood\t1",
        "2026-13-01\tfood\t1",
        "2026-10-28\t \t1",
        "bad",
        "2026-10-28\tfood\t-1",
    ] {
        assert!(parse_entries(s).is_err());
    }
}
#[test]
fn step3_grouping() {
    let entries =
        parse_entries("2026-10-28\tfood\t0.10\n2026-10-28\tbus\t2\n2026-10-29\tfood\t0.20")
            .unwrap();
    let t = totals(&entries).unwrap();
    assert_eq!(t["food"], 30);
    assert_eq!(t["bus"], 200);
    assert_eq!(t.keys().next().unwrap(), "bus");
}
#[test]
fn step3_checked_sum() {
    let a = Entry {
        day: "2026-10-28".into(),
        category: "x".into(),
        cents: u64::MAX,
    };
    let mut b = a.clone();
    b.cents = 1;
    assert!(totals(&[a, b]).is_err());
    assert!(totals(&[]).unwrap().is_empty());
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
    std::process::Command::new(env!("CARGO_BIN_EXE_ledgerlite"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn step3_cli_persistent_book() {
    let t = CliTemp::new("book");
    let f = t.0.join("中文 账本.tsv");
    for v in ["0.10", "0.20"] {
        assert!(cli(&[
            "add".as_ref(),
            f.as_os_str(),
            "2026-10-28".as_ref(),
            "food".as_ref(),
            v.as_ref()
        ])
        .status
        .success());
    }
    let o = cli(&["summary".as_ref(), f.as_os_str()]);
    assert!(o.status.success());
    assert_eq!(
        String::from_utf8(o.stdout).unwrap().replace("\r\n", "\n"),
        "food\t0.30\ntotal=0.30\n"
    );
    assert_eq!(std::fs::read_to_string(&f).unwrap().lines().count(), 2);
}
#[test]
fn step3_cli_refuse_damage() {
    let t = CliTemp::new("preserve");
    let f = t.0.join("ledger.tsv");
    let raw = b"not a valid record\n";
    std::fs::write(&f, raw).unwrap();
    assert!(!cli(&[
        "add".as_ref(),
        f.as_os_str(),
        "2026-10-28".as_ref(),
        "food".as_ref(),
        "1".as_ref()
    ])
    .status
    .success());
    assert_eq!(std::fs::read(&f).unwrap(), raw);
    std::fs::write(&f, b"2026-10-28\tfood\t1.00\n").unwrap();
    let before = std::fs::read(&f).unwrap();
    assert!(!cli(&[
        "add".as_ref(),
        f.as_os_str(),
        "2023-02-29".as_ref(),
        "food".as_ref(),
        "1".as_ref()
    ])
    .status
    .success());
    assert_eq!(std::fs::read(&f).unwrap(), before);
}
