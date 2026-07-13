#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
mkdir -p "$ROOT/spec/document" "$ROOT/spec/commands"
cargo run --quiet --manifest-path "$ROOT/Cargo.toml" -p lg-cli -- schema document -o "$ROOT/spec/document/v1.schema.json"
cargo run --quiet --manifest-path "$ROOT/Cargo.toml" -p lg-cli -- schema commands -o "$ROOT/spec/commands/v1.schema.json"
