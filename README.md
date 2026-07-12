# Layered Graphics

Layered Graphics is a FOSS, browser-first graphics engine for embedding Photoshop-like authoring into applications, scripts, and AI-agent workflows.

The project is headless by design. It provides a portable layered document, compositing and editing semantics, fast previews, high-quality exports, automation APIs, a CLI, and an unstyled editor toolkit. Applications remain in control of their interface and product-specific concepts.

> Project status: planning and specification. The public API and `.kgfx` format are not yet stable.

## Why Layered Graphics?

Most image libraries operate on a single pixel buffer. That works well for resize-and-export pipelines, but becomes awkward when a product needs editable layers, masks, selections, text, non-destructive effects, history, or interactive previews.

Full editor frameworks tend to impose their own interface and application model. Layered Graphics occupies the space between those approaches: a reusable graphics-authoring foundation that can live inside any application.

The guiding questions are:

- What if an application could embed the useful parts of Photoshop without embedding a Photoshop-shaped UI?
- What if an AI agent could build and revise a complete composition without a human opening an editor?
- What if the same document and operations worked in a browser, a script, and a CLI?

## Intended workflows

Layered Graphics is designed around three initial end-to-end workflows.

### Agent-authored graphics

An agent can create a `.kgfx` document, add assets and layers, inspect its structure, render a preview, visually evaluate the result, revise it, and deliver both the editable document and exported image.

One representative task is generating a polished banner for a project README from a prompt, supplied assets, shapes, and text.

### Scripted image production

A script can load a document or template, change assets and properties, and render many outputs efficiently. Thumbnail generation is the initial batch-rendering benchmark.

### Embedded authoring

Applications can build their own editor using the document engine and unstyled interaction toolkit. Spriteform is the first integration target: its smart layers, variants, and generated sprites remain application-level concepts that compile into ordinary graphical layers.

## Product layers

The project is organized as a set of adoptable layers:

1. **Document and command model** — portable state, validated mutations, transactions, history, diffs, extensions, and assets.
2. **Graphics primitives** — raster, paint, text, shape, fill, group, mask, adjustment, filter, selection, transform, blend, and clipping behavior.
3. **Rendering** — an incremental GPU preview path and an authoritative high-quality export path.
4. **Automation surfaces** — JavaScript APIs, worker sessions, and the `lg` CLI.
5. **Editor toolkit** — framework-neutral viewport, hit-testing, transform, painting, selection, snapping, keyboard, and clipboard behavior.
6. **Project site** — a custom landing experience, task-oriented documentation, generated API reference, examples, and an interactive playground.

Consumers can use the document and renderer alone, add automation, or build a full editor without adopting a prescribed visual interface.

## Technology stack

Layered Graphics is developed as one monorepo containing the engine, runtime bindings, CLI, JavaScript packages, editor toolkit, examples, specifications, and tests.

- **Rust** implements the canonical document engine, command reducer, asset/container handling, authoritative CPU renderer, native Node bindings, and native `lg` CLI.
- **WebAssembly** brings the Rust engine and authoritative renderer to browsers and provides the portable execution path for JavaScript runtimes.
- **TypeScript** provides the public JavaScript SDK over WASM in browsers and native bindings in Node, plus the worker protocol, imperative API, editor toolkit, and optional framework bindings.
- **WebGPU** powers retained interactive previews. A documented fallback preserves functionality when WebGPU is unavailable.
- **Cargo workspaces and pnpm workspaces** manage the Rust and TypeScript portions of the repository.
- **Astro and Starlight** power the landing and documentation site as a statically deployable monorepo application.

The engine remains modular: consumers should not need to download editor bindings, optional graphics capabilities, or native integrations they do not use. See [Technology Stack](docs/TECH_STACK.md) for package boundaries, tooling, and dependency policy.

## Design principles

### Commands are the common language

Every document mutation is represented as a validated command. User gestures, imperative API calls, scripts, CLI subcommands, and agent actions all pass through the same execution path.

```ts
doc.execute([
  {
    type: "layer.add",
    layer: {
      id: "hero",
      type: "image",
      name: "Hero",
      assetId: "hero-image",
    },
  },
  {
    type: "layer.update",
    id: "hero",
    set: {
      opacity: 0.7,
      blendMode: "multiply",
    },
  },
]);
```

An ergonomic imperative API compiles to those commands rather than defining separate behavior:

