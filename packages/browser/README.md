# @layered-graphics/browser

Worker-owned retained previews for [Layered Graphics](https://github.com/iamkaf/layeredgraphics).

```bash
npm install @layered-graphics/core @layered-graphics/browser
```

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

The package uses module workers, WebAssembly, and optional WebGPU presentation with Canvas2D fallback. It is experimental alpha software.
