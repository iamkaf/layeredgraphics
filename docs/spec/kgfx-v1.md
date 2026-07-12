# `.kgfx` v1 experimental format

Status: implemented experimental contract. Compatibility is not guaranteed until the first stable release.

## Container

A `.kgfx` document is a ZIP-compatible archive containing:

```text
manifest.json
assets/<sha256>
```

Callers must use the public APIs rather than depending on physical archive layout. Readers enforce bounded entry sizes, safe asset paths, declared byte lengths, and SHA-256 integrity.

The CLI saves through a temporary archive and atomically replaces the destination after the new document is complete.

## Manifest

The manifest uses camel-case JSON keys:

```json
{
  "schemaVersion": 1,
  "id": "stable-document-id",
  "revision": 3,
  "canvas": { "width": 1200, "height": 630, "dpi": 144 },
  "layers": [],
  "assets": {},
  "extensions": {}
}
```

Layer arrays are ordered bottom to top. A group contains a `children` array with the same ordering rule.

Unknown namespaced values under `extensions` are preserved across load, command execution, and save.

## Common layer fields

```json
{
  "id": "hero",
  "name": "Hero",
  "visible": true,
  "opacity": 1,
  "blendMode": "normal",
  "transform": { "x": 0, "y": 0, "scaleX": 1, "scaleY": 1 }
}
```

Supported blend modes are `normal` and `multiply`. Opacity is in the inclusive range `0..1`. Scale values must be finite and nonzero; negative values flip content.

## Layer kinds

### Image

```json
{ "type": "image", "assetId": "hero-image" }
```

Milestone 1 decodes PNG image assets.

### Fill

```json
{ "type": "fill", "width": 1200, "height": 630, "color": [9, 13, 24, 255] }
```

Colors are non-premultiplied 8-bit RGBA arrays.

### Text

```json
{
  "type": "text",
  "text": "Layered Graphics",
  "fontAssetId": "display-font",
  "fontSize": 70,
  "color": [244, 247, 255, 255]
}
```

The authoritative renderer accepts embedded TrueType/OpenType fonts through `fontdue` and the small `LGF1` bitmap fixture format used to make the repository banner completely reproducible.

### Group

```json
{ "type": "group", "children": [] }
```

Children composite into an isolated transparent surface before group transform, opacity, and blend mode are applied.

## Assets

Embedded assets are keyed by stable logical ID:

```json
{
  "hero-image": {
    "id": "hero-image",
    "mediaType": "image/png",
    "byteLength": 2049,
    "sha256": "…",
    "source": { "type": "embedded", "path": "assets/…" },
    "originalName": "hero.png"
  }
}
```

Several logical IDs may share content-addressed bytes. Asset filenames are descriptive metadata and do not control identity.

Linked asset records are reserved in the model but host resolution is not part of Milestone 1.

## Revisions

Every nonempty successful transaction increments `revision` once. A rejected transaction does not mutate state or revision.

## Resource limits

The initial reader limits a manifest entry to 16 MiB and each embedded asset to 512 MiB. Additional total-document, layer-depth, and decoded-image limits will be introduced before stable release.
