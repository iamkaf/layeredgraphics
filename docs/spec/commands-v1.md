# Command protocol v1 experimental contract

Status: implemented experimental contract. Commands use camel-case JSON and execute as atomic arrays through `lg exec` and the Rust API.

## Envelope

Every command has an `op` discriminator:

```json
[
  { "op": "documentUpdate", "dpi": 144 },
  { "op": "layerUpdate", "id": "hero", "set": { "opacity": 0.7 } }
]
```

`lg exec file.kgfx ops.json` accepts one command or an array. `-` reads standard input. An array increments the document revision once if every command succeeds and the resulting document validates.

## Document update

```json
{ "op": "documentUpdate", "width": 1200, "height": 630, "dpi": 144 }
```

Every field is optional, but supplied values must produce a valid canvas.

## Layer add

```json
{
  "op": "layerAdd",
  "layer": {
    "id": "background",
    "name": "Background",
    "visible": true,
    "opacity": 1,
    "blendMode": "normal",
    "transform": { "x": 0, "y": 0, "scaleX": 1, "scaleY": 1 },
    "type": "fill",
    "width": 1200,
    "height": 630,
    "color": [9, 13, 24, 255]
  },
  "parentId": null,
  "index": 0
}
```

`parentId` targets a group. An omitted index appends the layer at the top of the destination stack.

## Layer update

```json
{
  "op": "layerUpdate",
  "id": "hero",
  "set": {
    "name": "Hero image",
    "visible": true,
    "opacity": 0.7,
    "blendMode": "multiply",
    "x": 80,
    "y": 40,
    "scaleX": 1.25,
    "scaleY": 1.25
  }
}
```

Kind-specific properties include `assetId`, `text`, `fontAssetId`, `fontSize`, `color`, `width`, and `height`. Supplying a property incompatible with the target layer kind rejects the transaction.

## Layer remove

```json
{ "op": "layerRemove", "id": "hero" }
```

Removing a group removes its subtree.

## Layer move

```json
{ "op": "layerMove", "id": "hero", "parentId": null, "above": "background" }
```

Use exactly one of `to`, `above`, or `below`. If none is supplied, the layer moves to the top of the destination. Reference IDs must be direct children of that destination.

## Asset add

```json
{
  "op": "assetAdd",
  "id": "hero-image",
  "mediaType": "image/png",
  "bytesBase64": "…",
  "originalName": "hero.png"
}
```

The reducer computes content integrity metadata. The CLI's `asset add` command reads raw file bytes and constructs this operation.

## Asset remove

```json
{ "op": "assetRemove", "id": "hero-image" }
```

Removing an asset still referenced by a layer causes final transaction validation to fail, leaving the document unchanged.

## Result

Successful execution returns:

```json
{
  "revision": 4,
  "applied": 2,
  "changedLayers": ["hero"],
  "changedAssets": []
}
```

Errors identify the zero-based failing command or final validation failure. CLI machine output is written to standard output and diagnostics to standard error.
