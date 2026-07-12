# Cross-Phase Plan: Website and Documentation

## Objective

Build a single public site that explains why Layered Graphics exists, proves what it can do, and gives humans and agents everything required to adopt it successfully.

The site is a product surface, not a release-week packaging task. It evolves alongside the engine so examples, APIs, compatibility claims, and workflows remain testable throughout development.

## Technology choice

The site lives at `apps/site` in the pnpm workspace and uses:

- Astro for custom landing pages and static generation
- Starlight for documentation layouts and navigation
- Markdown and MDX for authored content
- Pagefind, provided by Starlight, for static full-text search
- Small client-side islands for interactive Layered Graphics examples

This combination supports a distinctive landing page and a conventional, searchable documentation experience in one deployment. It avoids requiring an application server or a separate documentation repository.

The site consumes published-style workspace packages during development. It must not reach into private source modules that external consumers cannot import.

## Audiences

### Application developers

They need to understand the engine boundaries, integrate the browser or Node runtime, build an editor, resolve assets, handle worker sessions, and ship reliably.

### Automation and agent authors

They need CLI installation, stable operation schemas, structured output, examples using standard input, validation behavior, inspection capabilities, and complete headless workflows.

### Contributors

They need architecture, workspace orientation, test commands, specification processes, compatibility policy, benchmark methodology, and contribution expectations.

### Evaluators

They need a fast explanation, compelling visual proof, honest project status, browser compatibility, performance evidence, licensing, and a short path to trying the engine.

## Information architecture

### Landing surface

The custom homepage should communicate the project within one screen:

- “Embed Photoshop-like graphics authoring in any app” positioning
- Browser-first, headless, agent-friendly, and FOSS attributes
- A visual layered-composition demonstration
- Primary calls to action: Get started, Try the playground, View on GitHub
- Installation snippets for JavaScript and CLI users

Supporting marketing pages:

- Features and product boundaries
- Use cases: embedded editor, agent-authored graphics, scripted production
- Performance and architecture overview
- Examples gallery
- Roadmap and project status
- FOSS and community information

Claims should link to documentation, fixtures, benchmarks, or compatibility notes rather than relying on unqualified marketing language.

### Documentation surface

Organize docs by user intent:

1. **Start**
   - Project status and installation
   - Five-minute browser composition
   - Five-minute CLI composition
   - Render and inspect a `.kgfx` file
2. **Concepts**
   - Documents and assets
   - Layers and compositing
   - Commands, transactions, and history
   - Preview versus authoritative rendering
   - Extensions and application models
3. **Guides**
   - Browser worker integration
   - Node and batch rendering
   - Agent and CLI workflows
   - Building an editor
   - Fonts, linked assets, and portability
   - Performance and memory tuning
4. **Graphics behavior**
   - Layers, groups, masks, and clipping
   - Blend modes
   - Text, shapes, and fills
   - Filters and adjustments
   - Selections, painting, and raster operations
   - Photoshop compatibility and known differences
5. **Reference**
   - JavaScript API
   - Command schemas
   - CLI commands and exit codes
   - `.kgfx` format
   - Diagnostics
   - Browser and runtime support
6. **Project**
   - Roadmap
   - Architecture
   - Contributing
   - Security and governance
   - Changelog and migrations

## Landing-page experience

The homepage should demonstrate the engine rather than only describe it. Its hero can show a layered banner with a compact operation sequence and immediate rendered result.

Progressive enhancement rules:

- The page remains complete and legible without JavaScript.
- The initial page load does not require engine WASM or WebGPU.
- The interactive demo loads on intent or when it becomes visible.
- A static image or video-free visual provides the initial proof state.
- Reduced-motion and keyboard users receive equivalent content and controls.
- Unsupported browsers see the authoritative-rendered result and clear capability messaging.

The landing page should not imitate a full Photoshop interface. It should emphasize embeddability, document structure, and code/command-driven creation.

## Interactive examples

Examples are small, focused, and reproducible:

- Add, update, move, and remove a layer
- Compare interactive and refined preview tiers
- Toggle clipping, masks, and blend modes
- Edit text and shapes without rasterizing
- Apply an operation array and inspect the resulting revision
- Generate related thumbnails from shared assets
- Use transform, snapping, selection, and paint controllers

Every interactive example includes:

- A static fallback
- Source code or commands
- Reset behavior
- Capability and error reporting
- A link to the relevant guide and reference
- A fixture that can run outside the site

The full playground may be a dedicated route or sibling application, but it shares public packages and design tokens with the site.

## API and schema documentation

Reference content should be generated where automation improves correctness, then surrounded by authored explanations.

Generate:

- TypeScript public API signatures
- Command variants and field definitions
- Diagnostic codes and severity
- CLI command/options tables
- Document manifest and extension schemas

Author manually:

- Mental models and workflow guidance
- Transaction and history examples
- Performance recommendations
- Compatibility explanations
- Migration narratives
- Common failure recovery

Generated reference output must be deterministic and checked for drift in continuous integration. The documentation build should fail when public commands or APIs change without updated reference artifacts.

## Executable documentation

Code examples are part of the test suite:

- TypeScript examples typecheck against workspace packages.
- CLI examples execute against the built `lg` artifact where practical.
- Operation arrays validate against the current schema.
- Rendered examples compare against approved fixtures.
- Referenced `.kgfx` downloads open and validate.

