# Phase 5: Spriteform Integration and Production Hardening

## Objective

Prove Layered Graphics inside a real authoring application, complete the scripted batch story, and turn the accumulated engine into a stable FOSS release with explicit compatibility, performance, security, and maintenance policies.

Spriteform is the primary integration partner, not a source of application-specific features for the core engine.

## User outcomes

At the end of this phase:

- Spriteform authors receive fast interactive composition previews and authoritative exports from Layered Graphics.
- Spriteform retains its smart-layer, variant, and turbo-generation workflows without duplicating graphics primitives.
- Scripts generate large thumbnail sets through a documented, bounded batch API.
- Agents can install the CLI and complete the banner workflow using stable commands and structured inspection.
- External developers can adopt the project with clear support, versioning, contribution, and security expectations.

## Integration boundaries

### Layered Graphics owns

- Graphical document state and assets
- Commands, transactions, history, and changesets
- Layer compositing and editing semantics
- Preview and export rendering
- Editor behavior exposed by the unstyled toolkit
- Graphical inspection and validation

### Spriteform owns

- Sprite packs and filesystem organization
- Base sprite inventory
- Smart layers and named variants
- Turbo generators and combinatorial selection rules
- Tags, groups, export metadata, and pack workflows
- Product UI and assistant behavior

### Adapter responsibility

The integration adapter maps a chosen Spriteform composition state to a Layered Graphics document or command transaction. It maintains stable identity where possible so changing one variant invalidates only affected graphical content.

Spriteform metadata may be stored under a namespaced `.kgfx` extension when portability is useful, but the engine does not interpret variant-generation semantics.

## Scope

### Spriteform model adapter

Build a documented translation for:

- Composite canvas settings
- Sprite-part layers and source assets
- Paint layers
- Folders/groups
- Filters and adjustments
- Visibility, ordering, opacity, anchors, flips, rotation, and position
- Smart-layer variant selection as application-driven transactions
- Stable mapping between Spriteform records and graphical layer IDs

The adapter reports unsupported source behavior explicitly and includes migration tests for representative packs.

### Interactive compositor migration

Replace preview generation with a persistent worker-backed render session. The UI should:

- Execute graphical changes without serializing and rerendering an entire pack
- Present GPU preview output directly
- Use interactive and refined quality tiers appropriately
- Preserve pixel-art nearest-neighbor behavior
- Cancel obsolete previews
- Surface missing or invalid asset diagnostics

Where Spriteform maintains application state, updates and engine transactions need a clear atomic ordering and recovery policy.

### Export migration

Move magic-sprite and turbo-sprite output to authoritative Layered Graphics rendering. Preserve:

- Existing canvas and placement expectations
- Pixel-art sampling
- Transparency
- Cancellation and progress
- Atomic output writes
- Reproducible naming and variant selection

Before switching defaults, run old and new renderers against a representative corpus and classify every difference as a bug, an intentional compatibility correction, or an accepted migration change.

### Thumbnail pipeline

Use warm batch sessions for:

- Magic sprite thumbnails
- Turbo variant thumbnails
- General scripted thumbnail generation

The pipeline supports bounded concurrency, priority for visible results, cancellation, progress, backpressure, and cache reuse. Rendering thousands of candidates must not require materializing every full output simultaneously.

### Agent workflow hardening

Exercise the CLI through realistic agent sessions:

- Create and inspect a document
- Import supplied assets and fonts
- Construct and revise a banner
- Render previews and final outputs
- Diagnose missing assets, overflow, fonts, alignment, and validation failures
- Diff revisions and apply operation arrays through standard input

Revise command names, structured output, error messages, and help text based on observed friction. Agent ergonomics must also preserve predictable shell scripting behavior.

### Package and runtime hardening

Prepare supported distributions for:

- Browser applications
- Browser workers
- Node scripts
- The `lg` executable
- Framework-neutral editor toolkit
- Optional framework bindings

Define support policy for browser versions, Node versions, GPU capability levels, and fallback behavior. Package boundaries should allow renderer-only or document-only consumers to avoid unnecessary payload where practical.

### Format and API stability

Before stable release, publish:

- `.kgfx` compatibility and migration policy
- Command protocol versioning policy
- Extension namespace rules
- Public JavaScript API stability policy
- Preview-versus-authoritative fidelity policy
- Deprecation process
- Corruption and recovery behavior

Historical fixtures begin with every pre-release document version that users could reasonably have persisted.

### FOSS readiness

Complete:

