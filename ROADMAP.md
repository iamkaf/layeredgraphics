# Layered Graphics Roadmap

This roadmap describes the path from an executable document format to a production-ready embedded authoring engine. It is outcome-oriented: each phase must complete useful vertical workflows, not only isolated subsystems.

Dates are intentionally omitted until the implementation team and technical spike results establish realistic throughput.

Implementation follows the repository-wide choices in [Technology Stack](docs/TECH_STACK.md): Rust and WebAssembly for the engine, TypeScript for public browser-facing APIs and editor behavior, WebGPU for previews, and a single Cargo/pnpm monorepo.

## Outcomes that guide every phase

Three reference workflows remain active throughout development:

1. **README banner** — create, inspect, revise, and export a polished layered graphic through commands and the CLI.
2. **Thumbnail batch** — generate many related images from a script with predictable memory use and strong cache reuse.
3. **Spriteform composite** — render and interactively edit sprite compositions while Spriteform retains ownership of variants and generation rules.

Each phase expands what these fixtures can demonstrate. Regressions in any previously supported fixture block a release.

## Phase 1: Executable document

Status: complete (2026-07-12).

Establish the product contract: `.kgfx`, assets, commands, history, inspection, a reference renderer, and the initial CLI.

Primary outcomes:

- A versioned, self-contained document can be created, validated, saved, reopened, and migrated.
- All mutations pass through commands and atomic transactions.
- The same operations are available through command and imperative JavaScript APIs.
- Basic layered compositions render to PNG, JPEG, and WebP.
- Agents and scripts can inspect documents through stable structured output.
- A simple layered banner can be authored without a graphical editor.

Detailed plan: [Phase 1 — Executable Document](docs/plans/01-executable-document.md)

## Phase 2: Browser preview and rendering performance

Status: complete (2026-07-12).

Add the retained, incremental rendering architecture needed for interactive browser applications and high-throughput batch jobs.

Primary outcomes:

- Documents run in a browser worker without blocking the application UI.
- GPU previews update incrementally and support explicit quality tiers.
- The export renderer remains authoritative when preview fidelity differs.
- Render invalidation and cache behavior are observable and testable.
- Representative pixel-art, 4K, deep-layer, and batch workloads have performance budgets.

Detailed plan: [Phase 2 — Browser Rendering](docs/plans/02-browser-rendering.md)

## Phase 3: Photoshop-like graphics primitives

Build the editing and compositing vocabulary required for real graphics authoring.

Primary outcomes:

- Masks, clipping, groups, blend modes, filters, adjustments, text, shapes, and gradients follow documented Photoshop-like semantics.
- Selections constrain compatible editing operations.
- Painting and explicit destructive raster operations are available.
- A polished banner remains fully editable after creation.
- Compatibility behavior is covered by visual conformance fixtures.

Detailed plan: [Phase 3 — Graphics Primitives](docs/plans/03-graphics-primitives.md)

## Phase 4: Headless editor toolkit

Provide application-independent interaction behavior without prescribing an editor interface.

Primary outcomes:

- Applications can compose viewport, selection, transform, painting, snapping, keyboard, clipboard, and history controllers.
- The toolkit is framework-neutral at its core.
- Optional bindings demonstrate integration without becoming the product architecture.
- A developer can build a useful browser editor in an afternoon.

Detailed plan: [Phase 4 — Editor Toolkit](docs/plans/04-editor-toolkit.md)

## Phase 5: Spriteform integration and production hardening

Validate the engine inside a real authoring product, complete batch workflows, and stabilize the public release contract.

Primary outcomes:

- Spriteform uses Layered Graphics for composition previews and exports.
- Spriteform-specific smart layers and variants remain outside the core document semantics.
- Batch thumbnail rendering demonstrates bounded concurrency and cache reuse.
- The public API, extension policy, compatibility policy, and migration guarantees are documented.
- The project is ready for an initial stable FOSS release.

Detailed plan: [Phase 5 — Integration and Hardening](docs/plans/05-integration-hardening.md)

## Cross-cutting work

The following are not deferred to a single phase:

### Landing and documentation site

The public Astro/Starlight site develops alongside the engine. Phase 1 establishes the landing page, documentation information architecture, and executable command examples. Later phases add live rendering examples, generated API reference, editor-toolkit tutorials, benchmarks, compatibility notes, and versioned release documentation.

Detailed plan: [Website and Documentation](docs/plans/06-website-documentation.md)

### Compatibility

Photoshop-like behavior is specified with small reference fixtures. Where exact compatibility is impossible or undesirable, the difference is explicit rather than accidental.

### Performance

Every feature includes representative performance coverage. Optimizations must preserve authoritative output or be restricted to documented preview modes.

### Security and robustness

Documents and assets are untrusted input. Validation, bounded allocation, archive safety, decoder hardening, and deterministic failures are part of the feature definition.

### Accessibility of the API

Human-readable errors, structured diagnostics, examples, and stable command names matter equally for application developers and agents.

### Versioning

Documents, commands, extensions, and public APIs have explicit compatibility rules before the first stable release. Migration tests use real historical fixtures.

## Deferred tracks

These may be explored after the initial stable release but must not distort the v1 architecture:

- PSD import or export
- CMYK and print-production color workflows
- GIF, SVG, and PDF output
- Video or animation timelines
- 3D layers or scenes
- Built-in multi-user collaboration
- A prescribed, styled editor application

The command log and stable-ID model should avoid foreclosing collaboration, but no collaboration protocol is promised by this roadmap.

## Release gates

A stable release requires:

- All three reference workflows passing end to end
- Browser and Node packages with documented support targets
- A distributable `lg` CLI
- Document migration and corruption-recovery tests
- Visual conformance coverage for supported compositing semantics
- Published performance results for the benchmark corpus
- API reference and task-oriented guides
- A production landing and documentation site with search, version-aware content, and runnable examples
- A defined FOSS license, contribution process, and security policy
