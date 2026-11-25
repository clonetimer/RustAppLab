# 覆盖率引导 Fuzzing（Linux）

需已完成 D11-A 帧处理项目；在个人副本下复制本目录的 fuzz 子目录，使 fuzz/Cargo.toml 的 path = ".." 指向自己的 d11-quality-lab。安装固定 nightly 和 cargo-fuzz 0.13.2，需 Linux x86_64、C++ 工具链。首次依赖准备：

    cargo +1.100.0 install cargo-fuzz --version 0.13.2 --locked
    cargo +nightly-2026-11-17 generate-lockfile --manifest-path fuzz/Cargo.toml
    cargo +nightly-2026-11-17 fetch --locked --manifest-path fuzz/Cargo.toml

保存生成的 Cargo.lock；随后在网络离线的受控环境执行：

    cargo +nightly-2026-11-17 fuzz run frame -- -runs=4096 -seed=20261125 -max_len=128

观察 libFuzzer 实际 coverage、执行次数与 corpus；没有崩溃不意味着穷尽输入。找到反例后把实际 artifact 文件路径交给 cargo fuzz run frame <artifact> 重放。只在自己的临时副本修改代码，恢复后再次重放。保留原输入文件、工具版本、锁文件与日志。不要测试公网服务或未知第三方程序。

此课首次准备会生成依赖锁；CI 保存实际锁及其哈希并检查 fuzz 过程不改变锁。后续可用该锁固定完整依赖图，初始模板仅固定直接依赖。
