# Graphics primitives plan

## Objective

Expand Layered Graphics from a basic layered renderer into a credible graphics-authoring engine. Add the primitives needed to create and revise polished compositions while following familiar Photoshop-like behavior.

This phase is defined by semantic coherence. A long feature list is not sufficient if masks, groups, clipping, selections, effects, and transforms interact inconsistently.

## Starting baseline

This work starts from `.kgfx` schema v1, stable layer/asset IDs, atomic commands and history, image/fill/bitmap-text/group layers, normal and multiply compositing, authoritative CPU export, retained worker previews, and cross-runtime conformance. These are extension points, not scaffolding to rebuild.

Each slice must extend the existing schema and reducer, `lg` commands, TypeScript facade, Rust/WASM/Node bindings, inspection, history/diff behavior, retained invalidation, authoritative renderer, preview declaration, and public documentation together.

## Milestone 2 boundary

The first two delivery slices form [Milestone 2: template-driven artwork](../MILESTONE_2.md). They prioritize the smallest coherent primitive set needed by form-driven graphics authoring applications: shapes, paths, strokes, gradients, production text, rectangular clipping, image fit/tile helpers, rotation, and PNG/SVG export.

Milestone 2 does not pull selections, painting, filters, direct-manipulation controllers, or application-specific template semantics forward. Its purpose is to prove that a structured recipe can compile into an editable portable composition without flattening or maintaining a second renderer.

## User outcomes

When this plan is complete:

- An agent can create a polished banner using editable text, shapes, images, masks, adjustments, and effects.
- A user can paint, select, mask, transform, and filter content without flattening the document by default.
- Scripts can construct the same documents using commands without editor-only state.
- Familiar Photoshop workflows behave predictably enough that application authors do not need to relearn compositing rules.
- Preview and authoritative renderers declare and test their fidelity for every supported primitive.

## Semantic specification first

Before implementing a feature family, document:

- Its persistent state
- Its coordinate space and bounds
- Its place in the layer and group hierarchy
- Its effect on alpha and color
- Its interaction with masks, clipping, opacity, blending, and selections
- Its invalidation footprint
- Whether preview output may approximate authoritative output
- How unsupported or malformed configurations fail

Small visual examples accompany the specification. Photoshop behavior is the compatibility reference where it is observable and legally reproducible; ambiguities and intentional deviations are recorded.

## Scope

### Complete layer hierarchy and compositing

Expand common layer behavior to cover:

- Nested groups
- Group opacity and blend behavior
- Pass-through versus isolated group compositing
- Expanded blend-mode set
- Clipping masks and clipping chains
- Per-layer raster masks
- Vector masks once editable path support is present
- Layer and mask linking
- Canvas and layer bounds outside the visible canvas

Stack order and effect scope must be consistent in the document, CLI listing, hit testing, previews, and exports.

### Text layers

Text remains editable and retains its source content and layout properties. Initial authoring capability includes:

- Embedded and application-resolved fonts
- Font family, weight, style, and size
- Fill color and opacity
- Line height and letter spacing
- Point text and bounded paragraph text
- Left, center, and right alignment
- Basic wrapping
- Non-destructive transforms
- Missing-font diagnostics and explicit fallback reporting

Text metrics must be stable enough to avoid surprising layout movement between preview and export. Advanced typography can follow after the initial primitive is coherent.

### Shape and fill layers

Support editable:

- Rectangles with corner radii
- Ellipses
- Lines
- Polygons and paths
- Solid fills
- Linear and radial gradients
- Fill and stroke styling
- Basic stroke joins, caps, and alignment
- Non-destructive transforms

Boolean path operations and advanced stroke styling may be staged within the phase after the common path model is proven.

### Adjustment and filter layers

Add non-destructive adjustments with masks, opacity, and blend behavior. The initial set should cover high-value authoring needs:

- Brightness and contrast
- Levels
- Curves
- Hue and saturation
- Color balance or temperature and tint
- Black and white
- Gradient map
- Threshold and posterize

Pixel filters initially include:

- Gaussian blur
- Sharpen
- Basic edge and emboss-style effects where semantics are clear

The specification distinguishes adjustments that operate in a defined color space from legacy-compatible approximations.

### Selection model

Selections are editable channel-like data associated with an authoring session and optionally persisted as named channels. Support:

- Rectangle and ellipse selection
- Polygon and freehand lasso
- Color-range or wand selection with tolerance
- Replace, add, subtract, and intersect operations
- Invert, feather, grow, shrink, and border
- Selection bounds and mask inspection
- Saving and loading named selections

The active transient selection need not pollute the portable graphical layer tree, but commands that modify content must capture enough selection input to replay deterministically.

### Selection-aware operations

Selections constrain:

- Painting and erasing
- Bucket fill
- Clear, copy, cut, and paste
- Raster filters
- Mask creation
- Pixel movement and transformation

Operations define whether they sample the active layer or composited visible content.

### Painting and raster editing

Provide a practical initial paint engine:

- Paint and erase
- Size, hardness, opacity, flow, and spacing
- Pressure-aware input
- Stroke streaming for interactive use
- Alpha lock
- Bucket fill with tolerance and contiguous modes
- Explicit destructive resize, crop, trim, and rasterize operations

