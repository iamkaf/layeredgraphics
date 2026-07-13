# Archive: first vertical slice

This completed checkpoint proved that the architecture could produce its own editable project banner end to end.

`examples/readme-banner/build.sh` uses public `lg` commands to create and render a layered icon, embed the image and a reproducible bitmap font, build nested layers, apply operations, update and reorder content, validate and inspect the document, export it, and verify an approved PNG checksum.

The Rust/WASM playground opens and re-renders the same document in a browser. CLI and browser tests protect the workflow.

The current, broader evidence is recorded in [Foundation completion evidence](FOUNDATION_AUDIT.md). Painting, selections, masks, clipping, adjustments, filters, and Spriteform integration remain outside the current engine.
