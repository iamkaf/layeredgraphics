import type { Command, CommandResult, HistoryState, Layer, LayerInput, LayerPatch, Manifest, OutputFormat, RenderOptions, Rgba } from "@layered-graphics/core";
export * from "./index";

export declare class CommandExecutionError extends Error {
  readonly code: "command.invalid" | "transaction.validation";
  readonly commandIndex?: number;
}

export declare class GraphicsDocument {
  static create(options: { width: number; height: number; dpi?: number }): GraphicsDocument;
  static open(bytes: Uint8Array): GraphicsDocument;
  readonly layers: LayerCollection;
  readonly assets: AssetCollection;
  readonly extensions: ExtensionCollection;
  execute(commands: Command | Command[], label?: string): CommandResult;
  updateCanvas(set: { width?: number; height?: number; dpi?: number; color?: Rgba }, label?: string): CommandResult;
  undo(): HistoryState;
  redo(): HistoryState;
  readonly history: HistoryState;
  readonly manifest: Manifest;
  render(format?: OutputFormat, options?: RenderOptions): Buffer;
  exportKgfx(): Buffer;
  provideLinkedAsset(id: string, bytes: Uint8Array): void;
  close(): void;
}

export declare class LayerHandle {
  readonly id: string;
  update(set: LayerPatch, label?: string): CommandResult;
  remove(label?: string): CommandResult;
  move(destination: { parentId?: string; to?: number; above?: string; below?: string }, label?: string): CommandResult;
}
export declare class LayerCollection {
  add(input: LayerInput, options?: { parentId?: string; index?: number; label?: string }): LayerHandle;
  get(id: string): LayerHandle;
  list(): Layer[];
}
export declare class AssetCollection {
  add(input: { id: string; mediaType: string; bytes: Uint8Array; originalName?: string; author?: unknown }, label?: string): CommandResult;
  link(input: { id: string; mediaType: string; reference: string; byteLength: number; sha256: string; originalName?: string; author?: unknown }, label?: string): CommandResult;
  relink(id: string, input: { reference: string; byteLength: number; sha256: string }, label?: string): CommandResult;
  remove(id: string, label?: string): CommandResult;
}
export declare class ExtensionCollection {
  set(namespace: string, value: unknown, label?: string): CommandResult;
  remove(namespace: string, label?: string): CommandResult;
  list(): Record<string, unknown>;
}
