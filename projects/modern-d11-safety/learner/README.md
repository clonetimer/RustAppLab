# 我的安全接口练习

复制整个 learner 目录到个人目录，Rust 1.100.0。修改 src/lib.rs 两个 TODO，分别运行：

    cargo test --locked --offline --test contracts step1_
    cargo test --locked --offline --test contracts step2_

完成后运行 cargo test --locked --offline。用普通 safe slice/array 方法也可通过公开合同；若练习 unsafe，请写清每条 SAFETY 不变量，不能仅用“测试没崩溃”证明正确。比较安全替代实现，记录输入、边界及解释。Ubuntu 可选 Miri，安装固定 nightly 与 miri/rust-src 组件后运行 cargo +nightly-2026-11-17 miri test --locked --offline。首次 miri setup 可能需联网。

不要把 miri-practice 中的故意 UB 程序作为原生 executable 运行。后续 libFuzzer 与 RustSec 参见完整课程 course/BOOK_D11_SAFETY.md。
