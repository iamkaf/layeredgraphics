# Phase 2: Browser Preview and Rendering Performance

## Objective

Make Layered Graphics feel immediate in browser applications while preserving an authoritative export path. Introduce retained render sessions, incremental invalidation, GPU previews, worker execution, and measurable performance budgets across sprite, 4K, deep-layer, and batch workloads.

The primary preview implementation is WebGPU, with the authoritative Rust renderer compiled to WASM as the functional browser fallback. A separate WebGL renderer is deferred unless measured browser-support needs justify it.

## User outcomes

At the end of this phase:

- Editing a document does not block the browser's main thread.
- A caller receives a preview quickly after a small layer change without rerendering unrelated work.
- Applications can trade detail for latency during interaction and request a refined preview afterward.
- Scripts can render many related outputs while reusing decoded assets and intermediate results.
- Pixel-art previews remain crisp and coordinate-correct.
- Export output remains governed by the reference rendering semantics.

## Rendering contract

Layered Graphics has two rendering roles:

### Preview renderer

The preview path prioritizes responsiveness. It may:

- Render at a reduced resolution during interaction
- Use GPU-native approximations for expensive operations
- Delay refinement until input settles
- Skip invisible or sub-pixel detail at coarse quality tiers

It must not silently change document state. Any known semantic or fidelity difference from reference output must be classified and testable.

### Authoritative renderer

The authoritative path prioritizes documented compositing behavior and final quality. It is used for exports, golden tests, and workflows requiring predictable output.

Preview and authoritative rendering share document interpretation and invalidation concepts even if their execution backends differ.

## Scope

### Retained render sessions

A render session is a derived, disposable view of a document revision. It maintains:

- Resolved and decoded assets
- Compiled layer and effect state
- Cached intermediate surfaces
- Spatial and dependency information
- Preview viewport and quality settings
- Revision and invalidation tracking

Sessions consume changesets produced by the document engine. They must also recover safely by rebuilding from canonical state if changes are missed or incompatible.

### Incremental invalidation

Classify changes by their rendering impact:

- Metadata-only changes that do not affect pixels
- Local pixel changes with a bounded dirty region
- Transform changes affecting old and new bounds
- Stack, blend, group, or visibility changes affecting downstream composition
- Asset changes affecting every referencing layer
- Canvas or global setting changes requiring broad invalidation

Correctness comes before minimal invalidation. Instrumentation must reveal why a layer or region rerendered so performance work is diagnosable.

### Browser worker model

The standard browser integration places document execution and rendering coordination off the main thread. The public session API covers:

- Opening and closing a document
- Executing transactions
- Subscribing to revisions, diagnostics, and preview readiness
- Setting viewport and quality intent
- Requesting preview frames, regions, or exported bytes
- Cancelling obsolete work
- Recovering from worker failure

Input coalescing and cancellation prevent stale previews from monopolizing resources during rapid interaction.

### GPU preview

Implement a GPU-backed preview for the phase-1 layer set and expand it alongside Phase 3 primitives.

Requirements:

- Explicit capability detection
- A supported fallback when the preferred GPU API is unavailable
- Consistent coordinate, alpha, layer-order, and sampling behavior
- Resource lifetime and memory-budget management
- Context/device-loss recovery
- Readback only when required by the caller

The preview surface should be directly presentable by browser applications rather than forcing every frame through encoded image data.

### Quality tiers

Define intent-based tiers rather than exposing backend-specific switches:

- **Interactive:** lowest latency while a gesture is active
- **Preview:** balanced default for normal editing
- **Refined:** higher-detail idle preview
- **Authoritative:** export/reference semantics

Applications may set a preference, while the engine reports the tier actually delivered. Pixel-art documents can constrain scaling and sampling independently of tier.

### Large-document strategy

Add tiling when whole-surface allocation or recomposition is demonstrably wasteful. The strategy must account for:

- Effects that expand sampling bounds
- Group and adjustment dependencies
- Transformed layer coverage
- Tile seams and filtering margins
- Viewport-driven prioritization

Tiling is a means to meet measured budgets, not a requirement to tile every document.

### Batch rendering

Expose batch primitives that reuse a warm render session across related revisions or parameter sets. Support:

- Bounded concurrency
- Cancellation and progress
- Stable output ordering
- Per-item diagnostics
- Cache accounting
- Backpressure for encoded outputs

