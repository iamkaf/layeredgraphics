# Phase 1 and Phase 2 completion audit

Status: complete on 2026-07-12. This audit maps every exit criterion to checked-in evidence.

## Phase 1 — executable document

- Banner from clean checkout: `examples/readme-banner/build.sh` creates, mutates, validates, inspects and exports the approved engine-authored fixture using only `lg`; CI runs it.
- Versioned contracts: `spec/document/v1.schema.json`, `spec/commands/v1.schema.json`, `docs/spec/kgfx-v1.md` and `docs/spec/commands-v1.md`; a test rejects schema drift.
- Transactions/history/diff/persistence: atomic reducer tests, transaction-boundary undo/redo, supported-state diff tests, safe replacement tests, `lg diff` and retained `lg watch` integration coverage.
- Assets: content-addressed embedded deduplication, host-resolved linked references, relinking, integrity/length checks, author metadata and missing-asset diagnostics.
- Browser/Node equivalence: browser WASM and native Node execute the same checked `api-equivalence.ops.json`; each compares command and imperative graphical state. Native tests also cover history/render/serialization.
- Reference rendering: normal/multiply, ordering, groups, alpha, opacity, visibility, transform, backgrounds, scale, nearest/smooth and PNG/JPEG/WebP are covered by unit, contract, CLI and browser fixtures.
- Inspection/validation: structured tree/assets/extensions/bounds plus opt-in pixel-visible bounds; `.kgfx` and standalone operations validate through CLI/API.
- Robustness/migration: historical schema-0 fixture, future-version rejection, malformed ZIP, unsafe path, oversized declaration, per-document limits, linked integrity and previous-file preservation tests.
- One mutation path: CLI, WASM, native Node, imperative TypeScript and worker APIs all call the canonical Rust command reducer.
- Baselines: `crates/lg-core/examples/phase_baselines.rs`, checked results and documented methodology cover every requested Phase 1 category including CLI startup and peak memory.

## Phase 2 — browser preview and rendering performance

- Off-main-thread demo: `/playground/` transfers an `OffscreenCanvas` and owns document/render work in a module worker; Playwright proves a main-thread animation callback proceeds during preview.
- Retained/incremental correctness: Rust `RetainedRenderer` tests cache reuse, source eviction, memory bounds and byte-exact equality with cold output after changes. Browser tests repeat the cold comparison through WASM.
- Invalidation: structured categories, reasons, affected resources and dirty rectangles are public. Transform tests require old/new bounds; nested/group/global changes choose conservative eviction.
- GPU/fallback/recovery: explicit WebGPU detection and direct texture presentation, Canvas2D fallback, deterministic WebGPU device-loss recreation test and worker canonical-snapshot recovery path.
- Quality/cancellation/refinement: four intent tiers, viewport requests, one-slot preview coalescing, abort messages, superseded/cancelled counters and 120 ms idle refinement.
- Warm batches: bounded 1–8 chunk size, stable order, per-item diagnostics, progress/backpressure and retained encoded rendering. The checked multi-size image batch is 1.19× faster warm by median.
- Observability: frame stages, tiers, dirty reasons, cache activity, byte estimates and cancellation are returned from production APIs.
- Benchmark corpus/budgets: sprite, general 2K/4K, deep and 32-output batch categories have reproducible release measurements and machine-enforced numeric budgets.
- No encoded preview round trip: normal preview frames cross as raw transferable RGBA or present directly to the transferred canvas; PNG/WebP/JPEG encoding is reserved for export/batch requests.

## Deliberate Phase 2 boundaries

The WebGPU path composites retained top-level sources with a validated WGSL normal/multiply pipeline; isolated group sources and raw/authoritative requests remain Rust-composited. GPU filtering/rounding is explicitly tolerance-based rather than claimed byte-identical. Interactive resolution and viewport scaling are the other documented approximations. Tile rendering remains measurement-triggered: the current 4K retained corpus stays within the 256 MiB cache budget, so an always-on tiler would add complexity without meeting a demonstrated need.
