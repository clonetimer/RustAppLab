# D6 · 开发工具、Edition 迁移与 Nightly 实操

本指南将附录 D、E、G 转化为可复现操作。所有可能改写源码的实验都在临时副本中进行，不修改 RustAppLab 根 Workspace、17 个学员工程或当前 RC 的 Edition/MSRV。

## 0. 版本边界

- 正式工程基线：Edition 2021；基础课程与 D1–D6 参考实验最低 Rust 1.74。
- Edition 2024 迁移实验需要 Rust 1.85.0 或更新工具链；不代表项目整体升级 MSRV。
- Nightly 仅在临时目录实验，不进入根 Workspace、默认 CI 或发布包。

## 1. 附录 D：实际使用工具

~~~powershell
rustup component add rustfmt clippy rust-docs rust-analyzer --toolchain stable
rustc --version
cargo --version
rustfmt --version
cargo clippy --version
rust-analyzer --version

$manifest = "projects/book-d6-appendices/Cargo.toml"
cargo fmt --manifest-path $manifest --all -- --check
cargo clippy --locked --offline --manifest-path $manifest --workspace --all-targets -- -D warnings
cargo test --locked --offline --manifest-path $manifest --workspace --all-targets
cargo test --locked --offline --manifest-path $manifest --workspace --doc
cargo test --locked --offline --manifest-path $manifest --workspace --all-targets --release
cargo doc --locked --offline --manifest-path $manifest --workspace --no-deps
~~~

在 IDE 打开 projects/book-d6-appendices，确认 rust-analyzer 能识别 crate、跳转到 Task 定义并显示诊断。人为引入一个未使用变量，观察 Clippy 诊断，修复后重跑。保存工具版本与每条命令退出码；IDE 诊断不能代替 Cargo 检查。

## 2. 附录 E：临时副本迁移 2021 → 2024

严禁直接在正式仓库执行 Edition 迁移。PowerShell：

~~~powershell
$source = (Resolve-Path "projects/book-d6-appendices").Path
$lab = Join-Path $env:TEMP "RustAppLab-D6-Edition-2024"
if (Test-Path $lab) { Remove-Item $lab -Recurse -Force }
Copy-Item $source $lab -Recurse
rustup toolchain install 1.85.0 --profile minimal
$manifest = Join-Path $lab "appendix-lab/Cargo.toml"
cargo +1.85.0 fix --edition --manifest-path $manifest --allow-no-vcs
$content = [IO.File]::ReadAllText($manifest)
$content = $content.Replace('edition = "2021"', 'edition = "2024"')
[IO.File]::WriteAllText($manifest, $content, [Text.UTF8Encoding]::new($false))
cargo +1.85.0 fmt --manifest-path $manifest --all -- --check
cargo +1.85.0 clippy --locked --offline --manifest-path $manifest --workspace --all-targets -- -D warnings
cargo +1.85.0 test --locked --offline --manifest-path $manifest --workspace --all-targets
cargo +1.85.0 test --locked --offline --manifest-path $manifest --workspace --doc
cargo +1.85.0 test --locked --offline --manifest-path $manifest --workspace --all-targets --release
~~~

POSIX shell 可使用：

~~~sh
src="$(pwd)/projects/book-d6-appendices"
lab="${TMPDIR:-/tmp}/RustAppLab-D6-Edition-2024"
rm -rf "$lab"; cp -R "$src" "$lab"
rustup toolchain install 1.85.0 --profile minimal
manifest="$lab/appendix-lab/Cargo.toml"
cargo +1.85.0 fix --edition --manifest-path "$manifest" --allow-no-vcs
sed -i.bak 's/edition = "2021"/edition = "2024"/' "$manifest"
cargo +1.85.0 test --locked --offline --manifest-path "$manifest" --workspace --all-targets
cargo +1.85.0 test --locked --offline --manifest-path "$manifest" --workspace --doc
~~~

保存迁移前后 Cargo.toml、diff、rustc 版本和测试结果。解释 Edition 2024 验证为何不能把正式工程 MSRV 从 1.74 自动改成 1.85。失败时区分语法迁移、依赖/MSRV 和临时路径问题，不得修改正式工程绕过。

## 3. 附录 G：隔离 Nightly

使用日期化工具链，避免滚动 nightly 造成不可复现。以下实验在 TEMP 中创建独立 crate，并使用尚未稳定的 generic_const_exprs：

~~~powershell
$nightly = Join-Path $env:TEMP "RustAppLab-D6-Nightly"
if (Test-Path $nightly) { Remove-Item $nightly -Recurse -Force }
New-Item -ItemType Directory -Force (Join-Path $nightly "src") | Out-Null
@'
[package]
name = "nightly-isolation-lab"
version = "0.1.0"
edition = "2021"
publish = false
'@ | Set-Content -Encoding utf8 (Join-Path $nightly "Cargo.toml")
@'
#![feature(generic_const_exprs)]
#![allow(incomplete_features)]
pub struct Square<const N: usize> where [(); N * N]: { cells: [u8; N * N] }
impl<const N: usize> Square<N> where [(); N * N]: {
    pub fn zeroed() -> Self { Self { cells: [0; N * N] } }
    pub fn cells(&self) -> &[u8] { &self.cells }
}
#[cfg(test)]
mod tests {
    use super::Square;
    #[test]
    fn const_expression_sizes_storage() {
        assert_eq!(Square::<3>::zeroed().cells().len(), 9);
    }
}
'@ | Set-Content -Encoding utf8 (Join-Path $nightly "src/lib.rs")
rustup toolchain install nightly-2026-11-17 --profile minimal
cargo +nightly-2026-11-17 test --manifest-path (Join-Path $nightly "Cargo.toml")
~~~

若当日 Nightly 不可用，记录原因和尝试日期，不得改用 Stable 冒充通过。完成后删除临时目录。

**隔离验收**：根 Cargo.toml、Cargo.lock、rust-toolchain.toml 不应变化；feature gate 只能留在临时 crate；Nightly 结果不计入 Stable、Rust 1.74 MSRV 或发布证据。

## 4. 个人完成记录

| 项目 | 必须保留的证据 |
|---|---|
| D · 工具链 | rustfmt、Clippy、rust-analyzer、rustdoc 版本与实际运行结果 |
| E · Edition | 临时副本迁移前后 Cargo.toml、diff、Rust 1.85+ 测试结果与解释 |
| G · Nightly | 日期化工具链版本、feature-gated 实验结果、隔离检查 |
| 反思 | Edition 与 MSRV 的区别；为什么 Nightly 不进入默认工程 |

本指南不自动读取本机工具链、生成学习成绩或修改覆盖矩阵。教师/学习者应根据真实提交的证据更新个人掌握状态。
