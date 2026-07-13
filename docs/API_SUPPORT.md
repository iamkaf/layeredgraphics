# Runtime and package support

Status: Phase 1/2 experimental prerelease contract.

## Packages

| Package | Runtime | Purpose |
| --- | --- | --- |
| `@layered-graphics/core` | Modern browsers | Rust/WASM document, command and authoritative rendering API; command and imperative authoring styles |
| `@layered-graphics/browser` | Modern browsers with module workers | Worker-owned retained sessions, quality tiers, WebGPU/Canvas2D presentation, cancellation, metrics and batches |
| `@layered-graphics/node` | Node.js | Native N-API binding to the same Rust document and renderer |
| `lg` | Native executable | File-oriented authoring, inspection, validation, diff, watch and export |

The prerelease support floor is the current and previous Node.js LTS lines (Node 22 and 24) and the latest two stable releases of Chromium, Firefox and Safari. The worker path requires module workers, WebAssembly, transferable `ArrayBuffer`s and `OffscreenCanvas` for direct presentation. Browsers without WebGPU use the authoritative WASM renderer and Canvas2D presentation. Callers can request raw RGBA when they do not transfer a canvas.

Initial native Node release targets are x64 Linux GNU, x64 Windows and x64 macOS. CI builds and executes each host-native package. Arm64 and musl packages remain candidates until dedicated target runners exercise the complete fixture suite; they are not silently advertised from cross-compilation alone.

## Stability

The document schema, command schema and packages are versioned but experimental. Schema version 1 can migrate the checked-in historical schema-0 fixture. A future schema is rejected rather than guessed. No stable-v1 compatibility promise applies until the release gate in the roadmap is complete.

All document mutations use the Rust reducer. The TypeScript imperative helpers emit the same public command protocol; they do not own a parallel document model. Worker and native bindings exchange command JSON with that reducer.

## Error and resource behavior

Transactions are atomic. Command failures identify the zero-based command index; successful results contain `fromRevision`, `revision`, affected layers/assets, invalidation records and a warnings array. CLI runtime failures exit 1, command-line usage failures exit 2, and machine-readable success output remains on stdout while diagnostics use stderr.

Current hard limits are a 32,768-pixel canvas dimension, 268,435,456 canvas pixels, 10,000 layers, 128 levels of nesting, 10,000 assets, 16 MiB manifest JSON, 512 MiB per embedded asset and 1 GiB total embedded payload. These are safety ceilings, not recommended interactive sizes.
