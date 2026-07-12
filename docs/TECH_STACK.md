# Technology Stack

## Status

This document records the initial implementation direction for Layered Graphics. Individual libraries may change when spikes or measurements justify it, but the language, runtime, repository, and renderer boundaries are architectural decisions.

## Repository model

Everything required to build and release Layered Graphics lives in this repository:

- Rust engine crates
- WebAssembly bindings
- Native CLI
- TypeScript packages
- Browser worker runtime
- Editor toolkit and optional framework bindings
- Specifications and generated schemas
- Examples, compatibility fixtures, fuzz targets, and benchmarks
- Build, packaging, release, and documentation tooling

The repository uses a Cargo workspace and a pnpm workspace. The root task runner should coordinate them without hiding the underlying Cargo and pnpm commands. A contributor must be able to work on either side independently.

An intended starting layout is:

```text
crates/
  lg-document/          canonical document, commands, history, validation
  lg-container/         .kgfx persistence, assets, migrations
  lg-render/            shared render graph and compositing semantics
  lg-render-cpu/        authoritative CPU renderer
  lg-render-gpu/        retained GPU preview renderer
  lg-codecs/            image decode and encode boundary
  lg-wasm/              browser/JavaScript WASM bindings
  lg-node/              native Node bindings
  lg-cli/               native lg executable

packages/
  core/                 TypeScript command types and SDK entry point
  node/                 Node SDK and native package loader
  worker/               browser worker client and host
  editor/               framework-neutral editor toolkit
  react/                optional React bindings

apps/
  site/                 Astro landing and Starlight documentation site
  playground/           engine and renderer development surface
  example-editor/       unstyled-toolkit integration example

spec/
  document/             .kgfx and manifest specification
  commands/             command and diagnostic specification

fixtures/
  conformance/          visual semantics fixtures
  performance/          stable benchmark documents
  workflows/            banner, thumbnails, and Spriteform cases
```

Names are provisional until the workspace is scaffolded. Package boundaries should be retained even if some begin in fewer physical crates.

## Rust engine

Rust owns behavior that must remain consistent across interactive, scripted, and agent workflows:

- Canonical document state
- Command validation and reduction
- Transactions, history, changesets, and migrations
- `.kgfx` container and asset integrity
- Render-graph construction
- Photoshop-like compositing semantics
- Authoritative CPU rendering
- Native CLI execution

The core document crate should avoid platform-specific I/O and runtime dependencies. Filesystem, browser storage, networking, and application asset resolution live behind host interfaces.

The baseline is stable Rust using the current edition selected during scaffolding. Unsafe Rust is not prohibited, but requires a localized abstraction, written safety rationale, and focused tests or benchmarks demonstrating why it is necessary.

## WebAssembly boundary

The Rust engine compiles to WebAssembly for browser use. Bindings use `wasm-bindgen` and expose a deliberately small session-oriented surface rather than mirroring every Rust type or method.

Boundary guidelines:

- Commands and diagnostics use generated, versioned data contracts.
- Large pixel and asset buffers cross by transferable or shared storage where supported, not repeated JSON/base64 encoding.
- Hot interactive loops stay on one side of the boundary.
- Browser work normally runs in a dedicated worker.
- Optional capabilities are split or loaded lazily when doing so materially improves startup size.

Node uses native Rust bindings as its standard high-throughput path. The bindings use `napi-rs` unless the scaffolding spike exposes a blocking distribution or compatibility issue. The WASM distribution remains usable in Node as a portability and contract-testing fallback. Both paths execute the same core Rust behavior and pass the same command and rendering fixture suites.

## TypeScript SDK

TypeScript owns the developer-facing JavaScript experience:

- Strict public command, document-view, diagnostic, and result types
- Runtime loading and capability detection
- Worker client and message coordination
- Ergonomic imperative API compiled into public commands
- Subscriptions, cancellation, and resource lifetimes
- Framework-neutral editor controllers
- Optional React integration

The TypeScript SDK must not reimplement document mutation or compositing rules. Pure presentation geometry and browser-input behavior belong in TypeScript when they are editor concerns rather than document semantics.

Packages use strict TypeScript and modern ESM. Build output and supported browser/Node targets will be fixed before the first published prerelease.

The public SDK keeps runtime selection behind stable TypeScript interfaces: browser consumers load WASM and worker support, while Node consumers load the matching native package. Runtime-specific entry points remain available for applications that need explicit control.

## Rendering stack

### Authoritative rendering

The authoritative renderer is CPU-based Rust and defines supported output semantics. It is used for:

- Final PNG, JPEG, and WebP exports
- Golden conformance fixtures
- Deterministic debugging and cold-render comparisons
- Environments without a supported GPU

Rasterization, text, image decoding, and color libraries are selected behind internal interfaces. Likely initial candidates include `tiny-skia` for 2D rasterization and `cosmic-text`/`swash` for text shaping and glyph rendering, but conformance, WASM size, licensing, and performance spikes decide the final dependency set.

### Preview rendering

The preview renderer uses WebGPU, implemented through `wgpu` where it allows useful code and shader sharing without compromising browser integration. It consumes the same render graph and changesets as the authoritative renderer.

