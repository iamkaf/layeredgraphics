# Headless editor toolkit plan

## Objective

Provide reusable authoring behavior that lets an application build a credible browser editor without adopting a prescribed interface, component library, or application state model.

The toolkit translates user intent into document commands and presentation overlays. It does not become a second document engine.

## Starting baseline and dependency

The browser runtime already owns documents in a module worker, exposes retained previews and invalidation metrics, recovers from worker/device loss, and supports atomic history. The toolkit consumes those APIs; it does not introduce another render session or state store.

Implementation begins after transform, selection, paint, bounds, and snapping query contracts are stable. Viewport and input normalization may be prototyped earlier, but public controllers target shipped commands rather than speculative operations.

## Success criterion

A developer familiar with a modern web framework can build a basic browser editor in an afternoon with:

- Canvas display, zoom, and pan
- Layer selection and hit testing
- Transform handles
- Painting and erasing
- Layer creation, ordering, and visibility
- Undo and redo
- Keyboard shortcuts and clipboard operations
- Export

The resulting editor uses application-owned components and styles.

## Design boundaries

### The toolkit owns behavior

It may provide:

- Framework-neutral controllers and state machines
- Input normalization
- Geometry and overlay descriptions
- Commands and transaction boundaries
- Accessibility-oriented action descriptions
- Optional framework bindings

### The application owns presentation

It owns:

- Panels, buttons, menus, dialogs, and icons
- CSS and visual design
- Product navigation and persistence policy
- Application-specific models and permissions
- How overlays are drawn, when multiple presentation options exist

### The document remains canonical

Committed edits flow through document commands. Controllers may hold ephemeral gesture state such as pointer capture, drag origin, transform preview, or an in-progress brush stroke. Cancelling a gesture discards that state; committing it creates a transaction.

## Scope

### Viewport controller

Support:

- Pan and zoom around a focal point
- Fit canvas, fit selection, actual pixels, and configured zoom limits
- Coordinate conversion among client, viewport, canvas, and layer spaces
- Device-pixel-ratio changes
- Pixel-grid and nearest-neighbor behavior
- Viewport resize and preview-quality intent

Viewport state is workspace state, not portable document content, unless an application explicitly persists it in its namespace.

### Hit testing and selection

Provide:

- Topmost visible-content hit testing
- Bounds-based fallback for non-pixel sources
- Cycling through overlapping candidates
- Single and multiple layer selection
- Selection through or within groups according to policy
- Locked, hidden, clipped, and masked-layer behavior
- Structured hit results explaining the candidate and coordinate

Applications can customize selection policy without replacing geometry or render-aware hit testing.

### Transform controller

Support move, resize, rotate, and flip for one or many selected layers. It provides:

- Handle geometry independent of rendering framework
- Pointer, keyboard, and numeric-input operations
- Modifier policies for aspect ratio, center transforms, and angle constraints
- Live preview state
- Commit and cancel
- Snapping integration
- Clear handling of mixed parents and coordinate spaces

The controller emits one logical transaction for a completed gesture.

### Guides, snapping, alignment, and distribution

Provide configurable candidates for:

- Canvas edges and centers
- Layer visible and transformed bounds
- Selection bounds
- User guides
- Pixel boundaries
- Equal gaps and common alignment lines

Snapping output includes the chosen candidates and visual guide geometry. Applications decide how to draw the feedback.

Alignment and distribution actions use the graphics-plan helpers and produce ordinary transform commands.

### Tool controller model

Define a common lifecycle for tools:

- Activation and deactivation
- Pointer, keyboard, and cancellation input
- Cursor and overlay intent
- Ephemeral state updates
- Transaction commit
- Diagnostics and capability requirements

Initial tool controllers:

- Move/transform
- Layer select
- Marquee and lasso selections
- Brush and eraser
- Bucket fill
- Basic shape creation
- Text placement and editing handoff
- Eyedropper
- Hand and zoom

Applications may register their own tools using the same lifecycle.

### Input normalization

Normalize mouse, touch, pen, and keyboard input into tool events while preserving useful data:

- Pointer identity and capture
- Pressure and tilt
- Modifier keys
- Coalesced points where available
- Gesture cancellation
- Platform-aware primary shortcuts

The core does not assume React synthetic events or a specific DOM tree.

### Keyboard command system

Provide semantic actions rather than hard-coded UI shortcuts:

