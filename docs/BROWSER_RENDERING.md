# Browser rendering sessions

The browser runtime keeps canonical document execution and rendering coordination in a module worker. `BrowserRenderSession` is the main-thread proxy; `RenderWorkerHost` owns a `GraphicsDocument`, Rust `RetainedRenderer`, optional `PreviewPresenter`, cancellation state and idle-refinement scheduling.

```ts
import { BrowserRenderSession, moduleWorkerFactory } from "@layered-graphics/browser";

const session = await BrowserRenderSession.open(
  moduleWorkerFactory(),
  kgfxBytes,
  document.querySelector("canvas"),
);

await session.execute({ op: "layerUpdate", id: "hero", set: { x: 120 } });
const frame = await session.preview("interactive", { includePixels: false });
```

Transferring a canvas enables direct worker presentation. WebGPU is selected explicitly when an adapter and canvas context are available. It retains top-level source textures, applies transforms/sampling/opacity and ping-pong composites normal or multiply in WGSL; groups arrive as retained isolated Rust surfaces. Otherwise raw authoritative RGBA is presented with Canvas2D. A caller without a canvas receives transferable pixels. Encoded images are not part of the per-edit preview loop.

## Quality and viewport intent

- `interactive` renders at half resolution with nearest sampling, favoring gesture cadence.
- `preview` renders at document resolution for normal editing.
- `refined` renders at document resolution with smooth sampling after input settles.
- `authoritative` uses reference/export intent.

The frame reports both requested and delivered tiers. A viewport can crop and nearest-scale the transferable/direct frame. Pixel-art callers should use nearest sampling and integer viewport zoom independently of quality intent.

Known preview differences are deliberately narrow: interactive frames may be lower resolution, viewport scaling is nearest-neighbor, GPU filtering/8-bit rounding and display conversion may differ from reference output, and nested groups rasterize in Rust before GPU group composition. Coordinates, straight-alpha source-over equations, ordering, transforms and normal/multiply behavior are mirrored by a validated WGSL pipeline. [`fixtures/conformance/preview-tolerances.json`](../fixtures/conformance/preview-tolerances.json) records per-feature tolerances. Refined raw pixels are tested byte-for-byte against a cold authoritative render; callers requiring reference pixels request raw/authoritative output.

## Invalidation and retained state

Command results classify metadata, local pixels, composition, assets and global changes. Transform changes include old and new bounds. Metadata-only changes render nothing. Top-level transform, opacity and visibility changes reuse source surfaces; source, nested-group and asset changes evict dependent entries. Stack/removal and global changes deliberately invalidate broadly when that is safer.

The Rust retained renderer owns bounded source/intermediate surfaces and records cache hits, misses, evictions and bytes. A default session budget is 256 MiB. Missing or incompatible changes can always recover by clearing derived state and rebuilding from the canonical document.

## Scheduling and lifecycle

Only one not-yet-started preview remains queued; a newer preview rejects the obsolete request. Abort signals send cancellation. After a transaction, a 120 ms idle timer requests a refined frame. Worker failures reject in-flight work and reopen the last acknowledged `.kgfx` snapshot in a fresh worker. GPU device loss discards device resources and capability detection runs again on the next frame. Closing releases the document, caches, timers, worker and GPU resources.

Warm batches are processed in stable-order chunks. `maxConcurrency` is bounded to 1–8, progress is reported after each chunk, outputs are transferred between chunks for backpressure, and per-item errors do not reorder the remaining results.

## Metrics

Every preview reports total, raster and presentation time; revision and tier; dirty layers/regions and reasons; cache activity; CPU/GPU byte estimates; and cancelled/superseded counts. `session.metrics()` exposes the latest summary without enabling logs.
