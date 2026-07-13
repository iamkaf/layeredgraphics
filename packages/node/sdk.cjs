const native = require("./index.js");

class CommandExecutionError extends Error {
  constructor(message) {
    super(message);
    this.name = "CommandExecutionError";
    const match = message.match(/command (\d+):\s*(.*)/s);
    this.code = match ? "command.invalid" : "transaction.validation";
    this.commandIndex = match ? Number(match[1]) : undefined;
  }
}

class GraphicsDocument {
  constructor(raw) {
    this.raw = raw;
    this.layers = new LayerCollection(this);
    this.assets = new AssetCollection(this);
    this.extensions = new ExtensionCollection(this);
  }

  static create({ width, height, dpi = 72 }) { return new GraphicsDocument(new native.NativeDocument(width, height, dpi)); }
  static open(bytes) { return new GraphicsDocument(native.NativeDocument.open(Buffer.from(bytes))); }
  execute(commands, label) {
    try { return JSON.parse(this.raw.execute(JSON.stringify(commands), label)); }
    catch (error) { throw new CommandExecutionError(error instanceof Error ? error.message : String(error)); }
  }
  updateCanvas(set, label = "Update canvas") { return this.execute({ op: "documentUpdate", ...set }, label); }
  undo() { return JSON.parse(this.raw.undo()); }
  redo() { return JSON.parse(this.raw.redo()); }
  get history() { return JSON.parse(this.raw.historyState); }
  get manifest() { return JSON.parse(this.raw.manifest); }
  render(format = "png", options = {}) { return this.raw.render(format, JSON.stringify({ scale: 1, sampling: "nearest", ...options })); }
  exportKgfx() { return this.raw.exportKgfx(); }
  provideLinkedAsset(id, bytes) { this.raw.provideLinkedAsset(id, Buffer.from(bytes)); }
  close() { /* N-API object lifetime is governed by JavaScript reachability. */ }
}

class LayerHandle {
  constructor(document, id) { this.document = document; this.id = id; }
  update(set, label = `Update ${this.id}`) { return this.document.execute({ op: "layerUpdate", id: this.id, set }, label); }
  remove(label = `Remove ${this.id}`) { return this.document.execute({ op: "layerRemove", id: this.id }, label); }
  move(destination, label = `Move ${this.id}`) { return this.document.execute({ op: "layerMove", id: this.id, ...destination }, label); }
}

class LayerCollection {
  constructor(document) { this.document = document; }
  add(input, options = {}) {
    const layer = {
      id: input.id ?? "", name: input.name ?? input.id ?? "Layer", visible: input.visible ?? true,
      opacity: input.opacity ?? 1, blendMode: input.blendMode ?? "normal",
      transform: input.transform ?? { x: 0, y: 0, scaleX: 1, scaleY: 1 }, ...input,
      ...(input.type === "group" ? { children: input.children ?? [] } : {}),
    };
    const result = this.document.execute({ op: "layerAdd", layer, parentId: options.parentId, index: options.index }, options.label ?? `Add ${layer.name}`);
    return new LayerHandle(this.document, result.changedLayers[0] ?? layer.id);
  }
  get(id) { return new LayerHandle(this.document, id); }
  list() { return this.document.manifest.layers; }
}

class AssetCollection {
  constructor(document) { this.document = document; }
  add(input, label = `Add ${input.id}`) {
    return this.document.execute({ op: "assetAdd", id: input.id, mediaType: input.mediaType, bytesBase64: Buffer.from(input.bytes).toString("base64"), originalName: input.originalName, author: input.author }, label);
  }
  link(input, label = `Link ${input.id}`) { return this.document.execute({ op: "assetLink", ...input }, label); }
  relink(id, input, label = `Relink ${id}`) { return this.document.execute({ op: "assetRelink", id, ...input }, label); }
  remove(id, label = `Remove ${id}`) { return this.document.execute({ op: "assetRemove", id }, label); }
}

class ExtensionCollection {
  constructor(document) { this.document = document; }
  set(namespace, value, label = `Set ${namespace}`) { return this.document.execute({ op: "extensionSet", namespace, value }, label); }
  remove(namespace, label = `Remove ${namespace}`) { return this.document.execute({ op: "extensionRemove", namespace }, label); }
  list() { return this.document.manifest.extensions; }
}

module.exports = { ...native, CommandExecutionError, GraphicsDocument, LayerHandle, LayerCollection, AssetCollection, ExtensionCollection };
