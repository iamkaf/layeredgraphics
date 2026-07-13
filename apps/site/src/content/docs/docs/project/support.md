---
title: Runtime support
description: Experimental browser, Node, native and package support targets.
---

The prerelease supports the current and previous Node LTS lines (22 and 24) and the latest two stable Chromium, Firefox and Safari releases. The browser worker needs module workers and WebAssembly; direct presentation uses OffscreenCanvas. WebGPU is optional because Canvas2D presents authoritative WASM pixels as fallback.

Packages are split by purpose: `@layered-graphics/core` for WASM document APIs, `@layered-graphics/browser` for retained workers, `@layered-graphics/node` for native N-API and `lg` for native file workflows.

Read the complete [support and resource policy](https://github.com/iamkaf/layeredgraphics/blob/main/docs/API_SUPPORT.md).
