# `.kgfx` v1 experimental format

Status: implemented experimental contract. Compatibility is not guaranteed before stable v1. The generated manifest schema is [`spec/document/v1.schema.json`](../../spec/document/v1.schema.json).

## Container

A `.kgfx` document is a ZIP-compatible archive containing `manifest.json` and content-addressed `assets/<sha256>` entries. Callers use public APIs rather than archive layout. CLI persistence writes, flushes and syncs a temporary archive, then atomically replaces the destination.

## Manifest

```json
{
  "schemaVersion": 1,
  "id": "stable-document-id",
  "revision": 3,
  "canvas": { "width": 1200, "height": 630, "dpi": 144, "color": [0, 0, 0, 0] },
  "layers": [],
  "assets": {},
  "extensions": {}
}
```

Layer arrays are bottom-to-top. Groups recursively contain `children` with the same rule. Unknown reverse-domain extension values survive load, mutation and save.

Common layer properties are stable ID, name, visibility, opacity, `normal`/`multiply` blend mode and a non-destructive x/y/scale transform. Negative scale flips. Initial kinds are image, solid fill and group; reproducible text is already available as an additive primitive. Groups composite to an isolated surface before group properties apply.

The reference renderer decodes supported PNG/JPEG/WebP assets and TrueType/OpenType or fixture bitmap fonts. Colors are straight 8-bit RGBA. Reference output supports transparent/caller backgrounds, scale, nearest/smooth sampling and PNG/JPEG/WebP encoding.

## Assets

```json
{
  "hero-image": {
    "id": "hero-image",
    "mediaType": "image/png",
    "byteLength": 2049,
    "sha256": "…",
    "source": { "type": "embedded", "path": "assets/…" },
    "originalName": "hero.png",
    "author": { "tool": "spriteform" }
  }
}
```

Stable logical IDs are independent of filenames and content hashes. Identical bytes deduplicate physical payloads without merging logical IDs. Linked sources use `{ "type": "linked", "reference": "opaque-host-reference" }`; the core never assumes path/URL semantics. Hosts resolve files, URLs, memory or browser storage and provide verified bytes. CLI resolves relative references from the document directory.

## Revisions and migration

Every nonempty successful transaction increments revision once; rejection does not mutate. Schema 0 migrates legacy `canvas.w`/`canvas.h`, supplies DPI, transparent color and collection defaults, and becomes schema 1. The historical fixture is under `fixtures/migrations`. Future schemas fail explicitly.

## Resource and integrity limits

The reader limits manifest JSON to 16 MiB, each embedded asset to 512 MiB, total embedded bytes to 1 GiB and asset count to 10,000. Documents allow at most 10,000 layers, depth 128, 32,768 pixels per dimension and 268,435,456 canvas pixels. Transactions allow 10,000 commands. Embedded paths must be relative under `assets/`; malformed archives, missing entries, mismatched declarations, unsupported versions and SHA-256 failures are rejected before rendering.
