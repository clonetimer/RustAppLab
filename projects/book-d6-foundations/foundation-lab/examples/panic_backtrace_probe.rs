//! Both modes deliberately fail; use a fresh process for each backtrace setting.
#[inline(never)]
fn probe_inner(index_mode: bool) {
    if index_mode {
        let values = [7_u32];
        let index = std::hint::black_box(values.len());
        eprintln!("{}", values[index]);
    } else {
        panic!("RustAppLab backtrace probe: locate this expression in your own source");
    }
}

#[inline(never)]
fn probe_middle(index_mode: bool) {
    probe_inner(index_mode);
}

#[inline(never)]
fn probe_outer(index_mode: bool) {
    probe_middle(index_mode);
}

fn main() {
    let index_mode = std::env::args().nth(1).as_deref() == Some("index");
    probe_outer(index_mode);
}
