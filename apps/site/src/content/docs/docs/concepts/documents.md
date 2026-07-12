---
title: Documents and commands
description: Canonical state, embedded assets, and atomic operations.
---

A `.kgfx` file contains a versioned JSON manifest and content-addressed embedded assets. Image and font layers refer to stable asset IDs rather than filesystem paths.

Every mutation is a command. CLI subcommands and the future imperative SDK compile to the same command protocol, so user gestures, scripts, and agents cannot create competing document behavior.

```json
[
  {
    "op": "layerUpdate",
    "id": "hero",
    "set": { "opacity": 0.7, "blendMode": "multiply" }
  }
]
```

An operation array executes atomically. If a command is invalid or the resulting document fails validation, the transaction leaves the original file unchanged.
