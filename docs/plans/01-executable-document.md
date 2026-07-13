# Completed foundation: executable documents

Completed 2026-07-12. This record replaces the original implementation checklist; current behavior is normative in the schemas, specifications, tests, and public guides.

## Delivered

- `.kgfx` v1 container, generated document/command schemas, schema-0 migration, embedded and linked assets
- canonical commands, atomic transactions, revisions, changesets, undo/redo, supported-state diffing
- image, fill, bitmap-text, and group layers with stable IDs, transforms, opacity, visibility, normal/multiply blend
- authoritative PNG/JPEG/WebP output with nearest/smooth sampling and scale/layer options
- Rust core, `lg` CLI, browser WASM SDK, native Node SDK, structured validation and inspection
- hostile-container coverage, allocation limits, checksums, safe paths, and atomic file replacement
- engine-authored README banner and cross-runtime operation equivalence

## Durable constraints

- Commands are the only committed mutation path.
- The portable document is canonical; render caches and editor gestures are derived state.
- Unknown namespaced extensions survive load/save.
- Unsupported schema versions and missing capabilities fail explicitly.
- New layer features must integrate with persistence, history, inspection, every supported runtime, and authoritative output.

## Evidence

See [`../FOUNDATION_AUDIT.md`](../FOUNDATION_AUDIT.md), [`../spec/kgfx-v1.md`](../spec/kgfx-v1.md), [`../spec/commands-v1.md`](../spec/commands-v1.md), and `crates/lg-core/tests/contracts.rs`.
