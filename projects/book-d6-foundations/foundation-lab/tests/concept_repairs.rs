//! Execute the teaching failures, not just the Markdown labels describing them.
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "rustapplab-concept-{label}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).expect("new isolated experiment directory");
        Self(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn rustc() -> Command {
    Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()))
}

fn library_artifact() -> PathBuf {
    let executable = std::env::current_exe().expect("integration-test location");
    let deps = executable.parent().unwrap();
    fs::read_dir(deps)
        .expect("compiled dependency directory")
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("libbook_d6_foundation_lab-")
                && path.extension().is_some_and(|ext| ext == "rlib")
        })
        .max_by_key(|path| fs::metadata(path).unwrap().modified().unwrap())
        .expect("Cargo has built the teaching library")
}

#[test]
fn compile_fail_examples_report_the_intended_rust_error() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/concept_repairs.rs");
    assert!(source.is_file(), "the 31 repair examples must exist");
    let text = fs::read_to_string(source).unwrap();
    let scratch = Scratch::new("diagnostics");
    let library = library_artifact();
    let mut concept = "";
    let mut blocks = Vec::new();
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        if let Some(id) = line.strip_prefix("//! ## ") {
            concept = id;
        }
        if let Some(code) = line.strip_prefix("//! ```compile_fail,") {
            assert!(
                code.starts_with('E') && code.len() == 5,
                "{concept}: {code}"
            );
            let mut body = String::new();
            for line in lines.by_ref() {
                if line == "//! ```" {
                    break;
                }
                body.push_str(line.strip_prefix("//! ").unwrap_or(""));
                body.push('\n');
            }
            blocks.push((concept, code, body));
        }
    }
    assert!(
        blocks.len() >= 20,
        "the repair must include actual rejected programs"
    );
    for (index, (concept, code, body)) in blocks.iter().enumerate() {
        let file = scratch.0.join(format!("failure_{index}.rs"));
        fs::write(&file, format!("fn main() {{\n{body}\n}}\n")).unwrap();
        let output = rustc()
            .arg(&file)
            .args([
                "--edition=2021",
                "--crate-name",
                "concept_failure",
                "--error-format=json",
                "--emit=metadata",
            ])
            .arg("--extern")
            .arg(format!("book_d6_foundation_lab={}", library.display()))
            .arg("-L")
            .arg(format!(
                "dependency={}",
                library.parent().unwrap().display()
            ))
            .arg("-o")
            .arg(scratch.0.join("failure.rmeta"))
            .output()
            .expect("run the actual Rust compiler");
        let diagnostic = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success(), "{concept} unexpectedly compiled");
        assert!(
            diagnostic.contains(&format!("\"code\":\"{code}\"")),
            "{concept} failed for the wrong reason; expected {code}: {diagnostic}"
        );
        println!("{concept}: rejected with {code}");
    }
}

fn compile_probe(scratch: &Path, name: &str, strategy: &str) -> PathBuf {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(format!("{name}.rs"));
    assert!(
        source.is_file(),
        "missing subprocess probe: {}",
        source.display()
    );
    let executable = scratch.join(format!("{name}-{strategy}{}", std::env::consts::EXE_SUFFIX));
    let output = rustc()
        .arg(&source)
        .args([
            "--edition=2021",
            "-C",
            "opt-level=0",
            "-C",
            "debuginfo=2",
            "-C",
        ])
        .arg(format!("panic={strategy}"))
        .arg("-o")
        .arg(&executable)
        .output()
        .expect("compile the independent process");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    executable
}

