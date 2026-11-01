use badgeforge::{svg::Element, *};
#[test]
fn step1_escape() {
    assert_eq!(
        escape_xml("A&B <tag> \"x\" 'y'"),
        "A&amp;B &lt;tag&gt; &quot;x&quot; &apos;y&apos;"
    );
}
#[test]
fn step1_keep_unicode() {
    assert_eq!(escape_xml("作品卡"), "作品卡");
    assert_eq!(escape_xml("&lt;"), "&amp;lt;");
    assert!(escape_xml("").is_empty());
}
#[test]
fn step2_trait_render() {
    let b = Badge {
        title: "Rust".into(),
        subtitle: "我的作品".into(),
    };
    let s = b.render();
    assert!(s.contains("<text"));
    assert!(s.contains("Rust"));
    assert!(s.contains("我的作品"));
}
#[test]
fn step2_no_text_injection() {
    let b = Badge {
        title: "<script>".into(),
        subtitle: "A&B".into(),
    };
    let s = b.render();
    assert!(!s.contains("<script>"));
    assert!(s.contains("&lt;script&gt;"));
    assert!(s.contains("A&amp;B"));
}
#[test]
fn step3_generic_composition() {
    struct Dot;
    impl Element for Dot {
        fn render(&self) -> String {
            "<circle r=\"5\"/>".into()
        }
    }
    let s = document(&Dot);
    assert!(s.contains("http://www.w3.org/2000/svg"));
    assert!(s.contains("<circle r=\"5\"/>"));
    assert!(s.trim_end().ends_with("</svg>"));
}
#[test]
fn step3_independent_documents() {
    let a = document(&Badge {
        title: "甲".into(),
        subtitle: "一".into(),
    });
    let b = document(&Badge {
        title: "乙".into(),
        subtitle: "二".into(),
    });
    assert!(a.contains("甲"));
    assert!(!b.contains("甲"));
    assert_ne!(a, b);
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
    std::process::Command::new(env!("CARGO_BIN_EXE_badgeforge"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn step3_cli_svg_and_no_overwrite() {
    let t = CliTemp::new("svg");
    let f = t.0.join("我的 作品.svg");
    let args = [
        "make".as_ref(),
        "A&B <Rust>".as_ref(),
        "学习".as_ref(),
        f.as_os_str(),
    ];
    assert!(cli(&args).status.success());
    let before = std::fs::read_to_string(&f).unwrap();
    assert!(before.contains("A&amp;B &lt;Rust&gt;"));
    assert!(!cli(&args).status.success());
    assert_eq!(std::fs::read_to_string(&f).unwrap(), before);
}
#[test]
fn step3_cli_reject_bad_title() {
    let t = CliTemp::new("bad-svg");
    let f = t.0.join("bad.svg");
    assert!(
        !cli(&["make".as_ref(), "".as_ref(), "x".as_ref(), f.as_os_str()])
            .status
            .success()
    );
    assert!(!f.exists());
}
