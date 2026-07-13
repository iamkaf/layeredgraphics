---
title: Layered Graphics
description: Headless Photoshop-like graphics authoring for applications, scripts, and agents.
---

Layered Graphics is a FOSS, browser-first engine for creating and editing portable layered graphics without adopting a prescribed editor interface.

> **Status:** Phases 1 and 2 are complete: executable `.kgfx` documents, command/history/inspection APIs, embedded and linked assets, native CLI/Node, PNG/JPEG/WebP exports, and worker-owned retained previews with WebGPU/Canvas2D presentation.

## Build the project banner

From the repository root:

```bash
./examples/readme-banner/build.sh
```

This produces an editable document and PNG using only public CLI operations, validates the document, emits structured inspection data, and verifies the approved visual fixture.

## Choose a path

- Continue with the [CLI quickstart](/docs/cli/).
- Learn about [documents and commands](/docs/concepts/documents/).
- Build a [worker-backed browser preview](/docs/browser/).
- Review the [benchmark method and initial budgets](/docs/project/benchmarks/).
- Run the [live rendering proof](/playground/).
