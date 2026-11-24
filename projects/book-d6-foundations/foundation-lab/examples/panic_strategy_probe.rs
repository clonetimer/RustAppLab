//! Compile twice with -C panic=unwind / -C panic=abort; always run as a child.
use std::path::PathBuf;

struct Sentinel(PathBuf);

impl Drop for Sentinel {
    fn drop(&mut self) {
        std::fs::write(&self.0, "guard dropped\n").expect("write the destructor sentinel");
    }
}

#[inline(never)]
fn panic_body(path: PathBuf) {
    let _sentinel = Sentinel(path);
    eprintln!("entered panic body");
    panic!("strategy teaching panic");
}

fn main() {
    let path = std::env::args_os().nth(1).expect("a fresh sentinel path");
    let result = std::panic::catch_unwind(|| panic_body(PathBuf::from(path)));
    assert!(result.is_err(), "the deliberate panic must be observed");
    eprintln!("caught unwind");
}
