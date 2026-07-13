# @layered-graphics/core

The browser/WASM document and authoritative rendering API for [Layered Graphics](https://github.com/iamkaf/layeredgraphics).

```bash
npm install @layered-graphics/core
```

```ts
import { GraphicsDocument, initialize } from "@layered-graphics/core";

await initialize();
const document = await GraphicsDocument.create({ width: 640, height: 360 });
document.layers.add({
  id: "background",
  type: "fill",
  width: 640,
  height: 360,
  color: [9, 13, 24, 255],
});
const png = document.render("png");
document.close();
```

This package is experimental alpha software. See the [documentation](https://iamkaf.github.io/layeredgraphics/docs/) and [support policy](https://github.com/iamkaf/layeredgraphics/blob/main/docs/API_SUPPORT.md).
