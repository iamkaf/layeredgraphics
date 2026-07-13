# Foundation completion evidence

Completed 2026-07-12. This record maps the executable-document and browser-rendering foundations to checked evidence.

## Document engine

- `examples/readme-banner/build.sh` creates, mutates, validates, inspects, exports, and checksum-verifies an editable document through public `lg` commands.
- `spec/document/v1.schema.json`, `spec/commands/v1.schema.json`, and `docs/spec` define the checked contracts; CI rejects generated drift.
- Core tests cover atomic transactions, revisions, changesets, undo/redo, supported-state diffs, schema-0 migration, future-version rejection, malformed/unsafe archives, allocation limits, integrity, and atomic replacement.
- Embedded and host-resolved linked assets support relinking, content hashes, declared lengths, deduplication, author metadata, and explicit missing-asset diagnostics.
- Rust, CLI, browser WASM, and native Node execute the same operation fixture. Command and imperative styles compare equal graphical state.
- Authoritative rendering covers images, fills, bitmap text, groups, order, visibility, opacity, transforms, normal/multiply blend, nearest/smooth sampling, scale, isolated layers, PNG, JPEG, and WebP.
- Structured validation and inspection cover trees, assets, extensions, declared bounds, and opt-in pixel-visible bounds.

## Browser and performance runtime

- `/playground/` transfers an `OffscreenCanvas`; Playwright proves retained preview work does not block a main-thread animation callback.
- Rust and browser tests compare retained output with cold authoritative output after mutations and verify cache reuse, eviction, memory bounds, structured invalidation, and old/new dirty rectangles.
- WebGPU detection and direct presentation have shader validation and deterministic device-loss recreation. Canvas2D/WASM is the tested fallback.
- Four quality intents, viewport requests, one-slot coalescing, cancellation, 120 ms idle refinement, canonical-snapshot worker recovery, and observability are exposed by production APIs.
- Bounded warm batches preserve order and expose progress, diagnostics, backpressure, and cache metrics.
- Sprite, 2K, 4K, deep-document, and 32-output batch workloads have reproducible checked results and numeric budgets.

## Intentional boundaries

WebGPU composites retained top-level sources; isolated groups and authoritative/raw requests remain Rust-composited. GPU filtering, interactive resolution, and viewport scaling use documented tolerances. Tiling remains measurement-triggered because the checked 4K workload fits the 256 MiB retained-cache budget.

Masks, clipping, vector shapes, gradients, selection, painting, adjustments, filters, editor controllers, and Spriteform integration remain outside this completed foundation.
