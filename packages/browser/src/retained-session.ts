import { GraphicsDocument, type Command, type CommandResult, type Manifest, type PixelBuffer, type RenderOptions } from "@layered-graphics/core";
import type { BatchRequest, BatchResult, PreviewBackend, PreviewFrame, QualityIntent, RenderMetrics } from "./protocol.js";
import type { GpuLayerInput } from "./presenter.js";

const TIER_SCALE: Record<QualityIntent, number> = { interactive: 0.5, preview: 1, refined: 1, authoritative: 1 };

export class RetainedRenderSession {
  private cache = new Map<string, PixelBuffer>();
  private cacheBytes = 0;
  private hits = 0;
  private misses = 0;
  private evictions = 0;
  private cancelled = 0;
  private superseded = 0;
  private dirtyLayers: string[] = [];
  private dirtyRegions: RenderMetrics["dirtyRegions"] = [];
  private reasons: string[] = [];
  private closed = false;
  private lastMetrics: RenderMetrics;

  constructor(readonly document: GraphicsDocument, readonly memoryBudgetBytes = 256 * 1024 * 1024) {
    this.lastMetrics = this.metrics("rgba", "preview", "preview", 0, 0, 0);
  }

  static async create(options: { width: number; height: number; dpi?: number; memoryBudgetBytes?: number }): Promise<RetainedRenderSession> {
    return new RetainedRenderSession(await GraphicsDocument.create(options), options.memoryBudgetBytes);
  }

  static async open(bytes: Uint8Array, memoryBudgetBytes?: number): Promise<RetainedRenderSession> {
    return new RetainedRenderSession(await GraphicsDocument.open(bytes), memoryBudgetBytes);
  }

  get manifest(): Manifest { return this.document.manifest; }

  execute(commands: Command | Command[], label?: string): CommandResult {
    this.assertOpen();
    const result = this.document.execute(commands, label);
    this.consumeChanges(result);
    return result;
  }

  provideLinkedAsset(id: string, bytes: Uint8Array): void {
    this.assertOpen();
    this.document.provideLinkedAsset(id, bytes);
    this.evictions += this.cache.size;
    this.cache.clear();
    this.cacheBytes = 0;
    this.reasons = ["linkedAssetResolved"];
  }

  layerRaster(id: string, sampling: "nearest" | "smooth" = "nearest"): PixelBuffer {
    this.assertOpen();
    const key = `${id}:${sampling}`;
    const cached = this.cache.get(key);
    if (cached) {
      this.hits++;
      this.cache.delete(key);
      this.cache.set(key, cached);
      return cached;
    }
    this.misses++;
    const value = this.document.rasterizeLayer(id, sampling);
    this.cache.set(key, value);
    this.cacheBytes += value.data.byteLength;
    this.enforceBudget();
    return value;
  }

  preview(quality: QualityIntent, backend: PreviewBackend = "rgba", includePixels = true): PreviewFrame {
    this.assertOpen();
    const started = performance.now();
    const sampling = quality === "interactive" ? "nearest" : "smooth";
    const rasterStarted = performance.now();
    const pixels = this.document.renderRgba({ scale: TIER_SCALE[quality], sampling });
    const finished = performance.now();
    const retained = this.document.retainedMetrics;
    this.hits = retained.cacheHits;
    this.misses = retained.cacheMisses;
    this.evictions = retained.cacheEvictions;
    this.cacheBytes = retained.cacheBytes;
    this.lastMetrics = this.metrics(backend, quality, quality, finished - started, finished - rasterStarted, 0);
    const frame: PreviewFrame = {
      revision: this.manifest.revision,
      width: pixels.width,
      height: pixels.height,
      tier: quality,
      backend,
      metrics: this.lastMetrics,
    };
    if (includePixels) frame.pixels = pixels.data;
    this.clearDirty();
    return frame;
  }

  async previewLayers(quality: QualityIntent, shouldCancel: () => boolean = () => false): Promise<{ frame: PreviewFrame; layers: GpuLayerInput[]; outputScale: number; sampling: "nearest" | "smooth" }> {
    this.assertOpen();
    const started = performance.now();
    const sampling = quality === "interactive" ? "nearest" : "smooth";
    const layers: GpuLayerInput[] = [];
    for (const layer of this.manifest.layers) {
      if (shouldCancel()) {
        this.noteCancelled();
        throw new Error("Preview cancelled or superseded");
      }
      layers.push({ layer, raster: this.layerRaster(layer.id, sampling) });
      // Yield between independently retained sources so rapid input can supersede
      // large/deep work without allowing an unbounded message queue.
      await new Promise<void>((resolve) => setTimeout(resolve, 0));
    }
    const outputScale = TIER_SCALE[quality];
    const finished = performance.now();
    this.lastMetrics = this.metrics("webgpu", quality, quality, finished - started, finished - started, 0);
    const frame: PreviewFrame = {
      revision: this.manifest.revision,
      width: Math.max(1, Math.round(this.manifest.canvas.width * outputScale)),
      height: Math.max(1, Math.round(this.manifest.canvas.height * outputScale)),
      tier: quality,
      backend: "webgpu",
      metrics: this.lastMetrics,
    };
    this.clearDirty();
    return { frame, layers, outputScale, sampling };
  }

