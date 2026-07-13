import initWasm, {
  Document as RawDocument,
  inspect_kgfx,
  render_kgfx_png,
  validate_kgfx,
  version,
} from "./generated/lg_wasm.js";

export type WasmInitInput = Parameters<typeof initWasm>[0];
export type BlendMode = "normal" | "multiply";
export type Rgba = [number, number, number, number];
export type Sampling = "nearest" | "smooth";
export type OutputFormat = "png" | "jpg" | "jpeg" | "webp";

export interface Transform {
  x: number;
  y: number;
  scaleX: number;
  scaleY: number;
}

export interface LayerCommon {
  id: string;
  name: string;
  visible: boolean;
  opacity: number;
  blendMode: BlendMode;
  transform: Transform;
}

export type Layer = LayerCommon &
  (
    | { type: "image"; assetId: string }
    | { type: "fill"; width: number; height: number; color: Rgba }
    | { type: "text"; text: string; fontAssetId: string; fontSize: number; color: Rgba }
    | { type: "group"; children: Layer[] }
  );

export type LayerInput = Partial<Pick<LayerCommon, "id">> &
  Partial<Omit<LayerCommon, "id">> &
  (
    | { type: "image"; assetId: string }
    | { type: "fill"; width: number; height: number; color: Rgba }
    | { type: "text"; text: string; fontAssetId: string; fontSize: number; color: Rgba }
    | { type: "group"; children?: Layer[] }
  );

export interface LayerPatch {
  name?: string;
  visible?: boolean;
  opacity?: number;
  blendMode?: BlendMode;
  x?: number;
  y?: number;
  scaleX?: number;
  scaleY?: number;
  assetId?: string;
  text?: string;
  fontAssetId?: string;
  fontSize?: number;
  color?: Rgba;
  width?: number;
  height?: number;
}

export type Command =
  | { op: "documentUpdate"; width?: number; height?: number; dpi?: number; color?: Rgba }
  | { op: "layerAdd"; layer: Layer; parentId?: string; index?: number }
  | { op: "layerUpdate"; id: string; set: LayerPatch }
  | { op: "layerRemove"; id: string }
  | { op: "layerMove"; id: string; parentId?: string; to?: number; above?: string; below?: string }
  | { op: "assetAdd"; id: string; mediaType: string; bytesBase64: string; originalName?: string; author?: unknown }
  | {
      op: "assetLink";
      id: string;
      mediaType: string;
      reference: string;
      byteLength: number;
      sha256: string;
      originalName?: string;
      author?: unknown;
    }
  | { op: "assetRelink"; id: string; reference: string; byteLength: number; sha256: string }
  | { op: "assetRemove"; id: string }
  | { op: "extensionSet"; namespace: string; value: unknown }
  | { op: "extensionRemove"; namespace: string };

export interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface Invalidation {
  impact: "metadata" | "localPixels" | "composite" | "asset" | "global";
  reason: string;
  fullRender: boolean;
  layerIds: string[];
  assetIds: string[];
  regions: Rect[];
}

export interface CommandResult {
  fromRevision: number;
  revision: number;
  applied: number;
  changedLayers: string[];
  changedAssets: string[];
  changes: Invalidation[];
  warnings: Array<{ severity: string; code: string; message: string; commandIndex?: number }>;
}

export class CommandExecutionError extends Error {
  readonly code: "command.invalid" | "transaction.validation";
  readonly commandIndex?: number;

  constructor(message: string) {
    super(message);
    this.name = "CommandExecutionError";
    const match = message.match(/command (\d+):\s*(.*)/s);
    this.code = match ? "command.invalid" : "transaction.validation";
    this.commandIndex = match ? Number(match[1]) : undefined;
  }
}

export interface HistoryState {
  undoCount: number;
  redoCount: number;
  undoLabel?: string;
  redoLabel?: string;
  revision: number;
}

export interface Manifest {
  schemaVersion: number;
  id: string;
  revision: number;
  canvas: { width: number; height: number; dpi: number; color: Rgba };
  layers: Layer[];
  assets: Record<string, unknown>;
  extensions: Record<string, unknown>;
}

