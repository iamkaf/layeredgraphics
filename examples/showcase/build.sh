#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
EXAMPLE="$ROOT/examples/showcase"
OUTPUT="$EXAMPLE/output"
LG="$ROOT/target/debug/lg"

mkdir -p "$OUTPUT" "$ROOT/apps/site/public"
rm -f "$OUTPUT/showcase.kgfx" "$OUTPUT/showcase.png"

cargo build --quiet --manifest-path "$ROOT/Cargo.toml" -p layered-graphics-cli

"$LG" new "$OUTPUT/showcase.kgfx" --id lg-showcase-v1 --width 1440 --height 960 --dpi 144
"$LG" asset add "$OUTPUT/showcase.kgfx" --id layers-art "$EXAMPLE/assets/layers.png"
"$LG" asset add "$OUTPUT/showcase.kgfx" --id display-font "$ROOT/examples/readme-banner/assets/layered.lgf" --media-type application/x-layered-font

"$LG" layer add "$OUTPUT/showcase.kgfx" --type fill --id background --name "Ink background" --width 1440 --height 960 --color '#070b14ff'
"$LG" layer add "$OUTPUT/showcase.kgfx" --type fill --id violet-glow --name "Violet field" --width 940 --height 840 --color '#3e2469a8' --x 500 --y 0 --blend multiply
"$LG" layer add "$OUTPUT/showcase.kgfx" --type fill --id copy-panel --name "Copy panel" --width 550 --height 760 --color '#0d1424f2' --x 70 --y 100
"$LG" layer add "$OUTPUT/showcase.kgfx" --type image --id hero-art --name "Generated layer study" --asset-id layers-art --x 650 --y 74 --scale-x 0.64 --scale-y 0.64
"$LG" layer add "$OUTPUT/showcase.kgfx" --type fill --id cyan-rail --name "Cyan rail" --width 14 --height 540 --color '#36e3c4ff' --x 70 --y 220
"$LG" layer add "$OUTPUT/showcase.kgfx" --type text --id kicker --name "Kicker" --font-asset-id display-font --text "HEADLESS GRAPHICS ENGINE" --font-size 17 --color '#36e3c4ff' --x 118 --y 162
"$LG" layer add "$OUTPUT/showcase.kgfx" --type text --id title-a --name "Title line one" --font-asset-id display-font --text "A CANVAS" --font-size 62 --color '#f4f7ffff' --x 118 --y 252
"$LG" layer add "$OUTPUT/showcase.kgfx" --type text --id title-b --name "Title line two" --font-asset-id display-font --text "FOR CODE" --font-size 62 --color '#a98affff' --x 118 --y 338
"$LG" layer add "$OUTPUT/showcase.kgfx" --type text --id line-one --name "Capability one" --font-asset-id display-font --text "EDITABLE DOCUMENTS" --font-size 18 --color '#c3ccdcff' --x 120 --y 510
"$LG" layer add "$OUTPUT/showcase.kgfx" --type text --id line-two --name "Capability two" --font-asset-id display-font --text "BROWSER PREVIEWS" --font-size 18 --color '#c3ccdcff' --x 120 --y 558
"$LG" layer add "$OUTPUT/showcase.kgfx" --type text --id line-three --name "Capability three" --font-asset-id display-font --text "SCRIPTABLE OUTPUT" --font-size 18 --color '#c3ccdcff' --x 120 --y 606
"$LG" layer add "$OUTPUT/showcase.kgfx" --type text --id format --name "Format badge" --font-asset-id display-font --text "KGFX" --font-size 18 --color '#070b14ff' --x 126 --y 737
"$LG" layer add "$OUTPUT/showcase.kgfx" --type fill --id format-badge --name "Format badge background" --width 116 --height 44 --color '#36e3c4ff' --x 108 --y 726
"$LG" layer move "$OUTPUT/showcase.kgfx" format-badge --below format

"$LG" validate "$OUTPUT/showcase.kgfx"
"$LG" render "$OUTPUT/showcase.kgfx" -o "$OUTPUT/showcase.png" --sampling smooth
cp "$OUTPUT/showcase.png" "$ROOT/apps/site/public/showcase.png"
cp "$OUTPUT/showcase.kgfx" "$ROOT/apps/site/public/showcase.kgfx"
printf 'Built %s and %s\n' "$OUTPUT/showcase.kgfx" "$OUTPUT/showcase.png"
