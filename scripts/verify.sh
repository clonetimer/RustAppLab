#!/usr/bin/env sh
set -eu
cd "$(dirname "$0")/.."
command -v cargo >/dev/null 2>&1 || { echo '未安装 Rust/Cargo。请先安装 rustup。' >&2; exit 127; }
echo '[1/4] cargo fmt (check)'
cargo fmt --all -- --check
echo '[2/4] cargo clippy (all packages)'
cargo clippy --locked --offline --workspace --all-targets -- -D warnings
echo '[3/4] cargo test (all packages, including smoke tests)'
cargo test --locked --offline --workspace --all-targets -- --test-threads=2
echo '[extra] All workspace doctests'
cargo test --locked --offline --workspace --doc
echo '[extra] Release full workspace tests'
cargo test --locked --offline --workspace --all-targets --release -- --test-threads=2
echo '[4/4] cargo build --release --workspace'
cargo build --locked --offline --workspace --release
echo '[extra] Advanced B independent reference workspace'
advanced_b_manifest=projects/advanced-b/reference/Cargo.toml
cargo fmt --manifest-path "$advanced_b_manifest" --all -- --check
cargo clippy --locked --offline --manifest-path "$advanced_b_manifest" --workspace --all-targets -- -D warnings
cargo clippy --locked --offline --manifest-path "$advanced_b_manifest" --workspace --all-targets --all-features -- -D warnings
cargo test --locked --offline --manifest-path "$advanced_b_manifest" --workspace --all-targets
cargo test --locked --offline --manifest-path "$advanced_b_manifest" --workspace --all-targets --all-features
cargo test --locked --offline --manifest-path "$advanced_b_manifest" --workspace --all-targets --release
cargo test --locked --offline --manifest-path "$advanced_b_manifest" --workspace --all-targets --release --all-features
cargo test --locked --offline --manifest-path "$advanced_b_manifest" --workspace --all-targets --no-default-features
cargo test --locked --offline --manifest-path "$advanced_b_manifest" --workspace --doc
cargo test --locked --offline --manifest-path "$advanced_b_manifest" --workspace --all-features --doc
printf '%s\n' '所有包的格式、Lint、测试、release 构建命令均已执行；真实用户/跨平台验收还需单独记录。'