export interface RenderOptions {
  layerId?: string;
  scale?: number;
  sampling?: Sampling;
  background?: Rgba;
}

export interface PixelBuffer {
  width: number;
  height: number;
  data: Uint8Array;
}

export interface RetainedRenderMetrics {
  cacheHits: number;
  cacheMisses: number;
  cacheEvictions: number;
  cacheBytes: number;
  entries: number;
}

let initialized: Promise<unknown> | undefined;

export function initialize(input?: WasmInitInput): Promise<unknown> {
  initialized ??= initWasm(input);
  return initialized;
}

export function engineVersion(): string {
  return version();
}

export function inspectKgfx(bytes: Uint8Array): Manifest {
  return JSON.parse(inspect_kgfx(bytes)) as Manifest;
}

export function validateKgfx(bytes: Uint8Array): unknown[] {
  return JSON.parse(validate_kgfx(bytes)) as unknown[];
}

export function renderKgfxPng(bytes: Uint8Array, scale = 1): Uint8Array {
  return render_kgfx_png(bytes, scale);
}

export class GraphicsDocument {
  readonly layers: LayerCollection;
  readonly assets: AssetCollection;
  readonly extensions: ExtensionCollection;

  private constructor(private readonly raw: RawDocument) {
    this.layers = new LayerCollection(this);
    this.assets = new AssetCollection(this);
    this.extensions = new ExtensionCollection(this);
  }

  static async create(options: { width: number; height: number; dpi?: number }): Promise<GraphicsDocument> {
    await initialize();
    return new GraphicsDocument(new RawDocument(options.width, options.height, options.dpi ?? 72));
  }

  static async open(bytes: Uint8Array): Promise<GraphicsDocument> {
    await initialize();
    return new GraphicsDocument(RawDocument.open(bytes));
  }

  execute(commands: Command | Command[], label?: string): CommandResult {
    try {
      return JSON.parse(this.raw.execute(JSON.stringify(commands), label)) as CommandResult;
    } catch (error) {
      throw new CommandExecutionError(error instanceof Error ? error.message : String(error));
    }
  }

  updateCanvas(set: { width?: number; height?: number; dpi?: number; color?: Rgba }, label = "Update canvas"): CommandResult {
    return this.execute({ op: "documentUpdate", ...set }, label);
  }

  undo(): HistoryState {
    return JSON.parse(this.raw.undo()) as HistoryState;
  }

  redo(): HistoryState {
    return JSON.parse(this.raw.redo()) as HistoryState;
  }

  get history(): HistoryState {
    return JSON.parse(this.raw.historyState()) as HistoryState;
  }

  get manifest(): Manifest {
    return JSON.parse(this.raw.manifest()) as Manifest;
  }

  render(format: OutputFormat = "png", options: RenderOptions = {}): Uint8Array {
    return this.raw.render(format, JSON.stringify({ scale: 1, sampling: "nearest", ...options }));
  }

  renderRgba(options: RenderOptions = {}): PixelBuffer {
    return unpackPixels(this.raw.renderRgba(JSON.stringify({ scale: 1, sampling: "nearest", ...options })));
  }

  renderRetained(format: OutputFormat = "png", options: RenderOptions = {}): Uint8Array {
    return this.raw.renderRetained(format, JSON.stringify({ scale: 1, sampling: "nearest", ...options }));
  }

  get retainedMetrics(): RetainedRenderMetrics {
    return JSON.parse(this.raw.retainedMetrics()) as RetainedRenderMetrics;
  }

  rasterizeLayer(id: string, sampling: Sampling = "nearest"): PixelBuffer {
    return unpackPixels(this.raw.rasterizeLayer(id, sampling));
  }

  exportKgfx(): Uint8Array {
    return this.raw.exportKgfx();
  }

  provideLinkedAsset(id: string, bytes: Uint8Array): void {
    this.raw.provideLinkedAsset(id, bytes);
  }

  close(): void {
    this.raw.free();
  }
}

