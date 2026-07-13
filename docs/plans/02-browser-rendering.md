# Completed foundation: browser rendering

Completed 2026-07-12. This record preserves the architectural decisions that subsequent graphics work must extend.

## Delivered

- module-worker ownership of document execution and retained rendering
- `BrowserRenderSession` lifecycle, canonical snapshot recovery, cancellation, coalescing, and idle refinement
- `interactive`, `preview`, `refined`, and `authoritative` quality intents plus viewport requests
- retained top-level sources, structured invalidation reasons, dirty rectangles, cache/memory metrics
- WebGPU composition and direct `OffscreenCanvas` presentation with Canvas2D/WASM fallback
- byte-exact retained/cold comparisons for authoritative raw output and tolerance contracts for GPU previews
- bounded warm batch rendering with progress, stable order, diagnostics, and cache reuse
- sprite, 2K, 4K, deep-layer, and batch workloads with checked numeric budgets

## Durable constraints

- Preview approximation is explicit; authoritative CPU output defines supported semantics.
- Normal preview frames use direct presentation or transferable RGBA, never encoded-image round trips.
- Device loss and worker failure recover from canonical document state.
- Every graphics primitive declares invalidation, retained-cache cost, quality-tier behavior, and conformance tolerance.
- Tiling remains measurement-triggered; it is not added solely as an architectural preference.

## Evidence

See [`../BROWSER_RENDERING.md`](../BROWSER_RENDERING.md), [`../BENCHMARKS.md`](../BENCHMARKS.md), [`../FOUNDATION_AUDIT.md`](../FOUNDATION_AUDIT.md), and the live `/playground/` proof.