- Undo and redo
- Delete, duplicate, group, and ungroup
- Select all, deselect, and invert selection
- Copy, cut, paste, and paste in place
- Nudge and larger-step nudge
- Tool activation
- Zoom actions

Default Photoshop-like bindings are available and overridable. Applications can query conflicts and present shortcut descriptions.

### Clipboard behavior

Define internal and system clipboard representations for:

- Layer subtrees with referenced assets
- Selected raster pixels
- Plain image data
- Text where applicable

Cross-document paste remaps IDs and imports required embedded assets atomically. Unsupported system clipboard capabilities fail without losing the internal clipboard payload.

### History integration

Expose document history in editor-friendly terms:

- Human-readable transaction labels
- Undo and redo availability
- Gesture coalescing
- Saved versus dirty revision markers
- Optional history inspection

The toolkit labels and groups commands but does not maintain a separate undo stack.

### Framework bindings and examples

The framework-neutral package is the contract. Optional bindings may provide:

- Lifecycle helpers for worker sessions and subscriptions
- Reactive snapshots
- Canvas or GPU-surface attachment helpers
- Controller hooks

An example editor demonstrates one possible UI while making clear that its component structure is not required.

## Deliverables

- Framework-neutral editor session and controller packages
- Viewport and coordinate system
- Hit testing and selection policies
- Transform and snapping controllers
- Initial tool-controller set
- Keyboard action and shortcut mapping
- Clipboard interchange and asset import behavior
- History labels and gesture grouping
- Optional bindings for the first supported web framework
- Minimal example editor and an integration guide
- Interaction, accessibility, and performance test harnesses
- Site-hosted editor tutorial and progressively enhanced interactive playground

## Work sequence

1. Specify coordinate spaces, ephemeral state, and commit boundaries.
2. Implement viewport control and preview-surface attachment.
3. Add hit testing and layer selection policies.
4. Add transform geometry, gesture preview, and command commit.
5. Integrate snapping, guides, alignment, and distribution.
6. Define the generic tool lifecycle and input normalization.
7. Implement selection, paint, fill, shape, and navigation tools.
8. Add semantic keyboard actions and default shortcut maps.
9. Implement clipboard behavior and cross-document asset transfer.
10. Add optional bindings, example editor, and the afternoon-build guide.

## Testing strategy

### Controller tests

- Feed normalized event sequences without a browser UI and assert state, overlays, and emitted commands.
- Cover commit, cancel, interruption, tool switching, pointer loss, and worker delay.
- Verify one completed gesture maps to the intended history transaction.

### Geometry tests

- Exercise zoom, pan, device scale, rotation, negative coordinates, nested transforms, and pixel alignment.
- Use property-based tests for coordinate round trips and transform invariants.

### Browser interaction tests

- Exercise mouse, keyboard, touch, and simulated pen input.
- Verify focus and shortcut behavior inside a host application.
- Check that rapid interaction supersedes stale preview work.

### Integration tests

- Build equivalent small editors with the framework-neutral API and the optional binding.
- Paste layers between documents and verify IDs, assets, output, and undo.
- Run the example editor against every supported browser target.

## Accessibility expectations

Although the toolkit does not own UI, it must not force pointer-only interaction. Semantic actions need:

- Keyboard equivalents for core editing operations
- Human-readable names and shortcut descriptions
- Numeric transform paths
- Focus-independent command invocation
- Sufficient state for applications to announce selection, tool, and operation changes

The example editor demonstrates accessible integration rather than claiming accessibility automatically.

## Performance requirements

- Pointer processing and overlay updates remain independent of slow authoritative export work.
- Transform and brush previews use the interactive quality path.
- Coalesced pointer input does not create one history entry per event.
- Hit testing avoids full document readback for routine selection.
- Controller subscriptions do not require copying the full document on every pointer move.

## Exit criteria

This plan is complete when:

- The documented afternoon editor can be built using only public packages.
- Core tools support commit, cancel, undo, redo, and worker-backed previews.
- Controllers are tested without reliance on a specific UI framework.
- Mouse, keyboard, touch, and pen paths are covered where browsers expose them.
- Cross-document clipboard operations preserve appearance and remain undoable.
- Default shortcuts and Photoshop-like interaction choices are documented.
- The example application contains no privileged access to engine internals.

## Deferred

- A production-ready styled editor
- Application navigation, authentication, storage, and collaboration UI
- Product-specific inspectors and asset libraries
- A requirement to support every frontend framework with first-party bindings
