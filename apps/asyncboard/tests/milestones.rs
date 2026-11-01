use asyncboard::{
    complete, parse_jobs, render_report, run_jobs, runtime::TimerDriver, Completion, Job,
};
use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Temp(std::path::PathBuf);
impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "asyncboard-studio-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn step1_parses_owned_unicode_jobs() {
    let jobs =
        parse_jobs("# 学员测试\nfirst\t0\t你好 Rust\nnext_2\t135\t保存我的任务报告\n").unwrap();
    assert_eq!(jobs.len(), 2);
    assert_eq!(jobs[0].id, "first");
    assert_eq!(jobs[0].note, "你好 Rust");
    assert_eq!(jobs[1].delay_ms, 135);
}
#[test]
fn step1_rejects_bad_values_and_duplicates() {
    for bad in [
        "",
        "# 只有注释\n",
        "x\t-1\t任务",
        "x\t30001\t任务",
        "x\tabc\t任务",
        "x\t10\t",
        "bad/id\t0\t任务",
        "x\t1\ta\nx\t2\tb",
        "x\t0\ta\tmore",
    ] {
        assert!(parse_jobs(bad).is_err(), "必须拒绝非法清单：{bad:?}");
    }
}
#[test]
fn step2_renders_order_and_unicode() {
    let jobs = [
        Completion {
            id: "second".into(),
            delay_ms: 30,
            note: "第二步 ✓".into(),
        },
        Completion {
            id: "first".into(),
            delay_ms: 80,
            note: "第一步".into(),
        },
    ];
    assert_eq!(
        render_report(&jobs),
        "id\twait_ms\tnote\nsecond\t30\t第二步 ✓\nfirst\t80\t第一步\n"
    );
}
#[test]
fn step2_empty_report_has_only_header() {
    assert_eq!(render_report(&[]), "id\twait_ms\tnote\n");
}
#[test]
fn step3_real_await_will_not_finish_immediately() {
    let mut timer = TimerDriver::new().unwrap();
    let job = Job {
        id: "timer".into(),
        delay_ms: 90,
        note: "需要真的等待".into(),
    };
    let delay = timer.delay(Duration::from_millis(90)).unwrap();
    let now = Instant::now();
    let results =
        asyncboard::runtime::drive(vec![complete(job.clone(), delay)], Duration::from_secs(2))
            .unwrap();
    assert!(
        now.elapsed() >= Duration::from_millis(70),
        "代码应等待 Future，不能立刻返回"
    );
    assert_eq!(results[0].id, job.id);
    assert_eq!(results[0].note, job.note);
}
#[test]
fn step3_multiple_jobs_finish_in_readiness_order() {
    let jobs = parse_jobs("slow\t150\t慢\nfast\t25\t快\nmid\t70\t中").unwrap();
    let start = Instant::now();
    let done = run_jobs(jobs).unwrap();
    assert_eq!(
        done.iter().map(|j| j.id.as_str()).collect::<Vec<_>>(),
        ["fast", "mid", "slow"]
    );
    assert!(
        start.elapsed() >= Duration::from_millis(110),
        "不能立即跳过异步等待"
    );
}
#[test]
fn step3_demo_cli_saves_a_real_report() {
    let temp = Temp::new();
    let dir = temp.0.join("我的 Async 项目");
    let out = Command::new(env!("CARGO_BIN_EXE_asyncboard"))
        .args(["demo"])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let content = fs::read_to_string(dir.join("RESULTS.tsv")).unwrap();
    assert_eq!(content.lines().count(), 4);
    assert_eq!(
        content.lines().nth(1).unwrap().split('\t').next().unwrap(),
        "write"
    );
}
#[test]
fn step3_cli_preserves_existing_report() {
    let temp = Temp::new();
    let input = temp.0.join("jobs.tsv");
    let output = temp.0.join("result.tsv");
    fs::write(&input, "a\t0\t你好\n").unwrap();
    fs::write(&output, "已有重要数据").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_asyncboard"))
        .arg("run")
        .arg(input)
        .arg(&output)
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert_eq!(fs::read_to_string(output).unwrap(), "已有重要数据");
}
