---
title: Browser preview sessions
description: Run retained document work in a worker and present with WebGPU or Canvas2D.
---

`@layered-graphics/browser` keeps document execution and preview coordination outside the main thread.

```ts
import { BrowserRenderSession, moduleWorkerFactory } from "@layered-graphics/browser";

const session = await BrowserRenderSession.open(
  moduleWorkerFactory(),
  kgfxBytes,
  document.querySelector("canvas"),
);

await session.execute({ op: "layerUpdate", id: "hero", set: { x: 120 } });
await session.preview("interactive", { includePixels: false });
```

The transferred canvas presents through WebGPU when supported and Canvas2D otherwise. Without a canvas, previews return transferable RGBA. Normal preview edits never require encoded-image round trips.

Quality intents are `interactive`, `preview`, `refined`, and `authoritative`. Newer queued previews supersede old ones, idle input schedules refinement, and every frame reports timing, dirty reasons, cache activity and memory estimates. Viewports, cancellation, warm bounded batches and canonical-snapshot worker recovery are part of the session API.

[Open the executable preview proof](../../playground/) or read the repository's [full rendering contract](https://github.com/iamkaf/layeredgraphics/blob/main/docs/BROWSER_RENDERING.md).
