#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$ROOT/packages/core/src/generated"

command -v wasm-bindgen >/dev/null 2>&1 || {
  printf 'wasm-bindgen is required. Install it with: cargo install wasm-bindgen-cli --version 0.2.126 --locked\n' >&2
  exit 1
}

cargo build --quiet --release --manifest-path "$ROOT/Cargo.toml" -p lg-wasm --target wasm32-unknown-unknown
mkdir -p "$OUT"
wasm-bindgen \
  --target web \
  --typescript \
  --out-dir "$OUT" \
  --out-name lg_wasm \
  "$ROOT/target/wasm32-unknown-unknown/release/lg_wasm.wasm"