Long tutorials may extract tested source files rather than duplicating fragile snippets. Displayed output, IDs, and diagnostics should remain deterministic.

## Search and agent access

Human search uses the site's static Pagefind index. Pages provide descriptive titles, summaries, headings, and stable canonical URLs.

Agent-friendly delivery includes:

- Semantic server-rendered HTML
- Useful page text without client rendering
- Stable heading anchors
- A machine-readable documentation index or sitemap
- Direct links to command and schema JSON artifacts
- Plain Markdown sources in the repository
- An `llms.txt`-style entry point if the convention remains useful at launch

The public repository remains the source of truth; the site is its rendered, release-aware presentation.

## Versions and releases

Before the first stable release, docs may follow the latest prerelease with clear status banners. Stable releases require:

- Documentation matching the currently selected stable version
- Archived documentation for supported older major versions
- Version displayed in reference and guide pages
- Migration guide and changelog links
- Examples pinned to the documented package version
- Preview/main documentation clearly separated from stable documentation

The exact versioning mechanism should remain simple until there is more than one supported stable line. Avoid copying the entire content tree prematurely.

## Visual system

The site needs a recognizable identity without diverging from documentation usability:

- Shared color, type, spacing, radius, and code tokens
- Light and dark themes
- High-contrast focus and interactive states
- Layered/transparency motifs used with restraint
- Engine-produced graphics used as real product proof
- Responsive behavior from small phones through wide technical-reference layouts

Starlight's accessible documentation structure is the baseline. Custom components and overrides must preserve keyboard navigation, readable line lengths, heading hierarchy, and theme behavior.

## SEO and distribution

The static build includes:

- Page titles and descriptions
- Canonical URLs
- Social preview metadata and engine-generated social images
- Sitemap and robots policy
- Structured data where it truthfully describes software or documentation
- Redirects for renamed high-value pages
- A lightweight privacy-respecting analytics option only if the project needs it

Deployment remains host-neutral. Pull requests should receive preview deployments when the selected provider supports them.

## Cross-phase delivery

### During Phase 1

- Scaffold `apps/site` with Astro and Starlight.
- Establish design tokens, navigation, accessibility baseline, link checking, and deployment preview.
- Publish the landing page, project status, getting started, CLI guide, document concepts, and initial reference.
- Make the banner workflow executable from its documentation.

### During Phase 2

- Add a lazy-loaded browser preview example.
- Publish rendering architecture, quality tiers, capability support, benchmark methodology, and results.
- Document worker lifecycle, cancellation, recovery, and fallback behavior.

### During Phase 3

- Add primitive guides and combination examples.
- Publish Photoshop compatibility tables and known differences.
- Expand generated command and API reference.
- Turn the polished banner into a guided agent workflow and downloadable example.

### During Phase 4

- Publish the afternoon editor tutorial.
- Add controller, keyboard, clipboard, and accessibility guides.
- Provide a progressively enhanced editor playground using only public APIs.

### During Phase 5

- Add Spriteform integration and batch-rendering case studies.
- Publish stable support, versioning, migration, security, and governance pages.
- Complete release-aware docs, package links, redirects, and launch-quality landing content.

## Testing and quality gates

Every site build should check:

- TypeScript and Astro correctness
- Internal and important external links
- Markdown/MDX frontmatter and heading structure
- Code example typechecking and command validation
- Generated-reference drift
- Accessibility of primary templates and interactive examples
- HTML validity where tooling is reliable
- Bundle and page-weight budgets
- No accidental eager loading of the engine on static documentation pages
- Search index generation
- Sitemap and canonical URL correctness

Browser tests cover navigation, theme selection, mobile layout, keyboard use, search, copy buttons, interactive-example fallback, and worker/GPU capability errors.

## Performance budgets

The marketing and documentation surface must remain fast independently of engine size:

- Static content pages ship minimal client JavaScript.
- Engine WASM, GPU code, and example assets load only for interactive examples.
- Images use appropriate formats, dimensions, and lazy loading.
- Fonts are limited, subset where practical, and do not block readable fallback text.
- Page-weight and Core Web Vitals targets are recorded in site configuration before public launch.

Interactive playground budgets are tracked separately from ordinary documentation pages.

## Deliverables

- `apps/site` Astro/Starlight application
- Custom production landing page
- Documentation information architecture and navigation
- Shared site and example design tokens
- Task-oriented guides and generated reference pipeline
- Pagefind search and machine-readable site index
- Interactive examples with static fallbacks
- Deployment and preview configuration
- Link, example, accessibility, and page-weight checks
- Versioning, redirects, and release documentation policy

## Exit criteria

The cross-phase plan is complete for v1 when:

- A new visitor can understand the product and run a successful example without repository archaeology.
- Browser, Node, CLI, agent, and editor-toolkit paths each have a tested getting-started experience.
- Public API, command, CLI, diagnostic, and `.kgfx` reference matches release artifacts.
- Search, canonical links, sitemap, version labels, and migration navigation work in production.
- Interactive examples use only public packages and degrade cleanly without WebGPU or JavaScript.
- Primary pages pass the project's accessibility and performance gates.
- Documentation changes are required and tested alongside public behavior changes.
- The deployed site can be reproduced from this monorepo without private content or manual build steps.

## Deferred

- A headless CMS
- Server-rendered accounts or community features
- Localization before contributor demand and maintenance capacity exist
- Maintaining many historical documentation versions before multiple stable lines exist
- A full browser editor disguised as the landing page
