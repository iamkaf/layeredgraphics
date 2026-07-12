---
title: Layered Graphics
description: Headless Photoshop-like graphics authoring for applications, scripts, and agents.
---

Layered Graphics is a FOSS, browser-first engine for creating and editing portable layered graphics without adopting a prescribed editor interface.

> **Milestone status:** The engine currently implements the first executable vertical slice: `.kgfx` documents, embedded assets, basic image/fill/text/group layers, validated commands, a native CLI, authoritative PNG output, and browser rendering through Rust/WASM.

## Build the project banner

From the repository root:

```bash
./examples/readme-banner/build.sh
```

This produces an editable document and PNG using only public CLI operations, validates the document, emits structured inspection data, and verifies the approved visual fixture.

## Choose a path

- Continue with the [CLI quickstart](/docs/cli/).
- Learn about [documents and commands](/docs/concepts/documents/).
- Run the [browser rendering proof](/playground/).
