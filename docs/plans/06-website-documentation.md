# Website and documentation plan

The public site is a product surface and an executable confidence test. It describes shipped behavior, visualizes how the engine works, and gives developers a short path from evaluation to a running document.

## Current baseline

The Astro/Starlight application provides:

- a custom responsive landing page and searchable documentation shell
- an engine-authored visual showcase with downloadable PNG and editable `.kgfx`
- CLI, document, browser-session, runtime-support, benchmark, technology, and roadmap guides
- a browser proof that initializes WASM, validates and renders `.kgfx`, checks command/imperative equivalence, drives retained worker previews, compares cold output, and exercises invalidation, viewports, batches, and device-loss fallback
- sitemap, metadata, social preview, CI build, and Playwright smoke coverage

## Content rules

1. Describe current behavior in present tense. Roadmap statements live only in the roadmap and implementation plans.
2. Pair important claims with runnable code, a fixture, a benchmark, or a clearly labeled architecture diagram.
3. Treat visual output as documentation: every major primitive gets a small before/after or combination fixture.
4. Label authoritative output, GPU preview, measured data, and conceptual diagrams distinctly.
5. State unsupported features beside capability summaries so “Photoshop-like” never implies complete parity.
6. Keep examples copyable, keyboard-accessible, responsive, and useful without client-side JavaScript.

## Information architecture

- **Landing:** value proposition, editable showcase, current feature matrix, runtime architecture, workflow paths, benchmark chart, boundaries, calls to action.
- **Start:** install/build prerequisites, first CLI document, first browser session, first Node script.
- **Concepts:** documents, commands, assets, layer ordering, history, preview versus export, diagnostics.
- **Reference:** schemas, CLI, TypeScript APIs, compatibility tables, and limits.
- **Guides:** browser worker, batch rendering, agent workflow, linked assets, embedding, and recovery.
- **Project:** roadmap, technology, benchmarks, support, contribution, security, and releases.
- **Playground:** minimal executable proofs, not an implied editor application.

## Visual documentation system

| Claim | Preferred visual |
| --- | --- |
| Data flow or ownership | Mermaid architecture or sequence diagram |
| Supported combinations | compact matrix or table |
| Performance | SVG/HTML chart sourced from checked benchmark JSON |
| Graphics semantics | engine-rendered before/after or layer-stack fixture |
| Interaction behavior | short canvas demo with reduced-motion fallback |
| Portable composition | PNG preview plus downloadable `.kgfx` and operation log |

Generated source art is permitted when its provenance is disclosed. Layout and capability proofs should be built through Layered Graphics wherever current primitives can express them.

## Work attached to every engine slice

Each public capability includes:

- task-oriented guide or reference update
- support-table update
- visual fixture showing interaction with an existing feature
- runnable command/API example checked in CI
- preview-versus-authoritative fidelity note
- benchmark visualization when performance changes materially
- accessible alt text and mobile presentation

## Quality gates

- `pnpm site:build` succeeds with no broken internal links.
- Playwright verifies landing navigation, responsive layout, browser proof, and downloadable showcase assets.
- Metadata includes title, description, canonical URL, Open Graph image, and theme color.
- Core pages remain useful with scripts disabled; interactive proofs show loading and error states.
- Charts expose exact values in text or tables and never rely on color alone.
- Code samples use public APIs and are executed or type-checked where practical.
- The landing page ships minimal JavaScript, optimized raster dimensions, stable layout, and lazy non-hero media.

## Next increments

1. Generate schema/API reference pages from checked contracts.
2. Add visual conformance galleries alongside graphics primitives.
3. Add editor-controller diagrams and a minimal unstyled integration example.
4. Publish Spriteform and batch-production case studies with measured data.
5. Introduce versioned documentation and release notes with the first distributed prerelease.

## Completion signal

The site is ready for v1 when a new evaluator can understand current support, reproduce a visual workflow, choose a runtime, integrate a document, diagnose a failure, and find compatibility and release policy without reading source code.
