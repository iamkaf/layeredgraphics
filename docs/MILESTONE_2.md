# Milestone 2: template-driven artwork

## Outcome

An application, script, or agent can turn a structured artwork recipe into an editable Layered Graphics document, preview it interactively, revise its inputs, and export publication-ready PNG and SVG output.

The reference workflow is a form-driven artwork studio for icons, repository heroes, feature cards, social previews, and compatibility cards. The application owns templates, palettes, copy fields, safe-area guidance, asset roles, and publication constraints. Layered Graphics owns the canonical composition, assets, history, preview, persistence, and rendering.

This is deliberately narrower than a general-purpose image editor. It does not require canvas hit testing, selection handles, painting, clipboard behavior, or a prescribed UI.

## Required graphics vocabulary

- Editable rectangle, rounded-rectangle, ellipse, and path layers
- Solid, linear-gradient, and radial-gradient fills
- Fill and stroke styling, including width, joins, caps, and opacity
- Translation, scale, and rotation for graphical layers
- Production text backed by embedded or application-resolved font assets
- Font family, weight, size, letter spacing, alignment, and deterministic line layout
- Image contain, cover, and tile placement with nearest or smooth sampling
- Rectangular clipping for cover placement and template-safe composition
- Nested groups with predictable opacity and isolation
- PNG and SVG export from the same canonical document

Application helpers such as `fit`, `contain`, `cover`, `center`, and integer-scale placement compile into ordinary commands. Seeded decorative patterns are emitted as editable shape/path layers rather than hidden renderer behavior.

## Required workflow

The public APIs and CLI must support this sequence:

1. Create a fixed-size document from a surface definition.
2. Embed or link fonts and PNG, JPEG, or WebP assets.
3. Compile a recipe containing copy, palette, pattern seed, density, and asset roles into one atomic operation batch.
4. Change templates without losing source assets or application-owned recipe metadata.
5. Reorder, hide, reposition, rescale, crop, tile, and adjust the opacity of imported image layers.
6. Update copy, palette, compatibility labels, and deterministic patterns while preserving editable primitives.
7. Preview revisions through the browser worker without encoded-image round trips.
8. Undo and redo recipe revisions at meaningful transaction boundaries.
9. Save an editable `.kgfx` document and reopen it in browser and native runtimes.
10. Export PNG and SVG, then report dimensions and byte size for application-owned publication limits.

## Public integration boundary

Template and recipe semantics remain application-owned and are stored in a namespaced document extension when portability is useful. The engine does not gain concepts such as “social card,” “loader,” or “brand pack.” It exposes graphical primitives and transparent layout helpers that many authoring applications can reuse.

The reference integration should be a small public example with several surface presets and real editable outputs. It must use only published interfaces; no private renderer hooks or fixture-only document mutation are allowed.

## Verification

- A checked recipe fixture produces stable PNG and SVG outputs.
- Browser, native Node, CLI, and authoritative Rust output agree on document geometry and raster output.
- Font metrics and fallback diagnostics are explicit and reproducible.
- Contain, cover, tile, clipping, rotation, gradients, strokes, and integer scaling have focused visual fixtures.
- Recompiling an unchanged recipe produces no graphical diff.
- Changing one recipe field produces a bounded changeset and retained-preview invalidation.
- Exported files satisfy declared dimensions, and byte-size reporting is exact.
- The editable reference document survives save, reopen, migration, diff, undo, and redo tests.

## Exit criteria

Milestone 2 is complete when the public reference studio can create all supported surface presets from structured recipes, imported raster assets, and embedded fonts; save and reopen editable `.kgfx`; and export visually approved PNG and SVG files through Layered Graphics.

The milestone is not complete if templates are flattened into single raster layers, text is pre-rendered, SVG uses an unrelated application renderer, or preview and export derive from different composition models.