Interactive strokes may maintain temporary session state, but a completed stroke must produce a durable, undoable command result. Large pixel deltas should not make the public document manifest unusable.

### Alignment and layout assistance

Offer high-level design assistance without becoming a layout engine:

- Align selected layers to canvas, selection, or reference layer
- Distribute layers by centers or gaps
- Report distances, centers, and visible bounds
- Snap candidates for canvas edges, centers, guides, and other layers
- Fit, contain, cover, and center helpers

These helpers compile into ordinary transform commands so their results remain transparent and replayable.

### Inspection expansion

Extend structured inspection with:

- Fonts and fallback status
- Dominant and declared colors
- Selection and mask bounds
- Visible and transformed bounds
- Alignment and spacing measurements
- Empty or fully clipped layers
- Content outside the canvas
- Basic contrast diagnostics for eligible text/background cases
- Features approximated by the current preview backend

Inspection results distinguish guaranteed facts from pixel-derived estimates.

## Deliverables

- Published compositing semantics for all supported primitives
- Expanded command schemas and imperative wrappers
- Masks, clipping, and full group behavior
- Text and font asset workflow
- Shape, path, fill, and gradient layers
- Adjustments and initial filter set
- Selection system and selection-aware editing
- Paint, fill, and explicit raster operations
- Alignment and distribution helpers
- Expanded inspection and CLI commands
- Visual conformance corpus shared by preview and authoritative renderers
- Fully editable banner example demonstrating the phase
- Primitive guides, compatibility notes, and generated command/API reference published on the site

## Delivery slices

The sequence builds on the shipped renderer instead of opening parallel speculative subsystems.

1. **Milestone 2 compositing contract** — group isolation, rectangular clipping for cover placement, deterministic image contain/cover/tile helpers, rotation, and combination fixtures. Broader raster masks, pass-through behavior, clipping chains, and blend equations may follow after the milestone-critical subset.
2. **Milestone 2 editable vector sources** — rounded rectangles, ellipses, paths, strokes, gradients, production font assets, deterministic text layout, SVG export, and the public template-driven reference workflow.
3. **Bounded effects** — blur, sharpen, levels, curves, brightness/contrast, and hue/saturation across masks, clipping, groups, preview tiers, and invalidation.
4. **Selection channels** — transient/named storage, boolean combination, feather/grow/shrink, inspection, and deterministic command capture.
5. **Raster mutation** — brush/erase streams, fill, clipboard operations, crop/trim/resize/rasterize, compact deltas, and selection-aware replay.
6. **Design queries** — visible bounds, distances, align/distribute/fit helpers, snap candidates, and contrast diagnostics.

Each slice ships a small editable example and benchmark. A slice is incomplete if it only renders or only persists.

## Testing strategy

### Visual conformance matrix

Fixtures cover combinations, not only isolated features:

- Mask plus opacity plus transform
- Clipped layers inside isolated and pass-through groups
- Adjustments above, inside, and clipped to groups
- Text and shapes under non-normal blend modes
- Effects near tile and canvas boundaries
- Selection feathering applied to paint and filters

Authoritative output uses strict expectations. Preview output uses documented tolerances by quality tier.

### Command and history tests

- Every completed operation is replayable from commands and assets.
- Undo and redo restore graphical output and inspectable state.
- Transactions mixing vector, text, raster, and layer operations remain atomic.
- Large paint operations do not cause unbounded manifest or history growth.

### Cross-runtime tests

- Font resolution, layout bounds, and diagnostics are compared across supported runtimes.
- Browser and Node load and render the same portable documents.
- Missing optional capabilities degrade explicitly.

### Workflow test

An agent-facing command sequence creates a polished banner with editable text, shapes, images, masks, adjustments, and alignment helpers; renders it; inspects it; revises it; and proves the document remains editable.

## Performance requirements

- Active paint strokes provide low-latency preview feedback and commit without long UI stalls.
- Selection refinement and common adjustments remain interactive on representative document sizes.
- Effect sampling bounds integrate with tiled invalidation without seams.
- Text editing invalidates affected layout and composition regions, not unrelated assets.
- Hidden, clipped-out, and off-viewport content is skipped where dependency rules permit.

Each primitive adds at least one benchmark or extends an existing mixed workload.

## Exit criteria

This plan is complete when:

- The editable banner workflow passes using public APIs and the CLI.
- Supported Photoshop-like semantics are documented with combination fixtures.
- Every new primitive works through persistence, commands, history, inspection, preview, and export.
- Selection-aware paint, fill, filtering, copy, and transform operations pass replay tests.
- Missing fonts and unsupported preview features produce explicit diagnostics.
- Mixed-feature incremental renders match cold renders within declared tolerances.
- Performance budgets cover painting, selections, effects, and text.

## Deferred

- Full PSD compatibility
- Complete OpenType and professional typesetting controls
- Every Photoshop blend mode and filter
- Advanced vector illustration tooling
- Styled editor components
- Multi-user collaboration

Deferred breadth must not weaken the consistency of the primitives that are declared supported.
