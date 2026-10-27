#!/usr/bin/env bash
set -euo pipefail
# An optional local test adapter; this script does not change the platform checkout.
platform_dir=$(realpath "${1:?Pass the Wasm-Cloud-Platform checkout}")
repo_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
probe_dir="$repo_dir/target/wcp-media-probe"
mkdir -p "$probe_dir/src"
cp "$repo_dir/scripts/wcp-media-probe.rs" "$probe_dir/src/main.rs"
cp "$platform_dir/Cargo.lock" "$probe_dir/Cargo.lock"
cat > "$probe_dir/Cargo.toml" <<EOF
[package]
name = "djmx-wcp-media-probe"
version = "0.1.0"
edition = "2024"
[workspace]
[dependencies]
runtime = { path = "$platform_dir/crates/runtime" }
common = { path = "$platform_dir/crates/common" }
tokio = { version = "1", features = ["rt-multi-thread", "macros", "signal"] }
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
EOF
cd "$platform_dir"
# Respect the platform's own pinned toolchain and Linux target directory.
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/wasm-cloud-platform-target}"
cargo build --manifest-path "$probe_dir/Cargo.toml"
printf '%s\n' "$CARGO_TARGET_DIR/debug/djmx-wcp-media-probe"
