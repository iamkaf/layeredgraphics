---
title: Technology stack
description: Rust, WebAssembly, TypeScript, WebGPU, Astro, and Starlight.
---

Layered Graphics uses Rust for canonical document and rendering behavior, WebAssembly for browser execution, TypeScript for browser-facing APIs and editor behavior, and a native Rust CLI for headless workflows.

WebGPU will provide retained interactive previews in the next rendering phase. Authoritative CPU rendering remains the output reference.

The landing and documentation site uses Astro and Starlight. See the repository's [`docs/TECH_STACK.md`](https://github.com/iamkaf/layeredgraphics/blob/main/docs/TECH_STACK.md) for package boundaries and dependency policy.
