---
title: CLI quickstart
description: Create, inspect, and render a layered document with lg.
---

The native `lg` executable is an authoring interface for humans, scripts, and agents.

```bash
cargo build -p lg-cli
alias lg="$PWD/target/debug/lg"
```

Create a document and add a background:

```bash
lg new banner.kgfx --width 1200 --height 630 --dpi 144
lg layer add banner.kgfx \
  --type fill --id background \
  --width 1200 --height 630 --color '#090d18ff'
```

Embed an image and add it as a layer:

```bash
lg asset add banner.kgfx --id hero ./hero.png
lg layer add banner.kgfx \
  --type image --id hero-layer --asset-id hero \
  --x 80 --y 80 --scale-x 1.25 --scale-y 1.25
```

Inspect, validate, and render:

```bash
lg layer ls banner.kgfx
lg inspect banner.kgfx --json
lg validate banner.kgfx
lg render banner.kgfx -o banner.png --format png --sampling smooth
```

Use `lg exec file.kgfx ops.json` or pipe an operation array to `lg exec file.kgfx -` for atomic machine-authored transactions.

Linked assets, extensions, diffs and retained watch workflows are also available:

```bash
lg asset add banner.kgfx --id workspace-hero ./hero.png --linked --reference assets://hero
lg asset relink banner.kgfx workspace-hero ./updated.png --reference assets://hero-v2
lg extension set banner.kgfx com.example.spriteform '{"variant":"night"}'
lg diff original.kgfx banner.kgfx > changes.ops.json
lg watch banner.kgfx --ops changes.ops.json --render preview.webp
```

`lg inspect --pixels` opts into visible-content analysis. `lg schema document` and `lg schema commands` emit the generated contracts. `lg validate` accepts both `.kgfx` and standalone operation JSON.
