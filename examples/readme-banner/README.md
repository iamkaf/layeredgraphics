# README banner workflow

This fixture builds Layered Graphics' own banner using only the public `lg` CLI and the checked-in bitmap font asset.

```bash
./examples/readme-banner/build.sh
```

The workflow creates a small icon composition first, renders it to PNG, embeds that PNG into the banner document, applies individual commands and an operation array, validates and inspects the result, renders the final banner, verifies its approved SHA-256 fixture, and copies the image to the landing site.

Generated documents and intermediate images live under `examples/readme-banner/output/`.
