/// <reference lib="webworker" />
import { RetainedRenderSession } from "./retained-session.js";
import { PreviewPresenter } from "./presenter.js";
import type { PreviewFrame, WorkerRequest, WorkerResponse } from "./protocol.js";

export class RenderWorkerHost {
  private session?: RetainedRenderSession;
  private presenter?: PreviewPresenter;
  private gpuAvailable = false;
  private cancelled = new Set<number>();
  private pendingPreview?: WorkerRequest & { type: "preview" };
  private previewScheduled = false;
  private activePreview?: number;
  private activeJobs = new Map<number, AbortController>();
  private refinement?: ReturnType<typeof setTimeout>;

  constructor(private readonly send: (response: WorkerResponse, transfer?: Transferable[]) => void) {}

  async receive(request: WorkerRequest): Promise<void> {
    try {
      if (request.type === "cancel") {
        this.cancelled.add(request.target);
        this.activeJobs.get(request.target)?.abort(new Error("Work cancelled"));
        this.session?.noteCancelled();
        this.reply(request.id, undefined);
        return;
      }
      if (request.type === "preview") {
        if (this.activePreview !== undefined) {
          this.cancelled.add(this.activePreview);
          this.session?.noteSuperseded();
        }
        if (this.pendingPreview) {
          this.session?.noteSuperseded();
          this.fail(this.pendingPreview.id, "Preview superseded by a newer request");
        }
        this.pendingPreview = request;
        if (!this.previewScheduled) {
          this.previewScheduled = true;
          setTimeout(() => void this.flushPreview(), 0);
        }
        return;
      }
      await this.handle(request);
    } catch (error) {
      this.fail(request.id, error instanceof Error ? error.message : String(error));
    }
  }

  private async handle(request: Exclude<WorkerRequest, { type: "preview" | "cancel" }>): Promise<void> {
    switch (request.type) {
      case "create":
        this.replace(await RetainedRenderSession.create({ width: request.width, height: request.height, dpi: request.dpi }));
        if (request.canvas) this.presenter = new PreviewPresenter(request.canvas);
        {
          const capability = await this.presenter?.capability();
          this.gpuAvailable = capability?.webgpu ?? false;
          this.reply(request.id, { manifest: this.session!.manifest, capability });
        }
        break;
      case "open":
        this.replace(await RetainedRenderSession.open(request.bytes));
        if (request.canvas) this.presenter = new PreviewPresenter(request.canvas);
        {
          const capability = await this.presenter?.capability();
          this.gpuAvailable = capability?.webgpu ?? false;
          this.reply(request.id, { manifest: this.session!.manifest, capability });
        }
        break;
      case "execute": {
        const result = this.requireSession().execute(request.commands, request.label);
        this.reply(request.id, result);
        this.send({ id: 0, ok: true, event: "revision", value: result });
        clearTimeout(this.refinement);
        this.refinement = setTimeout(() => void this.emitRefinement(), 120);
        break;
      }
      case "provideLinkedAsset":
        this.requireSession().provideLinkedAsset(request.assetId, request.bytes);
        this.reply(request.id, undefined);
        break;
      case "export": {
        const bytes = this.requireSession().export(request.format, request.options);
        this.reply(request.id, bytes, [bytes.buffer]);
        break;
      }
      case "snapshot": {
        const bytes = this.requireSession().snapshot();
        this.reply(request.id, bytes, [bytes.buffer]);
        break;
      }
      case "manifest": this.reply(request.id, this.requireSession().manifest); break;
      case "metrics": this.reply(request.id, this.requireSession().getMetrics()); break;
      case "batch": {
        const controller = new AbortController();
        this.activeJobs.set(request.id, controller);
        try {
          const results = await this.requireSession().batch(request.requests, request.maxConcurrency, controller.signal);
          this.reply(request.id, results);
        } finally {
          this.activeJobs.delete(request.id);
        }
        break;
      }
      case "simulateDeviceLoss":
        this.presenter?.simulateDeviceLoss();
        this.reply(request.id, undefined);
        break;
      case "close":
        clearTimeout(this.refinement);
        this.session?.close();
        this.session = undefined;
        this.reply(request.id, undefined);
        break;
    }
  }

