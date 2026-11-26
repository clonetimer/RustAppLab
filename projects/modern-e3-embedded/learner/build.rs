use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=memory.x");
    if env::var("TARGET").as_deref() == Ok("thumbv7m-none-eabi") {
        let out = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo OUT_DIR"));
        fs::write(out.join("memory.x"), include_bytes!("memory.x")).expect("linker memory layout");
        println!("cargo:rustc-link-search={}", out.display());
        println!("cargo:rustc-link-arg-bin=firmware=-Tlink.x");
    }
}