The thumbnail fixture should exercise repeated asset reuse, small document changes, and multiple output sizes.

### Observability

Development and optional diagnostic builds report:

- Frame and export timing by stage
- Cache hits, misses, and evictions
- Dirty layers and regions with invalidation reasons
- GPU and CPU memory estimates
- Decode and encode time
- Work cancelled or superseded

Production APIs expose summarized metrics without requiring logging or developer tooling.

## Deliverables

- Worker-backed browser session API
- Retained render-session abstraction
- Incremental invalidation and dependency tracking
- GPU preview renderer for the supported phase-1 model
- Quality-tier policy and refinement scheduling
- Capability detection and fallback behavior
- Warm batch-render API
- Render diagnostics and benchmark runner
- Interactive preview demo without prescribed editor chrome
- Published baseline results for the benchmark corpus
- Site-hosted browser preview example and benchmark methodology/results pages

## Benchmark corpus

Maintain at least four categories:

### Sprite workload

- Small nearest-neighbor canvas
- Many reusable assets
- Frequent visibility and variant-like changes
- Large number of thumbnail outputs

### General graphics workload

- 2K and 4K canvases
- Mixed-size translucent layers
- Transforms and blend modes
- Preview zoomed both in and out

### Deep document workload

- Hundreds of layers and nested groups
- Changes near the top and bottom of the stack
- Visibility, ordering, and opacity mutations

### Batch workload

- Shared assets and mostly shared document structure
- Multiple sizes and formats
- Enough outputs to exercise backpressure and eviction

Benchmarks record median and tail latency, throughput, peak memory, and cache behavior. Hardware and browser details accompany published results.

## Initial performance budgets

Exact numeric budgets will be fixed after the Phase 1 baselines and an implementation spike. The release criteria should nevertheless enforce these experiential goals:

- A local transform or property gesture can sustain an interactive preview cadence on representative hardware.
- Obsolete input does not create an ever-growing render queue.
- Idle refinement completes without blocking further edits.
- Batch memory stays bounded as output count grows.
- Repeated renders with shared inputs measurably outperform cold full renders.
- The main browser thread remains available for application input and layout.

Numeric budgets belong in benchmark configuration once measured, not only in narrative documentation.

## Work sequence

1. Instrument the Phase 1 renderer and establish benchmark baselines.
2. Specify render sessions, revisions, invalidation categories, and cancellation.
3. Build the worker protocol and non-GPU retained session.
4. Add GPU capability detection and phase-1 layer rendering.
5. Implement direct preview presentation and viewport control.
6. Add quality tiers, coalescing, and idle refinement.
7. Optimize dirty-region behavior based on traces.
8. Add memory budgets, eviction, and device-loss recovery.
9. Implement warm batch rendering and thumbnail fixtures.
10. Publish benchmark results and documented preview differences.

## Testing strategy

### Conformance

- Compare preview output with authoritative fixtures using per-feature tolerances.
- Require exact structural and coordinate behavior even when pixel tolerances differ.
- Track every accepted preview deviation by feature and quality tier.

### Invalidation

- Mutate one property at a time and assert required dependencies rerender.
- Compare incremental output against a cold full render after randomized command sequences.
- Test old and new bounds for movement and effect expansion.

### Lifecycle

- Exercise cancellation, rapid revisions, worker restart, GPU loss, memory pressure, and document close.
- Verify resources are released after sessions and assets become unreachable.

### Performance

- Run reproducible browser and Node benchmarks in continuous integration where stable.
- Separate correctness failures from noisy performance alerts.
- Require an explicit review for material benchmark regressions.

## Exit criteria

Phase 2 is complete when:

- The browser demo performs document work outside the main thread.
- Incremental output matches cold authoritative output within documented tolerances.
- All benchmark categories have recorded budgets and reproducible results.
- Interactive edits supersede stale work and refine after settling.
- GPU fallback and device-loss recovery are tested.
- Thumbnail batches demonstrate bounded memory and warm-cache improvement.
- Spriteform-scale preview fixtures no longer require encoded-image round trips per edit.

## Deferred

- Full Phase 3 graphics primitives
- Ready-made editor interactions
- Application-specific Spriteform data modeling
- Claims of identical GPU and authoritative pixels