function unpackPixels(packed: Uint8Array): PixelBuffer {
  if (packed.byteLength < 8) throw new Error("The engine returned an invalid pixel buffer");
  const view = new DataView(packed.buffer, packed.byteOffset, packed.byteLength);
  const width = view.getUint32(0, true);
  const height = view.getUint32(4, true);
  const expected = width * height * 4;
  if (packed.byteLength !== expected + 8) throw new Error("The engine returned inconsistent pixel dimensions");
  return { width, height, data: packed.slice(8) };
}

export class LayerHandle {
  constructor(
    private readonly document: GraphicsDocument,
    readonly id: string,
  ) {}

  update(set: LayerPatch, label = `Update ${this.id}`): CommandResult {
    return this.document.execute({ op: "layerUpdate", id: this.id, set }, label);
  }

  remove(label = `Remove ${this.id}`): CommandResult {
    return this.document.execute({ op: "layerRemove", id: this.id }, label);
  }

  move(destination: { parentId?: string; to?: number; above?: string; below?: string }, label = `Move ${this.id}`): CommandResult {
    return this.document.execute({ op: "layerMove", id: this.id, ...destination }, label);
  }
}

export class LayerCollection {
  constructor(private readonly document: GraphicsDocument) {}

  add(input: LayerInput, options: { parentId?: string; index?: number; label?: string } = {}): LayerHandle {
    const layer = normalizeLayer(input);
    const result = this.document.execute(
      { op: "layerAdd", layer, parentId: options.parentId, index: options.index },
      options.label ?? `Add ${layer.name}`,
    );
    return new LayerHandle(this.document, result.changedLayers[0] ?? layer.id);
  }

  get(id: string): LayerHandle {
    return new LayerHandle(this.document, id);
  }

  list(): Layer[] {
    return this.document.manifest.layers;
  }
}

export class AssetCollection {
  constructor(private readonly document: GraphicsDocument) {}

  add(input: { id: string; mediaType: string; bytes: Uint8Array; originalName?: string; author?: unknown }, label = `Add ${input.id}`): CommandResult {
    return this.document.execute(
      {
        op: "assetAdd",
        id: input.id,
        mediaType: input.mediaType,
        bytesBase64: bytesToBase64(input.bytes),
        originalName: input.originalName,
        author: input.author,
      },
      label,
    );
  }

  link(input: { id: string; mediaType: string; reference: string; byteLength: number; sha256: string; originalName?: string; author?: unknown }, label = `Link ${input.id}`): CommandResult {
    return this.document.execute({ op: "assetLink", ...input }, label);
  }

  relink(id: string, input: { reference: string; byteLength: number; sha256: string }, label = `Relink ${id}`): CommandResult {
    return this.document.execute({ op: "assetRelink", id, ...input }, label);
  }

  remove(id: string, label = `Remove ${id}`): CommandResult {
    return this.document.execute({ op: "assetRemove", id }, label);
  }
}

export class ExtensionCollection {
  constructor(private readonly document: GraphicsDocument) {}

  set(namespace: string, value: unknown, label = `Set ${namespace}`): CommandResult {
    return this.document.execute({ op: "extensionSet", namespace, value }, label);
  }

  remove(namespace: string, label = `Remove ${namespace}`): CommandResult {
    return this.document.execute({ op: "extensionRemove", namespace }, label);
  }

  list(): Record<string, unknown> {
    return this.document.manifest.extensions;
  }
}

function normalizeLayer(input: LayerInput): Layer {
  return {
    id: input.id ?? "",
    name: input.name ?? input.id ?? "Layer",
    visible: input.visible ?? true,
    opacity: input.opacity ?? 1,
    blendMode: input.blendMode ?? "normal",
    transform: input.transform ?? { x: 0, y: 0, scaleX: 1, scaleY: 1 },
    ...input,
    ...(input.type === "group" ? { children: input.children ?? [] } : {}),
  } as Layer;
}

function bytesToBase64(bytes: Uint8Array): string {
  let binary = "";
  const size = 0x8000;
  for (let index = 0; index < bytes.length; index += size) {
    binary += String.fromCharCode(...bytes.subarray(index, index + size));
  }
  return btoa(binary);
}
