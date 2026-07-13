import type { Command, CommandResult, Manifest, OutputFormat, RenderOptions } from "@layered-graphics/core";

export type QualityIntent = "interactive" | "preview" | "refined" | "authoritative";
export type PreviewBackend = "webgpu" | "canvas2d" | "rgba";

export interface Viewport {
  x: number;
  y: number;
  width: number;
  height: number;
  zoom: number;
}

export interface RenderMetrics {
  revision: number;
  backend: PreviewBackend;
  requestedTier: QualityIntent;
  deliveredTier: QualityIntent;
  totalMs: number;
  rasterMs: number;
  decodeMs: number;
  encodeMs: number;
  presentMs: number;
  cacheHits: number;
  cacheMisses: number;
  cacheEvictions: number;
  cacheBytes: number;
  dirtyLayers: string[];
  dirtyRegions: Array<{ x: number; y: number; width: number; height: number }>;
  invalidationReasons: string[];
  cancelled: number;
  superseded: number;
  cpuBytes: number;
  gpuBytes: number;
}

export interface PreviewFrame {
  revision: number;
  width: number;
  height: number;
  tier: QualityIntent;
  backend: PreviewBackend;
  pixels?: Uint8Array;
  metrics: RenderMetrics;
}

export interface BatchRequest {
  commands?: Command | Command[];
  format?: OutputFormat;
  options?: RenderOptions;
  label?: string;
}

export interface BatchResult {
  index: number;
  revision: number;
  bytes?: Uint8Array;
  error?: string;
  metrics: RenderMetrics;
}

export type WorkerRequest =
  | { id: number; type: "create"; width: number; height: number; dpi?: number; canvas?: OffscreenCanvas }
  | { id: number; type: "open"; bytes: Uint8Array; canvas?: OffscreenCanvas }
  | { id: number; type: "execute"; commands: Command | Command[]; label?: string }
  | { id: number; type: "provideLinkedAsset"; assetId: string; bytes: Uint8Array }
  | { id: number; type: "preview"; quality: QualityIntent; viewport?: Viewport; includePixels?: boolean }
  | { id: number; type: "export"; format: OutputFormat; options?: RenderOptions }
  | { id: number; type: "snapshot" }
  | { id: number; type: "manifest" }
  | { id: number; type: "metrics" }
  | { id: number; type: "batch"; requests: BatchRequest[]; maxConcurrency: number }
  | { id: number; type: "cancel"; target: number }
  | { id: number; type: "simulateDeviceLoss" }
  | { id: number; type: "close" };

export type WorkerResponse =
  | { id: number; ok: true; value?: unknown }
  | { id: number; ok: false; error: string }
  | { id: 0; ok: true; event: "revision"; value: CommandResult }
  | { id: 0; ok: true; event: "previewReady"; value: PreviewFrame }
  | { id: 0; ok: true; event: "diagnostic"; value: { level: "warning" | "error"; message: string } };

export type SessionEvent =
  | { type: "revision"; value: CommandResult }
  | { type: "previewReady"; value: PreviewFrame }
  | { type: "diagnostic"; value: { level: "warning" | "error"; message: string } };

export type { Command, CommandResult, Manifest, OutputFormat, RenderOptions };
