# Command protocol v1 experimental contract

Status: implemented experimental contract. Commands use camel-case JSON and execute as atomic arrays through Rust, JavaScript, worker/native bindings and `lg exec`. The generated normative schema is [`spec/commands/v1.schema.json`](../../spec/commands/v1.schema.json).

## Envelope and transaction behavior

```json
[
  { "op": "documentUpdate", "dpi": 144, "color": [0, 0, 0, 0] },
  { "op": "layerUpdate", "id": "hero", "set": { "opacity": 0.7 } }
]
```

`lg exec file.kgfx ops.json` accepts one command or an array. `-` reads standard input. The whole array validates against a clone and commits once; failure leaves state and revision unchanged. A nonempty successful array increments revision exactly once.

## Command families

`documentUpdate` accepts optional `width`, `height`, `dpi` and initial `color`.

`layerAdd` carries a flattened image, fill, text or group layer plus optional `parentId` and `index`. Layer arrays are bottom-to-top; an omitted index appends at the top. An omitted/empty layer ID is replaced with a UUID and returned in `changedLayers`.

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
  }
}
```

`layerUpdate` patches common properties and compatible kind properties. `layerRemove` removes a subtree. `layerMove` targets an optional group and exactly one of `to`, `above` or `below`; no position moves to the top.

`assetAdd` carries `id`, `mediaType`, base64 bytes and optional `originalName`/structured `author`. Re-adding an ID replaces it. Distinct IDs with identical bytes remain logical aliases while the archive writes one content-addressed payload. `assetRemove` fails final validation while referenced.

```json
{
  "op": "assetLink",
  "id": "hero-image",
  "mediaType": "image/png",
  "reference": "workspace://images/hero",
  "byteLength": 2049,
  "sha256": "…"
}
```

Linked references are opaque. Hosts supply bytes and the engine verifies length/integrity. `assetRelink` changes reference, length and hash while retaining the ID.

`extensionSet` and `extensionRemove` address reverse-domain namespaces such as `com.example.spriteform`. Values are arbitrary JSON; unknown namespaces round-trip.

## Result, invalidation and errors

```json
{
  "fromRevision": 3,
  "revision": 4,
  "applied": 1,
  "changedLayers": ["hero"],
  "changedAssets": [],
  "changes": [{
    "impact": "localPixels",
    "reason": "layerTransformChanged",
    "fullRender": false,
    "layerIds": ["hero"],
    "assetIds": [],
    "regions": [
      { "x": 80, "y": 40, "width": 320, "height": 200 },
      { "x": 120, "y": 40, "width": 320, "height": 200 }
    ]
  }],
  "warnings": []
}
```

Impacts are `metadata`, `localPixels`, `composite`, `asset` and `global`. Transform changes carry old/new bounds. Errors identify the zero-based command (`command N: …`) or final validation. CLI JSON stays on stdout and diagnostics use stderr; runtime failures exit 1 and syntax/usage failures exit 2.

History records successful transaction boundaries and labels. Undo/redo restore snapshots. `lg diff` emits a valid supported-state transform; `lg watch` reapplies operations to a canonical base while retaining render sources.
