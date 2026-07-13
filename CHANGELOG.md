# Changelog

All notable changes are recorded here. The project follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and intends to use semantic versioning after its public API stabilizes.

## Unreleased

### Changed

- Updated the renderer, archive, and hashing foundations to `tiny-skia` 0.12, `zip` 8.6, and `sha2` 0.11 without changing document integrity hashes or rendered output
- Raised the minimum supported Rust version from 1.85 to 1.88
- Updated CI, Pages, and release workflows to the current Node.js 24-based GitHub Actions
- Added explicit TypeScript 6 support to Astro diagnostics while deferring TypeScript 7 until Astro supports its compiler API

### Available

- Versioned `.kgfx` documents with embedded and linked assets
- Atomic commands, history, changesets, diffing, inspection, extensions, and migration
- Image, fill, bitmap-text, and group layers with basic compositing
- PNG, JPEG, and WebP export
- Rust, native CLI, browser WASM, browser-worker, and native Node surfaces
- Retained WebGPU/Canvas2D browser previews and bounded warm batches
