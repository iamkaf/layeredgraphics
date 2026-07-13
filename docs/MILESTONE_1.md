# Milestone 1: The Engine Draws Its Own Banner

Status: complete.

Historical note: Phases 1 and 2 now supersede the deferred boundaries recorded by this first vertical milestone. Current evidence is maintained in [Phase 1/2 completion audit](PHASES_1_2_AUDIT.md).

This file maps each milestone requirement to executable evidence in the repository.

## Workspace

- The root `Cargo.toml` defines the Rust workspace containing `lg-core`, `lg-cli`, and `lg-wasm`.
- `pnpm-workspace.yaml` defines the TypeScript core package and Astro/Starlight site.
- `pnpm check` checks both language workspaces.
- `pnpm test` runs Rust contracts, CLI end-to-end tests, TypeScript generation, the site build, and browser smoke tests.

## Canonical document and command engine

- `crates/lg-core/src/document.rs` defines versioned documents, assets, layers, transforms, validation, and extension preservation.
- `crates/lg-core/src/command.rs` executes command arrays atomically and returns revisioned changesets.
- `crates/lg-core/tests/contracts.rs` proves rejected transactions do not mutate state and `.kgfx` round trips preserve extension and asset data.
- `docs/spec/commands-v1.md` records the implemented experimental command contract.

## Portable `.kgfx` container

- `crates/lg-core/src/container.rs` reads and safely replaces ZIP-compatible `.kgfx` archives.
- Embedded payloads are content-addressed, length-checked, SHA-256 verified, and protected by path and entry-size checks.
- `docs/spec/kgfx-v1.md` records the implemented experimental format.

## Native CLI

`crates/lg-cli/src/main.rs` implements:

- `lg new`
- `lg exec`
- `lg layer add|update|rm|ls|move`
- `lg asset add|ls|rm`
- `lg render`
- `lg inspect`
- `lg validate`

`crates/lg-cli/tests/cli.rs` invokes every command family against real temporary `.kgfx`, asset, operation, and PNG files. It also checks `--json`, inspection paths, operation validation, nested groups, updates, ordering, and output dimensions.

## Graphics and rendering

`crates/lg-core/src/render.rs` provides authoritative PNG rendering for:

- Embedded PNG image layers
- RGBA fill layers
- Embedded TrueType/OpenType and reproducible `LGF1` bitmap-font text layers
- Isolated nested groups
- Position, scale/flip, opacity, visibility, and stack ordering
- Normal and multiply compositing
- Whole-document and single-layer rendering at an explicit output scale

Unit tests cover source-over opacity and multiply behavior. The banner fixture combines every milestone layer kind and property family.

## Engine-authored banner

`examples/readme-banner/build.sh`:

1. Builds the native CLI.
2. Creates and renders an icon composition using public layer commands.
3. Creates a 1200×630 banner document.
4. Embeds the generated image and checked-in font.
5. Builds nested layers using individual commands and `lg exec`.
6. Updates and moves layers.
7. Validates and inspects the document.
8. Renders authoritative PNG output.
9. Verifies `examples/readme-banner/banner.sha256`.
10. Publishes the `.kgfx` and PNG to the site.

The generated banner appears in the repository README and on the site landing page.

## Browser/WASM proof

- `scripts/build-wasm.sh` builds the same Rust core for `wasm32-unknown-unknown` and generates web bindings.
- `packages/core` exposes initialization, validation, inspection, and rendering through a strict TypeScript package.
- `/playground/` fetches the same published `.kgfx` banner, validates it, inspects it, renders it through WASM, and presents the result without Canvas compositing or a server rendering API.
- `apps/site/tests/browser-smoke.spec.ts` runs the proof in Chromium and verifies dimensions and the exact authoritative PNG SHA-256.

## Landing and documentation site

- `apps/site` is a static Astro/Starlight application.
- The custom landing page uses the engine-generated banner.
- Starlight provides getting-started, CLI, concepts, roadmap, and technology pages plus Pagefind search and sitemap output.
- Browser tests verify the landing page and live WASM proof.

## Boundaries deferred at milestone completion

At milestone completion the code did not yet implement WebGPU, incremental preview, undo/redo, document diffing, JPEG/WebP output or batch thumbnails. Phases 1 and 2 subsequently implemented those foundations. Painting, selections, masks, clipping, adjustment layers, advanced filters and Spriteform integration remain later-phase work.

## Reproduction

```bash
./examples/readme-banner/build.sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
pnpm check
pnpm test
```