  private async flushPreview(): Promise<void> {
    this.previewScheduled = false;
    const request = this.pendingPreview;
    this.pendingPreview = undefined;
    if (!request) return;
    if (this.cancelled.delete(request.id)) {
      this.fail(request.id, "Preview cancelled");
      return;
    }
    this.activePreview = request.id;
    try {
      let frame: PreviewFrame;
      if (this.presenter && this.gpuAvailable && !request.includePixels && !request.viewport) {
        try {
          const gpu = await this.requireSession().previewLayers(request.quality, () => this.cancelled.has(request.id));
          const started = performance.now();
          await this.presenter.presentLayers(this.requireSession().manifest, gpu.layers, gpu.outputScale, gpu.sampling);
          frame = gpu.frame;
          frame.metrics.presentMs = performance.now() - started;
          frame.metrics.totalMs += frame.metrics.presentMs;
          frame.metrics.gpuBytes = gpu.layers.reduce((total, layer) => total + layer.raster.data.byteLength, frame.width * frame.height * 8);
        } catch (error) {
          if (this.cancelled.has(request.id)) throw error;
          this.gpuAvailable = false;
          frame = this.requireSession().preview(request.quality, "canvas2d", true);
          frame.backend = await this.presenter.present(frame);
          delete frame.pixels;
        }
      } else {
        frame = this.requireSession().preview(request.quality, this.presenter ? "webgpu" : "rgba", true);
        if (request.viewport) applyViewport(frame, request.viewport);
        if (this.presenter) {
          const started = performance.now();
          frame.backend = await this.presenter.present(frame);
          frame.metrics.backend = frame.backend;
          frame.metrics.presentMs = performance.now() - started;
          frame.metrics.totalMs += frame.metrics.presentMs;
        }
        if (!request.includePixels && this.presenter) delete frame.pixels;
      }
      this.replyFrame(request.id, frame);
      this.send({ id: 0, ok: true, event: "previewReady", value: { ...frame, pixels: undefined } });
    } catch (error) {
      this.fail(request.id, error instanceof Error ? error.message : String(error));
    } finally {
      this.cancelled.delete(request.id);
      if (this.activePreview === request.id) this.activePreview = undefined;
    }
  }

  private async emitRefinement(): Promise<void> {
    if (!this.session) return;
    try {
      let frame: PreviewFrame;
      if (this.presenter && this.gpuAvailable) {
        try {
          const gpu = await this.session.previewLayers("refined");
          await this.presenter.presentLayers(this.session.manifest, gpu.layers, gpu.outputScale, gpu.sampling);
          frame = gpu.frame;
        } catch {
          this.gpuAvailable = false;
          frame = this.session.preview("refined", "canvas2d", true);
          frame.backend = await this.presenter.present(frame);
          delete frame.pixels;
        }
      } else if (this.presenter) {
        frame = this.session.preview("refined", "canvas2d", true);
        frame.backend = await this.presenter.present(frame);
        delete frame.pixels;
      } else frame = this.session.preview("refined", "rgba", true);
      this.send({ id: 0, ok: true, event: "previewReady", value: frame });
    } catch (error) {
      this.send({ id: 0, ok: true, event: "diagnostic", value: { level: "warning", message: `Idle refinement failed: ${String(error)}` } });
    }
  }

  private replace(session: RetainedRenderSession): void { this.session?.close(); this.session = session; }
  private requireSession(): RetainedRenderSession { if (!this.session) throw new Error("Open or create a document first"); return this.session; }
  private reply(id: number, value: unknown, transfer: Transferable[] = []): void { this.send({ id, ok: true, value }, transfer); }
  private fail(id: number, error: string): void { this.send({ id, ok: false, error }); }
  private replyFrame(id: number, frame: PreviewFrame): void {
    const transfer = frame.pixels ? [frame.pixels.buffer as ArrayBuffer] : [];
    this.reply(id, frame, transfer);
  }
}

function applyViewport(frame: PreviewFrame, viewport: { x: number; y: number; width: number; height: number; zoom: number }): void {
  if (!frame.pixels || viewport.width <= 0 || viewport.height <= 0 || viewport.zoom <= 0) return;
  const left = Math.max(0, Math.floor(viewport.x));
  const top = Math.max(0, Math.floor(viewport.y));
  if (left >= frame.width || top >= frame.height) throw new Error("Viewport does not intersect the rendered canvas");
  const sourceWidth = Math.min(frame.width - left, Math.ceil(viewport.width));
  const sourceHeight = Math.min(frame.height - top, Math.ceil(viewport.height));
  const width = Math.max(1, Math.round(sourceWidth * viewport.zoom));
  const height = Math.max(1, Math.round(sourceHeight * viewport.zoom));
  if (width > 32_768 || height > 32_768 || width * height > 268_435_456) throw new Error("Viewport output exceeds resource limits");
  const output = new Uint8Array(width * height * 4);
  for (let y = 0; y < height; y++) for (let x = 0; x < width; x++) {
    const sourceX = left + Math.min(sourceWidth - 1, Math.floor(x / viewport.zoom));
    const sourceY = top + Math.min(sourceHeight - 1, Math.floor(y / viewport.zoom));
    const from = (sourceY * frame.width + sourceX) * 4;
    output.set(frame.pixels.subarray(from, from + 4), (y * width + x) * 4);
  }
  frame.width = width;
  frame.height = height;
  frame.pixels = output;
}

const scope = globalThis as unknown as DedicatedWorkerGlobalScope;
if (typeof scope.postMessage === "function" && !("document" in globalThis)) {
  const host = new RenderWorkerHost((response, transfer = []) => scope.postMessage(response, transfer));
  scope.onmessage = (event: MessageEvent<WorkerRequest>) => void host.receive(event.data);
}
