# E4 SignalBridge 学员工程

复制本目录到自己的新目录。安装固定 Rust 1.100.0 与 wasm32-unknown-unknown 目标。只修改 src/lib.rs 的三个 TODO，固定测试与 C/Web 文件不改。

- 步骤 1：cargo test --frozen --test contracts step1_
- 步骤 2：cargo test --frozen --test contracts step2_
- 步骤 3：cargo test --frozen --test contracts step3_
- 最后：cargo test --frozen --all-targets；cargo build --frozen --release
- WASM：cargo build --frozen --release --target wasm32-unknown-unknown
- 把 target/wasm32-unknown-unknown/release/e4_signal_bridge.wasm 复制到 web/，从 web/ 使用本地静态 HTTP 服务（例如已安装 Python 时 python -m http.server 8000 --bind 127.0.0.1），浏览器打开 http://127.0.0.1:8000/。
- C 语言客户端见课程中的编译命令。

请记录 3 个正常/异常输入的预测、实际结果和解释，再把高阈值改为 19 比较输出。不要把参考答案当自己的练习。