  export(format: "png" | "jpg" | "jpeg" | "webp", options: RenderOptions = {}): Uint8Array {
    this.assertOpen();
    return this.document.render(format, { sampling: "smooth", ...options });
  }

  exportRetained(format: "png" | "jpg" | "jpeg" | "webp", options: RenderOptions = {}): Uint8Array {
    this.assertOpen();
    return this.document.renderRetained(format, { sampling: "smooth", ...options });
  }

  async batch(requests: BatchRequest[], maxConcurrency = 2, signal?: AbortSignal): Promise<BatchResult[]> {
    this.assertOpen();
    if (!Number.isInteger(maxConcurrency) || maxConcurrency < 1 || maxConcurrency > 8) {
      throw new Error("maxConcurrency must be an integer from 1 to 8");
    }
    // A canonical mutable session serializes related revisions. The caller's bound
    // limits each output chunk/backpressure window; effective render concurrency is one.
    const results: BatchResult[] = [];
    for (let index = 0; index < requests.length; index++) {
      if (signal?.aborted) {
        this.cancelled++;
        throw signal.reason ?? new DOMException("Batch cancelled", "AbortError");
      }
      const request = requests[index]!;
      try {
        if (request.commands) this.execute(request.commands, request.label ?? `Batch item ${index}`);
        const started = performance.now();
        const bytes = this.exportRetained(request.format ?? "png", request.options);
        const elapsed = performance.now() - started;
        const metrics = this.metrics("rgba", "authoritative", "authoritative", elapsed, elapsed, 0);
        metrics.decodeMs = 0;
        metrics.encodeMs = elapsed;
        results.push({ index, revision: this.manifest.revision, bytes, metrics });
      } catch (error) {
        results.push({ index, revision: this.manifest.revision, error: error instanceof Error ? error.message : String(error), metrics: this.lastMetrics });
      }
      await new Promise<void>((resolve) => setTimeout(resolve, 0));
    }
    return results;
  }

  noteSuperseded(): void { this.superseded++; }
  noteCancelled(): void { this.cancelled++; }
  getMetrics(): RenderMetrics { return { ...this.lastMetrics, cancelled: this.cancelled, superseded: this.superseded }; }
  snapshot(): Uint8Array { this.assertOpen(); return this.document.exportKgfx(); }

  close(): void {
    if (this.closed) return;
    this.cache.clear();
    this.cacheBytes = 0;
    this.document.close();
    this.closed = true;
  }

  private consumeChanges(result: CommandResult): void {
    this.dirtyLayers = [...new Set(result.changes.flatMap((change) => change.layerIds))];
    this.dirtyRegions = result.changes.flatMap((change) => change.regions);
    this.reasons = result.changes.map((change) => change.reason);
    const assets = new Set(result.changes.filter((change) => change.impact === "asset").flatMap((change) => change.layerIds));
    const sources = new Set(result.changes.filter((change) => change.reason === "layerSourceChanged" || change.reason === "layerAdded" || change.reason === "layerRemoved").flatMap((change) => change.layerIds));
    for (const id of [...assets, ...sources]) this.evictLayer(rootLayerId(this.manifest.layers, id) ?? id);
  }

  private evictLayer(id: string): void {
    for (const [key, value] of this.cache) if (key.startsWith(`${id}:`)) {
      this.cache.delete(key);
      this.cacheBytes -= value.data.byteLength;
      this.evictions++;
    }
  }

  private enforceBudget(): void {
    while (this.cacheBytes > this.memoryBudgetBytes && this.cache.size > 1) {
      const first = this.cache.entries().next().value as [string, PixelBuffer] | undefined;
      if (!first) break;
      this.cache.delete(first[0]);
      this.cacheBytes -= first[1].data.byteLength;
      this.evictions++;
    }
  }

  private metrics(backend: PreviewBackend, requested: QualityIntent, delivered: QualityIntent, totalMs: number, rasterMs: number, presentMs: number): RenderMetrics {
    return {
      revision: this.document.manifest.revision, backend, requestedTier: requested, deliveredTier: delivered,
      totalMs, rasterMs, decodeMs: rasterMs, encodeMs: 0, presentMs, cacheHits: this.hits, cacheMisses: this.misses, cacheEvictions: this.evictions,
      cacheBytes: this.cacheBytes, dirtyLayers: [...this.dirtyLayers], dirtyRegions: [...this.dirtyRegions],
      invalidationReasons: [...this.reasons], cancelled: this.cancelled, superseded: this.superseded,
      cpuBytes: this.cacheBytes, gpuBytes: 0,
    };
  }

  private clearDirty(): void { this.dirtyLayers = []; this.dirtyRegions = []; this.reasons = []; }
  private assertOpen(): void { if (this.closed) throw new Error("Render session is closed"); }
}

function rootLayerId(layers: Manifest["layers"], id: string): string | undefined {
  const contains = (layer: Manifest["layers"][number]): boolean =>
    layer.id === id || (layer.type === "group" && layer.children.some(contains));
  return layers.find(contains)?.id;
}