- License selection and notices
- Contribution guide and code of conduct
- Governance and maintainer expectations
- Security reporting policy
- Release and changelog process
- Reproducible development and test instructions
- Architecture and task-oriented documentation

Third-party libraries, codecs, fonts, fixtures, and compatibility references receive a license and provenance review.

## Deliverables

- Spriteform adapter package or feature-local integration layer
- Migrated interactive compositor preview
- Migrated magic/turbo export path
- Production thumbnail batch pipeline
- Compatibility report for the old and new Spriteform renderers
- Stable CLI workflow and agent-oriented examples
- Supported package matrix and runtime policy
- Stable document, command, extension, and API policies
- Security, contribution, governance, and release documentation
- Published performance and conformance results
- Initial stable FOSS release candidate
- Production landing/docs deployment with stable-version documentation, migration guidance, and release notes

## Migration strategy

Use staged adoption rather than a single replacement:

1. Translate representative composites and render offline comparisons.
2. Add the new renderer behind a development option.
3. Migrate thumbnail previews and observe cache and memory behavior.
4. Migrate interactive compositor preview.
5. Run authoritative exports in comparison mode.
6. Switch new exports after differences are resolved and rollback is proven.
7. Remove the old rendering path only after the supported corpus and real workflows are stable.

Persisted Spriteform pack migrations remain separate from `.kgfx` migrations. Avoid rewriting packs merely to enable preview rendering when an adapter can translate them in memory.

## Testing strategy

### Corpus comparison

Maintain anonymized or distributable representative packs covering:

- Small pixel sprites
- Deep folder trees
- Paint and filter layers
- Multiple smart layers and variant combinations
- Missing, renamed, and replaced source assets
- Large turbo generation sets

Compare dimensions, alpha, placement, sampling, and pixels. Review differences visually and structurally.

### End-to-end application tests

- Edit a base layer and verify preview, history, persistence, and export.
- Switch variants rapidly and verify stale work is cancelled.
- Generate thumbnails while prioritizing visible entries.
- Close or change packs during active jobs without leaks or corrupt writes.
- Restart after interruption and recover canonical application state.

### CLI and agent tests

- Run documented commands from a clean installation.
- Treat JSON output and exit codes as compatibility surfaces.
- Exercise paths with spaces, standard input, missing files, corrupted documents, and unsupported features.
- Record representative agent attempts and convert recurring failures into fixtures or documentation.

### Release tests

- Install packages in clean browser and Node example projects.
- Verify published artifacts contain required binaries, types, schemas, and licenses.
- Test document migration from every historical fixture.
- Run conformance and benchmark suites against release artifacts, not only source builds.

## Production performance gates

Set numeric budgets using the established benchmark corpus and real Spriteform traces. At minimum, gate:

- Preview latency during common layer and variant edits
- Idle refinement latency
- Thumbnail throughput and time to first visible thumbnail
- 2K and 4K authoritative export time
- Peak memory during large documents and large batches
- CLI cold-start time for inspect and simple mutation commands
- Browser package size by supported entry point

Published results identify hardware, browser/runtime, document fixture, quality tier, and warm/cold state.

## Reliability and security gates

- Fuzz or property-test document parsing, command validation, and supported decoders.
- Bound archive expansion, image dimensions, allocation requests, nesting depth, and command batch size.
- Define timeouts or cancellation points for expensive work.
- Sanitize or reject active content in vector and metadata inputs.
- Do not resolve linked assets across unexpected trust boundaries without application authorization.
- Verify safe replacement for CLI and application export writes.
- Provide actionable failures instead of panics or worker disappearance.

## Exit criteria

Phase 5 is complete when:

- README banner, thumbnail batch, and Spriteform workflows all pass end to end.
- Spriteform uses Layered Graphics for supported preview and export paths with an exercised rollback plan.
- Application-specific variant logic has not leaked into core graphics semantics.
- Stable package artifacts install and run in every supported environment.
- Document migrations and unknown extensions pass historical round-trip tests.
- Performance, conformance, and security release gates pass.
- Public API, CLI, format, support, and deprecation policies are published.
- FOSS governance, contribution, licensing, and security documentation is complete.

## Post-v1 candidates

After the stable foundation is proven, evaluate based on user workflows rather than parity for its own sake:

- Additional blend modes, adjustments, filters, and brush behavior
- Advanced typography and vector paths
- GIF, SVG, or PDF export
- PSD import or export
- Higher bit depth and expanded color management
- Collaboration protocols built on commands and stable IDs
- Additional framework bindings

CMYK production, video, and 3D remain separate product decisions rather than assumed extensions of the v1 engine.
