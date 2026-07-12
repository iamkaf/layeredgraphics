# Phase 1: Executable Document

## Objective

Create the smallest complete Layered Graphics system: a portable document that can be mutated through commands, inspected, rendered, and managed from JavaScript or the CLI.

This phase also scaffolds the Cargo/pnpm monorepo described in [Technology Stack](../TECH_STACK.md), including the initial Rust core and CLI crates, WASM and native Node bindings, and TypeScript SDK packages.

This phase establishes contracts that later rendering and editing work must preserve. It should favor a narrow, coherent model over prematurely implementing every graphics feature.

## User outcomes

At the end of this phase:

- A developer can create a `.kgfx` file, embed assets, add and rearrange layers, render it, and reopen it without losing information.
- A script can execute an array of operations atomically and receive structured results.
- An agent can inspect a document, understand its layer tree and assets, apply targeted changes, validate the result, and render an image.
- A caller can use an imperative JavaScript API without creating a second mutation model.
- A simple README banner can be created entirely through the CLI.

## Scope

### Document foundation

Define a versioned graphical document containing:

- Document identity and schema version
- Canvas width, height, DPI, and initial color settings
- Stable IDs for layers and assets
- A hierarchical layer tree
- Namespaced extension metadata
- Embedded and linked asset records
- Revision information needed for changesets and history

The graphical manifest should remain structurally inspectable even when large asset payloads are embedded. Unknown namespaced extension data must round-trip without interpretation.

### Initial layers

Implement only the layer kinds needed to validate the architecture:

- Image/raster layer referencing an asset
- Solid fill layer
- Group layer

Common properties include:

- Name
- Visibility
- Opacity
- Blend mode, initially normal plus a small compatibility-tested set
- Position and non-destructive transform
- Parent and stack position

The model must allow later addition of masks, filters, text, shapes, adjustments, and clipping without changing the command philosophy.

### Assets

Support:

- Embedded assets as the portable default
- Linked assets with an explicit reference and resolution status
- Stable asset IDs independent of filenames
- Media type, byte length, integrity metadata, and optional author metadata
- Deduplication without merging distinct logical asset IDs
- Missing-asset diagnostics

The engine must not assume that linked assets are filesystem paths. Applications may resolve references from files, URLs, memory, or browser storage.

### Command execution

Commands are data, not method calls hidden behind runtime objects. The initial command families cover:

- Document property updates
- Layer add, remove, update, and move
- Asset add, remove, replace, and relink
- Extension metadata set and remove

Execution requirements:

- Validate an entire transaction before committing it where possible.
- Either commit an atomic transaction completely or leave the document unchanged.
- Return command-indexed errors and warnings.
- Produce a changeset describing affected document regions and resources.
- Preserve caller-supplied stable IDs and generate IDs when omitted.
- Reject stale or invalid references predictably.

The initial history model records reversible transactions. Undo and redo operate on transaction boundaries rather than individual internal mutations.

### JavaScript APIs

Provide two equivalent authoring styles:

- A command-oriented API centered on `doc.execute(ops)`
- An imperative convenience API whose methods compile into the public command protocol

Both APIs must expose consistent validation errors, transaction semantics, and changesets. Examples and tests should demonstrate equivalence.

The supported runtime targets are modern browsers and current supported Node releases. Exact support versions will be fixed before implementation release candidates.

### Reference rendering

Implement a correctness-oriented renderer for the initial layers and common properties. It must support:

- Full-document render
- Individual-layer or group render
- Explicit output scale
- Transparent or caller-specified background
- PNG, JPEG, and WebP output
- Nearest-neighbor and smooth sampling policies

This renderer becomes the initial semantic reference. Phase 2's GPU preview may approximate it, but fixtures produced here define expected ordering, alpha, opacity, transform, and blend behavior.

### Inspection and validation

Structured inspection initially reports:

- Document and schema information
- Canvas properties
- Layer tree, common properties, and bounds
- Asset inventory and resolution state
- Extension namespaces
- Validation errors and warnings

