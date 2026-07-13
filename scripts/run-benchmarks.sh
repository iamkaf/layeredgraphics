#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cargo build --quiet --release --manifest-path "$ROOT/Cargo.toml" -p layered-graphics-cli
LG_CLI="$ROOT/target/release/lg" cargo run --quiet --release --manifest-path "$ROOT/Cargo.toml" -p layered-graphics --example phase_baselines -- "$@"
