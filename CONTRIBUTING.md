# Contributing

Layered Graphics welcomes bug reports, focused feature proposals, documentation improvements, fixtures, and code contributions.

## Before opening a change

- Search existing issues and discussions.
- For substantial API, document-format, rendering-semantic, or dependency changes, open an issue first so the contract can be agreed before implementation.
- Keep application-specific behavior outside the engine unless it generalizes across authoring products.

## Development setup

Install stable Rust, Node.js 24, pnpm 11.5.2, the `wasm32-unknown-unknown` target, and `wasm-bindgen-cli` 0.2.126.

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.126 --locked
corepack enable
pnpm install
```

Run the checks used by CI:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
./examples/readme-banner/build.sh
./examples/showcase/build.sh
pnpm check
pnpm test
pnpm benchmark
```

The browser suite needs Chromium; install it with:

```bash
pnpm --filter @layered-graphics/site exec playwright install chromium
```

## Change expectations

- Committed document changes pass through commands.
- New public behavior includes Rust and cross-runtime coverage where applicable.
- Graphics features specify persistence, coordinate space, compositing, history, invalidation, preview fidelity, authoritative output, and failure behavior.
- Public capability claims include a reproducible example or test.
- Performance changes update a representative workload rather than relying on anecdotes.
- Generated schemas and WASM bindings are regenerated and checked for drift.

Use clear commits and explain user-visible behavior, compatibility impact, and verification in pull requests.

By participating, you agree to follow the [Code of Conduct](CODE_OF_CONDUCT.md). Contributions are licensed under the repository's [MIT License](LICENSE).