```ts
const hero = doc.layers.add({
  id: "hero",
  type: "image",
  assetId: "hero-image",
});

hero.update({ opacity: 0.7, blendMode: "multiply" });
```

### The document is canonical

The document engine owns canonical graphical state. Executing commands produces a new revision and a changeset that renderers, editor bindings, and application stores can observe.

Rendering state is derived and replaceable. Applications may attach their own models, but should not maintain a competing copy of the layer tree.

### Editing is non-destructive by default

Transforms, text, shapes, masks, filters, adjustments, and cropping remain editable unless a caller explicitly requests a destructive raster operation.

### Preview and export have different jobs

Browser previews prioritize latency and may trade small amounts of detail for responsiveness. Authoritative exports prioritize quality, repeatability, and compatibility with the documented compositing semantics.

### Compatibility over novelty

Where established expectations exist, Layered Graphics follows Photoshop-like behavior for layer ordering, groups, clipping masks, selections, blend modes, masks, and effect scope. Deviations are documented and tested.

### Applications can extend without fragmenting the format

`.kgfx` defines a portable graphical core and permits namespaced application metadata. Unknown extensions are preserved across load and save. A consumer that does not understand a Spriteform extension can still inspect and render its graphical composition.

## The `.kgfx` document

A `.kgfx` file is portable and self-contained by default. It contains a versioned document manifest and embedded assets such as images and fonts. Linked assets are supported for workspace-oriented applications.

The core model includes:

- Canvas dimensions, DPI, and color settings
- Stable layer, mask, and asset identifiers
- A hierarchical layer tree
- Layer properties and non-destructive operations
- Embedded or linked assets
- Namespaced application metadata
- Schema version and migration information

The format must be safe to validate without rendering and practical to inspect or modify through the CLI.

## CLI direction

The CLI is a first-class authoring interface:

```text
lg new <file>.kgfx [--width N --height N --dpi N]
lg exec <file>.kgfx [ops.json | -]

lg layer add <file>.kgfx --type image --asset-id hero [--id ...]
lg layer update <file>.kgfx <id> --set opacity=0.7 --set blend=multiply
lg layer rm <file>.kgfx <id>
lg layer ls <file>.kgfx [--json]
lg layer move <file>.kgfx <id> --to 3 | --above <id> | --below <id>

lg asset add <file>.kgfx --id hero ./hero.png
lg asset ls <file>.kgfx
lg asset rm <file>.kgfx <id>

lg render <file>.kgfx -o out.png [--format png|jpg|webp --layer <id> --scale 2]
lg inspect <file>.kgfx [--path layers.0.opacity]
lg validate <file.kgfx | ops.json>
lg diff <a>.kgfx <b>.kgfx
lg watch <file>.kgfx --ops ops.json --render out.png
```

Structured output is available for agent and script consumption. Human-readable output remains concise and composable.

## Initial format support

Version 1 targets:

- PNG
- JPEG
- WebP

Later versions may add GIF, SVG, and PDF output. PSD compatibility, CMYK and print-production workflows, video timelines, 3D, built-in real-time collaboration, and a ready-made editor interface are not initial goals.

## Performance philosophy

The engine must be smart across very different workloads rather than optimized for only one canvas size:

- Pixel art from tiny sprites through large variant sets
- General graphics at 2K and 4K resolutions
- Documents containing hundreds of layers
- Batch jobs producing many related outputs

The implementation will use incremental invalidation, cached intermediate results, preview resolution tiers, lazy asset decoding, and large-document tiling where they materially improve a workload. Performance claims will be tied to checked-in benchmark documents and reproducible measurements.

## Roadmap

Development is divided into five phases:

1. Executable document and CLI foundation
2. Browser preview and rendering performance
3. Photoshop-like graphics primitives
4. Headless editor toolkit
5. Spriteform integration and production hardening

See [ROADMAP.md](ROADMAP.md) for the high-level sequence, [Technology Stack](docs/TECH_STACK.md) for implementation choices, and the [phase plans](docs/plans/) for detailed scope and acceptance criteria. The [Website and Documentation Plan](docs/plans/06-website-documentation.md) runs across every implementation phase.

## Contributing

Layered Graphics is intended to be developed in the open. Contribution instructions, governance, licensing details, and compatibility policies will be added with the initial repository scaffold.

During the planning stage, the most valuable contributions are concrete authoring workflows, representative documents, compatibility references, and measurable performance cases.