#[test]
fn unwind_runs_destructors_and_abort_does_not() {
    let scratch = Scratch::new("strategies");
    for strategy in ["unwind", "abort"] {
        let executable = compile_probe(&scratch.0, "panic_strategy_probe", strategy);
        let sentinel = scratch.0.join(format!("{strategy}-dropped.txt"));
        let output = Command::new(executable)
            .arg(&sentinel)
            .env("RUST_BACKTRACE", "0")
            .current_dir(&scratch.0)
            .output()
            .expect("run isolated panic strategy");
        let diagnostic = String::from_utf8_lossy(&output.stderr);
        assert!(diagnostic.contains("entered panic body"), "{diagnostic}");
        assert!(
            diagnostic.contains("strategy teaching panic"),
            "{diagnostic}"
        );
        if strategy == "unwind" {
            assert!(
                output.status.success(),
                "catch_unwind must recover this unwind"
            );
            assert_eq!(fs::read_to_string(&sentinel).unwrap(), "guard dropped\n");
            assert!(diagnostic.contains("caught unwind"));
        } else {
            assert!(
                !output.status.success(),
                "panic=abort must terminate the child"
            );
            assert!(
                !sentinel.exists(),
                "abort must not run this Rust destructor"
            );
            assert!(!diagnostic.contains("caught unwind"));
        }
        println!(
            "strategy={strategy}; status={}; destructor={}",
            output.status,
            sentinel.exists()
        );
    }
}

fn run_backtrace(executable: &Path, backtrace: &str, mode: &str) -> Output {
    Command::new(executable)
        .arg(mode)
        .env("RUST_BACKTRACE", backtrace)
        .output()
        .expect("run backtrace child")
}

#[test]
fn backtrace_switch_exposes_the_own_source_call_chain() {
    let scratch = Scratch::new("backtrace");
    let executable = compile_probe(&scratch.0, "panic_backtrace_probe", "unwind");
    let off = run_backtrace(&executable, "0", "explicit");
    let on = run_backtrace(&executable, "1", "explicit");
    for output in [&off, &on] {
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("panic_backtrace_probe.rs:"));
    }
    let off_text = String::from_utf8_lossy(&off.stderr);
    let on_text = String::from_utf8_lossy(&on.stderr);
    assert!(!off_text.contains("stack backtrace:"));
    assert!(on_text.contains("stack backtrace:"), "{on_text}");
    for frame in ["probe_inner", "probe_middle", "probe_outer"] {
        assert!(
            on_text.contains(frame),
            "missing own frame {frame}: {on_text}"
        );
    }
    assert!(on_text.find("probe_inner").unwrap() < on_text.find("probe_middle").unwrap());
    assert!(on_text.find("probe_middle").unwrap() < on_text.find("probe_outer").unwrap());
    println!("RUST_BACKTRACE=0: {}", off_text.trim());
    println!("RUST_BACKTRACE=1: {}", on_text.trim());
}

#[test]
fn explicit_and_index_panics_both_locate_own_source() {
    let scratch = Scratch::new("locations");
    let executable = compile_probe(&scratch.0, "panic_backtrace_probe", "unwind");
    for (mode, message) in [
        ("explicit", "RustAppLab backtrace probe"),
        ("index", "index out of bounds"),
    ] {
        let output = run_backtrace(&executable, "0", mode);
        let diagnostic = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success());
        assert!(diagnostic.contains(message), "{diagnostic}");
        assert!(
            diagnostic.contains("panic_backtrace_probe.rs:"),
            "{diagnostic}"
        );
        println!("mode={mode}; {}", diagnostic.trim());
    }
}

#[test]
fn missing_file_recovery_does_not_swallow_other_io_errors() {
    use book_d6_foundation_lab::concept_repairs::optional_note;
    use std::io::{self, ErrorKind};
    let scratch = Scratch::new("io-kind");
    let file = scratch.0.join("optional-note.txt");
    let missing = fs::read_to_string(&file);
    assert_eq!(missing.as_ref().unwrap_err().kind(), ErrorKind::NotFound);
    assert_eq!(optional_note(missing).unwrap(), "");
    fs::write(&file, "Rust").unwrap();
    assert_eq!(optional_note(fs::read_to_string(&file)).unwrap(), "Rust");
    // Permissions differ on administrator/root accounts. Inject only this error
    // value to exercise the policy; do not claim an OS-level permission failure.
    let denied = optional_note(Err(io::Error::from(ErrorKind::PermissionDenied)));
    assert_eq!(denied.unwrap_err().kind(), ErrorKind::PermissionDenied);
}
