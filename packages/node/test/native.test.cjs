const assert = require("node:assert/strict");
const test = require("node:test");
const { NativeDocument, version } = require("../index.js");
const { CommandExecutionError, GraphicsDocument } = require("../sdk.cjs");

test("native Node document executes, undoes, renders, and serializes", () => {
  const document = new NativeDocument(16, 16, 72);
  const result = JSON.parse(document.execute(JSON.stringify({
    op: "layerAdd",
    layer: {
      id: "fill",
      name: "Fill",
      visible: true,
      opacity: 1,
      blendMode: "normal",
      transform: { x: 0, y: 0, scaleX: 1, scaleY: 1 },
      type: "fill",
      width: 16,
      height: 16,
      color: [20, 40, 60, 255],
    },
  }), "Add fill"));
  assert.equal(result.revision, 1);
  assert.equal(Buffer.from(document.render("png")).subarray(1, 4).toString(), "PNG");
  assert.ok(document.exportKgfx().length > 0);
  document.undo();
  assert.equal(JSON.parse(document.manifest).layers.length, 0);
  document.redo();
  assert.equal(JSON.parse(document.manifest).layers.length, 1);
  assert.match(version(), /^0\.1\.0/);
});

test("native command and imperative APIs produce equivalent graphical state", () => {
  const commands = new NativeDocument(8, 8, 72);
  const fixture = require("node:fs").readFileSync(require("node:path").join(__dirname, "../../../apps/site/public/api-equivalence.ops.json"), "utf8");
  commands.execute(fixture);
  const imperative = GraphicsDocument.create({ width: 8, height: 8 });
  imperative.updateCanvas({ dpi: 144, color: [1, 2, 3, 4] });
  imperative.extensions.set("com.layeredgraphics.equivalence", { ok: true });
  imperative.layers.add({ id: "fill", name: "Fill", type: "fill", width: 4, height: 4, color: [9, 8, 7, 255], transform: { x: 1, y: 2, scaleX: 1, scaleY: 1 } });
  const graphical = (manifest) => ({ canvas: manifest.canvas, layers: manifest.layers, extensions: manifest.extensions });
  assert.deepEqual(graphical(JSON.parse(commands.manifest)), graphical(imperative.manifest));
  assert.throws(
    () => imperative.execute(JSON.parse(fixture)[2]),
    (error) => error instanceof CommandExecutionError && error.code === "command.invalid" && error.commandIndex === 0,
  );
});

test("native Node resolves the same opaque linked image contract as the browser", () => {
  const fs = require("node:fs");
  const path = require("node:path");
  const crypto = require("node:crypto");
  const bytes = fs.readFileSync(path.join(__dirname, "../../../apps/site/public/readme-banner.png"));
  const document = GraphicsDocument.create({ width: 16, height: 16 });
  document.assets.link({
    id: "linked-banner", mediaType: "image/png", reference: "memory://shared-fixture",
    byteLength: bytes.length, sha256: crypto.createHash("sha256").update(bytes).digest("hex"),
  });
  document.provideLinkedAsset("linked-banner", bytes);
  document.layers.add({ id: "linked-layer", type: "image", assetId: "linked-banner" });
  assert.ok(document.render("png").length > 0);
});
