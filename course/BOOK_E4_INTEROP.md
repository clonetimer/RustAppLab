# E4 · SignalBridge：让 C 和浏览器真正调用 Rust

[现代化教学计划](MODERN_RUST_ROADMAP.md) · [学习工程](../projects/modern-e4-interop/README.md)

目标：将一套有界整数信号分析逻辑编译成 Windows/Linux 原生动态库和 WebAssembly。学习 ABI、所有权和不安全代码边界，不把 Rust 源码重写为 JavaScript。固定 Rust 1.100.0，Edition 2024；没有第三方 crate，不改变经典课程 MSRV。

| 任务 | 学员动手 | 可验证结果 |
|---|---|---|
| 1 | classify | 整数 -100000..100000；低、正常、高；阈值必须低于高 |
| 2 | summarize | 1..4096 个样本；最小、最大、和、高告警数；非法输入拒绝 |
| 3 | e4_summarize | repr(C)、extern C、调用方内存、有界切片、成功后写输出；错误保留原值 |
| 综合 | C 与浏览器调用 | 真正链接动态库；真正编译 wasm32，浏览器调用 Rust 导出 |

先复制 learner 目录，按其中 README 运行每步合同。C 端的头文件和客户端是固定材料，Rust 端输出使用定宽整数而非 Rust String、Vec、Result 或未定义布局的结构。成功返回 0，输入/长度/空或未对齐指针分别 -1/-2/-3；非空指针有效性不可由 Rust 自动检查。调用方必须保证一块连续有效可读的输入与独占可写且不重叠的输出；不在调用期间释放、修改或跨线程竞争。Rust 不接管分配，也不把 Vec 指针交给 C；不能将任意地址传进来试错。sum 的绝对上界 4096 × 100000 小于 i32::MAX。

## 运行跨语言程序

在已完成 TODO 的学员工程运行 cargo build --frozen --release。

Linux：在工程目录运行 cc -std=c11 -Wall -Wextra -Werror c/client.c -Ltarget/release -le4_signal_bridge -Wl,-rpath,'$ORIGIN' -o target/release/c-client；再运行 ./target/release/c-client。

Windows：使用 x64 Native Tools Command Prompt（MSVC），运行 cl /nologo /W4 /WX c\\client.c /Fe:target\\release\\c-client.exe /link /LIBPATH:target\\release e4_signal_bridge.dll.lib；若 import lib 名称不同，使用 target/release 中实际生成的 import lib。运行 target\\release\\c-client.exe，应出现 E4_C_ABI_PASS。

浏览器：先 rustup target add wasm32-unknown-unknown --toolchain 1.100.0，再 cargo build --frozen --release --target wasm32-unknown-unknown。将生成的 e4_signal_bridge.wasm 复制到 web 目录，通过只绑定 127.0.0.1 的静态 HTTP 服务打开页面。JS 检查可表示的整数范围，再调用实际 e4_classify；不能把超出 i32 的 JS 数值直接传入（会发生截断/回绕）。页面的计数来自每个真实分类结果。浏览器不直接访问 Rust 内存，也不暴露原生指针 API。

## 迁移实验与验收边界

默认输入 -4,10,19,20,31，阈值 10/20，应得到 1 低、2 正常、2 高；把高阈值改成 19，结果 1/1/3。再尝试反向阈值和超范围输入，解释为什么 JS 与 Rust 都要校验。C 输出仅由调用者持有，错误不修改已有结果。真实 Rust 参考/学员红绿、C 编译及执行、实际 Wasm 文件、Node 实例化、Chromium 本机 HTTP 交互及截图、迁移后的空 Cargo 缓存重放与学员包源码隔离均由候选 CI 检验。当前未完成 CI；不宣称生产级 ABI 稳定、硬件信号安全、所有外部指针安全或真实学员掌握。

参考：[Rust Nomicon FFI](https://doc.rust-lang.org/nomicon/ffi.html) · [Rust 2024 unsafe attributes](https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-attributes.html) · [WebAssembly JavaScript API](https://developer.mozilla.org/en-US/docs/WebAssembly/JavaScript_interface)。