Inspection should distinguish declared layer bounds, transformed bounds, and visible-content bounds where available. Expensive pixel analysis should be opt-in.

Validation operates on both `.kgfx` documents and standalone command arrays.

### CLI foundation

Implement the initially proposed command surface:

```text
lg new
lg exec
lg layer add|update|rm|ls|move
lg asset add|ls|rm
lg render
lg inspect
lg validate
lg diff
lg watch
```

CLI behavior requirements:

- `-` denotes standard input where applicable.
- Mutating commands use safe replacement so interrupted writes do not corrupt the previous document.
- Commands have useful human output and stable `--json` output.
- Failures use documented nonzero exit codes.
- Diagnostics go to standard error when standard output contains machine-readable data.
- `lg exec` accepts one operation or an array and applies the input as one transaction by default.
- `lg diff` emits a valid operation array that transforms the supported graphical state of A into B.

`lg watch` may begin with a straightforward implementation; render-session reuse is improved in Phase 2.

## Deliverables

- Cargo and pnpm workspaces with shared development commands and continuous integration
- Published draft of the `.kgfx` container and manifest specification
- Published command protocol and error shape
- Document reducer with transactions and history
- Asset store abstraction with embedded and linked implementations
- Initial reference renderer
- Browser and Node JavaScript packages
- Native Node packages for the initially supported platforms
- `lg` CLI package
- Schema migration harness
- Example banner document, operation scripts, and exported output
- API equivalence, round-trip, migration, and corruption tests
- Initial Astro/Starlight site with product landing page, getting-started guide, CLI guide, and executable phase-1 examples

## Work sequence

1. Write semantic examples for document, layer ordering, transforms, assets, and transactions.
2. Define the manifest, extension envelope, command envelope, changeset, and diagnostics.
3. Implement in-memory document loading, validation, and command reduction.
4. Add history and reversible transaction coverage.
5. Implement the `.kgfx` container and safe persistence.
6. Implement assets and decoding for the initial formats.
7. Implement reference compositing for initial layers.
8. Expose command and imperative JavaScript APIs.
9. Build CLI commands on the same public APIs.
10. Complete the banner fixture and publish the draft specifications.

## Testing strategy

### Contract tests

- Command JSON examples validate against the published schema.
- Imperative calls emit behaviorally equivalent commands.
- Changesets identify every affected layer, asset, and region required by downstream consumers.
- Unknown extension namespaces survive round trips.

### Persistence tests

- Save/load is lossless for supported state.
- Embedded and linked asset behavior is consistent across browser and Node.
- Interrupted or invalid writes preserve the previous valid file.
- Malformed archives, oversized declarations, missing assets, and unsupported versions fail safely.

### Visual tests

- Layer order, transparency, opacity, transforms, sampling, and supported blends have small golden fixtures.
- Export formats are decoded and compared in a format-appropriate way.
- Pixel-art sampling has dedicated fixtures.

### Workflow test

A checked-in operation sequence must create a layered README banner, inspect it, validate it, and export it using only `lg` commands.

## Performance baseline

Phase 1 records baselines rather than promising final interactive performance. Measure:

- Document open/save time with embedded assets
- Command execution time for shallow and deep layer trees
- Full reference-render time for sprite-sized, 2K, and 4K canvases
- Peak memory while decoding and exporting
- CLI startup overhead

The benchmark documents become stable fixtures for later phases.

## Exit criteria

Phase 1 is complete when:

- The banner workflow passes from a clean checkout.
- Document and operation schemas are documented and versioned.
- Transactions, history, diffs, and safe persistence pass their contract tests.
- Browser and Node execute the same fixture operations.
- Supported reference-render fixtures pass consistently.
- Invalid and hostile document fixtures fail within defined resource limits.
- No later phase requires a competing document mutation path.

## Deferred

- Interactive GPU rendering
- Masks, selections, text, shapes, painting, and adjustment layers
- Editor interaction controllers
- Spriteform migration
- Stable v1 compatibility guarantees

These are deferred features, not reasons to leave the phase-1 extension and invalidation contracts undefined.
