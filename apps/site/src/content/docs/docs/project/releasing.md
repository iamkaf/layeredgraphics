---
title: Releasing
description: Unified versions, release PRs, registry publishing, and CLI artifacts.
---

Layered Graphics uses a single version across its Cargo workspace, npm packages, site, CLI binaries, and GitHub release.

The **Prepare release** workflow calculates the requested bump, synchronizes every manifest and lockfile, updates the changelog, and opens a protected release PR. After review and merge, the manually dispatched **Publish release** workflow builds the complete artifact set before publishing anything permanently.

Published artifacts include:

- `@layered-graphics/core` and `@layered-graphics/browser` on npm;
- `layered-graphics` and `layered-graphics-cli` on crates.io;
- Linux, macOS, and Windows `lg` binaries;
- SHA-256 checksums, schemas, and editable examples on GitHub Releases.

The native Node package remains private until its per-platform binary distribution is complete.

Read the repository's [complete release operator guide](https://github.com/iamkaf/layeredgraphics/blob/main/docs/RELEASING.md).
