#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FIXTURE="$ROOT/examples/readme-banner"
OUTPUT="$FIXTURE/output"
LG="$ROOT/target/debug/lg"

mkdir -p "$OUTPUT" "$ROOT/apps/site/public"
rm -f "$OUTPUT/icon.kgfx" "$OUTPUT/icon.png" "$OUTPUT/banner.kgfx" "$OUTPUT/banner.png" "$OUTPUT/inspect.json"

cargo build --quiet --manifest-path "$ROOT/Cargo.toml" -p layered-graphics-cli

"$LG" new "$OUTPUT/icon.kgfx" --id lg-readme-icon-v1 --width 256 --height 256 --dpi 72
"$LG" layer add "$OUTPUT/icon.kgfx" --type group --id icon-shapes --name "Icon shapes"
"$LG" layer add "$OUTPUT/icon.kgfx" --type fill --id icon-back --name "Purple block" --parent icon-shapes --width 150 --height 150 --color '#7548ffff' --x 20 --y 20
"$LG" layer add "$OUTPUT/icon.kgfx" --type fill --id icon-front --name "Cyan block" --parent icon-shapes --width 140 --height 140 --color '#34e1c5e8' --x 96 --y 96 --blend multiply
"$LG" layer add "$OUTPUT/icon.kgfx" --type fill --id icon-cut-v --name "Vertical light" --parent icon-shapes --width 24 --height 170 --color '#f4f7ffff' --x 116 --y 42
"$LG" layer add "$OUTPUT/icon.kgfx" --type fill --id icon-cut-h --name "Horizontal light" --parent icon-shapes --width 170 --height 24 --color '#f4f7ffff' --x 42 --y 116
"$LG" validate "$OUTPUT/icon.kgfx"
"$LG" render "$OUTPUT/icon.kgfx" -o "$OUTPUT/icon.png"

"$LG" new "$OUTPUT/banner.kgfx" --id lg-readme-banner-v1 --width 1200 --height 630 --dpi 144
"$LG" asset add "$OUTPUT/banner.kgfx" --id hero "$OUTPUT/icon.png"
"$LG" asset add "$OUTPUT/banner.kgfx" --id display-font "$FIXTURE/assets/layered.lgf" --media-type application/x-layered-font

"$LG" layer add "$OUTPUT/banner.kgfx" --type fill --id background --name "Midnight background" --width 1200 --height 630 --color '#090d18ff'
"$LG" layer add "$OUTPUT/banner.kgfx" --type group --id accents --name "Accent rails"
"$LG" exec "$OUTPUT/banner.kgfx" "$FIXTURE/accents.ops.json"
"$LG" layer add "$OUTPUT/banner.kgfx" --type image --id mark --name "Layered Graphics mark" --asset-id hero --x 92 --y 174 --scale-x 1.08 --scale-y 1.08
"$LG" layer add "$OUTPUT/banner.kgfx" --type group --id copy --name "Copy"
"$LG" layer add "$OUTPUT/banner.kgfx" --type text --id title-layered --name "Layered title" --parent copy --font-asset-id display-font --text "Layered" --font-size 70 --color '#f4f7ffff' --x 460 --y 138
"$LG" layer add "$OUTPUT/banner.kgfx" --type text --id title-graphics --name "Graphics title" --parent copy --font-asset-id display-font --text "Graphics" --font-size 70 --color '#8f6dffff' --x 460 --y 228
"$LG" layer add "$OUTPUT/banner.kgfx" --type text --id subtitle --name "Subtitle" --parent copy --font-asset-id display-font --text "HEADLESS GRAPHICS FOR EVERY APP." --font-size 18 --color '#b7c1d9ff' --x 462 --y 350
"$LG" layer add "$OUTPUT/banner.kgfx" --type text --id stack --name "Technology stack" --parent copy --font-asset-id display-font --text "RUST + WASM + TYPESCRIPT" --font-size 15 --color '#34e1c5ff' --x 462 --y 507

"$LG" layer update "$OUTPUT/banner.kgfx" subtitle --set opacity=0.92
"$LG" layer move "$OUTPUT/banner.kgfx" copy --above mark
"$LG" validate "$OUTPUT/banner.kgfx" --json
"$LG" inspect "$OUTPUT/banner.kgfx" --json > "$OUTPUT/inspect.json"
"$LG" render "$OUTPUT/banner.kgfx" -o "$OUTPUT/banner.png" --json

if [[ -f "$FIXTURE/banner.sha256" ]]; then
  (cd "$OUTPUT" && sha256sum --check "$FIXTURE/banner.sha256")
fi

cp "$OUTPUT/banner.png" "$ROOT/apps/site/public/readme-banner.png"
cp "$OUTPUT/banner.kgfx" "$ROOT/apps/site/public/readme-banner.kgfx"
printf 'Built %s and %s\n' "$OUTPUT/banner.kgfx" "$OUTPUT/banner.png"