Preview rendering may reduce resolution or approximate documented effects according to an explicit quality tier. It must preserve structural behavior such as coordinates, ordering, clipping relationships, and transaction revisions.

### Fallback

When WebGPU is unavailable or lost, the worker uses the CPU/WASM renderer for functional previews. A WebGL-specific renderer is not part of the initial plan; it should only be added if real support targets show that CPU fallback is insufficient.

## CLI

`lg` is a native Rust executable built on the same document, container, command, and authoritative-render crates. Native implementation provides fast startup, direct filesystem access, low-overhead batch rendering, and simple distribution for agent environments.

CLI schemas and output shapes are shared with the TypeScript SDK. Shell behavior, JSON output, exit codes, and standard-stream usage are public compatibility surfaces.

## `.kgfx` container

The portable file contains a versioned structured manifest plus content-addressed binary assets. The initial implementation is expected to use a ZIP-compatible container and JSON manifest because they are widely inspectable and recoverable.

This is an implementation choice, not permission for callers to depend on archive layout. The public contract is the `.kgfx` specification and APIs. Container safety includes bounded expansion, path validation, integrity checking, and atomic replacement.

If JSON size or parsing becomes a demonstrated bottleneck, indexed or binary metadata can be introduced through a format version without changing the command model.

## Schemas and code generation

Document, command, diagnostic, and inspection contracts need one normative definition. The scaffold phase will spike whether Rust types plus generated JSON Schema and TypeScript types provide adequate output, or whether a language-neutral schema source is preferable.

Whichever route is selected must enforce:

- No manually maintained duplicate command unions
- Checked-in or reproducibly generated public schemas
- Compatibility tests for Rust, WASM, TypeScript, and CLI parsing
- Reviewable schema changes in pull requests

## Build and development tooling

Initial tooling direction:

- Cargo for Rust build, test, benchmark, and fuzz targets
- pnpm workspaces for JavaScript dependencies and packages
- A thin root task layer for common cross-workspace commands
- Vitest for TypeScript unit and contract tests
- Browser integration tests using Playwright
- Rust benchmarks using Criterion or the standard benchmark approach selected during scaffolding
- Rust fuzzing for document, container, command, and codec boundaries
- A shader validation step for GPU pipelines

## Website and documentation

The landing page and documentation are one statically generated application under `apps/site`:

- [Astro](https://docs.astro.build/en/basics/astro-pages/) provides custom marketing pages, file-based routing, build-time content, and selective interactive islands.
- [Starlight](https://starlight.astro.build/) provides the documentation shell, Markdown/MDX authoring, accessible navigation, code presentation, SEO foundations, dark mode, and localization readiness.
- Starlight's built-in [Pagefind search](https://starlight.astro.build/guides/site-search/) provides static full-text search without an application server.

The homepage uses a custom Astro layout rather than forcing marketing content into a conventional documentation page. Documentation uses Starlight layouts and shared design tokens so both surfaces feel like one product.

The site is static by default. Interactive engine examples load client-side WASM/WebGPU only on pages that request them. API reference and schema documentation are generated from the same checked-in or reproducibly generated contracts shipped by the packages.

The site builds in continuous integration, checks internal links and code examples, and produces a deployable static artifact. Hosting remains provider-neutral until deployment configuration is selected.

Formatting and linting use standard ecosystem tools: `rustfmt`, Clippy, and a TypeScript formatter/linter selected during scaffolding. Generated code is verified as up to date in continuous integration.

## Testing architecture

The monorepo enables one fixture corpus to drive every runtime:

- Rust unit and property tests validate canonical semantics.
- WASM contract tests execute the same command fixtures in browsers.
- Preview conformance compares WebGPU output with authoritative CPU output using declared tolerances.
- CLI end-to-end tests exercise real packaged files and standard streams.
- TypeScript tests verify imperative calls compile to equivalent commands.
- Workflow fixtures cover the README banner, thumbnail batch, and Spriteform composition.

Correctness fixtures are small and diagnostic. Performance fixtures are representative and stable. Neither should depend on private assets outside this repository.

## Dependency policy

External libraries are welcome when they provide a well-tested primitive. Dependencies are evaluated for:

- Browser and WASM compatibility
- CPU and GPU performance
- Binary and package size
- Deterministic or explainable behavior
- Maintenance health
- FOSS license compatibility
- Security history and input-hardening posture

Layered Graphics should integrate strong primitives rather than recreate codecs, text shaping, or GPU abstraction without a product-specific reason. Public APIs must still describe Layered Graphics concepts instead of leaking dependency-specific types.

## Decisions intentionally left open

These require implementation spikes or measurements:

- Exact CPU rasterization and text libraries
- The root task runner and TypeScript formatter/linter
- The normative schema/code-generation source
- Shader language organization and preprocessing
- Which optional capabilities deserve separate WASM artifacts

Resolving these does not change the chosen architecture: Rust core, WASM browser execution, TypeScript SDK/toolkit, WebGPU previews, CPU authoritative exports, and one monorepo.
