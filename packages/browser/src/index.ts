import type { BatchRequest, BatchResult, Command, CommandResult, Manifest, OutputFormat, PreviewFrame, QualityIntent, RenderMetrics, RenderOptions, SessionEvent, Viewport, WorkerRequest, WorkerResponse } from "./protocol.js";
export { RetainedRenderSession } from "./retained-session.js";
export { PreviewPresenter } from "./presenter.js";
export type * from "./protocol.js";

type WorkerFactory = () => Worker;
type RequestWithoutId = WorkerRequest extends infer Request
  ? Request extends { id: number }
    ? Omit<Request, "id">
    : never
  : never;

export class BrowserRenderSession extends EventTarget {
  private worker: Worker;
  private nextId = 1;
  private pending = new Map<number, { resolve(value: unknown): void; reject(error: Error): void }>();
  private snapshot?: Uint8Array;
  private closed = false;
  private recovering = false;
  private directCanvas = false;
  private linkedAssets = new Map<string, Uint8Array>();

  private constructor(private readonly workerFactory: WorkerFactory) {
    super();
    this.worker = this.bind(workerFactory());
  }

  static async create(workerFactory: WorkerFactory, options: { width: number; height: number; dpi?: number; canvas?: HTMLCanvasElement }): Promise<BrowserRenderSession> {
    const session = new BrowserRenderSession(workerFactory);
    const canvas = options.canvas?.transferControlToOffscreen();
    session.directCanvas = Boolean(canvas);
    await session.call({ type: "create", width: options.width, height: options.height, dpi: options.dpi, canvas }, canvas ? [canvas] : []);
    await session.refreshSnapshot();
    return session;
  }

  static async open(workerFactory: WorkerFactory, bytes: Uint8Array, canvas?: HTMLCanvasElement): Promise<BrowserRenderSession> {
    const session = new BrowserRenderSession(workerFactory);
    session.snapshot = bytes.slice();
    const offscreen = canvas?.transferControlToOffscreen();
    session.directCanvas = Boolean(offscreen);
    await session.call({ type: "open", bytes: bytes.slice(), canvas: offscreen }, offscreen ? [offscreen] : []);
    return session;
  }

  execute(commands: Command | Command[], label?: string): Promise<CommandResult> {
    return this.call({ type: "execute", commands, label }).then(async (value) => {
      await this.refreshSnapshot();
      return value as CommandResult;
    });
  }

  async provideLinkedAsset(id: string, bytes: Uint8Array): Promise<void> {
    this.linkedAssets.set(id, bytes.slice());
    const copy = bytes.slice();
    await this.call({ type: "provideLinkedAsset", assetId: id, bytes: copy }, [copy.buffer]);
    await this.refreshSnapshot();
  }

  preview(quality: QualityIntent = "preview", options: { viewport?: Viewport; includePixels?: boolean; signal?: AbortSignal } = {}): Promise<PreviewFrame> {
    const id = this.nextId;
    const request = this.callWithId(id, { id, type: "preview", quality, viewport: options.viewport, includePixels: options.includePixels ?? !this.directCanvas }) as Promise<PreviewFrame>;
    if (options.signal) {
      const cancel = () => void this.call({ type: "cancel", target: id }).catch(() => undefined);
      if (options.signal.aborted) cancel(); else options.signal.addEventListener("abort", cancel, { once: true });
    }
    return request;
  }

  export(format: OutputFormat = "png", options?: RenderOptions): Promise<Uint8Array> { return this.call({ type: "export", format, options }) as Promise<Uint8Array>; }
  manifest(): Promise<Manifest> { return this.call({ type: "manifest" }) as Promise<Manifest>; }
  metrics(): Promise<RenderMetrics> { return this.call({ type: "metrics" }) as Promise<RenderMetrics>; }
  async batch(requests: BatchRequest[], maxConcurrency = 2, onProgress?: (completed: number, total: number) => void, signal?: AbortSignal): Promise<BatchResult[]> {
    if (!Number.isInteger(maxConcurrency) || maxConcurrency < 1 || maxConcurrency > 8) throw new Error("maxConcurrency must be an integer from 1 to 8");
    const output: BatchResult[] = [];
    for (let offset = 0; offset < requests.length; offset += maxConcurrency) {
      const chunk = requests.slice(offset, offset + maxConcurrency);
      const id = this.nextId++;
      const pending = this.callWithId(id, { id, type: "batch", requests: chunk, maxConcurrency }) as Promise<BatchResult[]>;
      const cancel = () => void this.call({ type: "cancel", target: id }).catch(() => undefined);
      if (signal?.aborted) cancel(); else signal?.addEventListener("abort", cancel, { once: true });
      const results = await pending;
      output.push(...results.map((result) => ({ ...result, index: result.index + offset })));
      onProgress?.(output.length, requests.length);
      await Promise.resolve();
    }
    return output;
  }
  simulateDeviceLoss(): Promise<void> { return this.call({ type: "simulateDeviceLoss" }) as Promise<void>; }

  subscribe(listener: (event: SessionEvent) => void): () => void {
    const wrapped = (event: Event) => listener((event as CustomEvent<SessionEvent>).detail);
    this.addEventListener("session", wrapped);
    return () => this.removeEventListener("session", wrapped);
  }

  async close(): Promise<void> {
    if (this.closed) return;
    await this.call({ type: "close" }).catch(() => undefined);
    this.closed = true;
    this.worker.terminate();
    for (const waiter of this.pending.values()) waiter.reject(new Error("Render session closed"));
    this.pending.clear();
  }

  private call(request: RequestWithoutId, transfer: Transferable[] = []): Promise<unknown> {
    const id = this.nextId++;
    return this.callWithId(id, { ...request, id } as WorkerRequest, transfer);
  }

  private callWithId(id: number, request: WorkerRequest, transfer: Transferable[] = []): Promise<unknown> {
    if (this.closed) return Promise.reject(new Error("Render session closed"));
    this.nextId = Math.max(this.nextId, id + 1);
    return new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
      this.worker.postMessage(request, transfer);
    });
  }

  private bind(worker: Worker): Worker {
    worker.onmessage = (event: MessageEvent<WorkerResponse>) => this.receive(event.data);
    worker.onerror = () => void this.recover();
    worker.onmessageerror = () => void this.recover();
    return worker;
  }

  private receive(response: WorkerResponse): void {
    if (response.id === 0 && response.ok && "event" in response) {
      const detail = { type: response.event, value: response.value } as SessionEvent;
      this.dispatchEvent(new CustomEvent("session", { detail }));
      return;
    }
    const waiter = this.pending.get(response.id);
    if (!waiter) return;
    this.pending.delete(response.id);
    if (response.ok) waiter.resolve(response.value); else waiter.reject(new Error(response.error));
  }

  private async refreshSnapshot(): Promise<void> {
    const bytes = await this.call({ type: "snapshot" }) as Uint8Array;
    this.snapshot = bytes.slice();
  }

  private async recover(): Promise<void> {
    if (this.recovering || this.closed) return;
    this.recovering = true;
    const error = new Error("Render worker restarted; in-flight work was cancelled");
    for (const waiter of this.pending.values()) waiter.reject(error);
    this.pending.clear();
    this.worker.terminate();
    this.worker = this.bind(this.workerFactory());
    try {
      if (!this.snapshot) throw new Error("No canonical snapshot is available");
      await this.call({ type: "open", bytes: this.snapshot.slice() });
      for (const [assetId, bytes] of this.linkedAssets) {
        const copy = bytes.slice();
        await this.call({ type: "provideLinkedAsset", assetId, bytes: copy }, [copy.buffer]);
      }
      this.dispatchEvent(new CustomEvent("session", { detail: { type: "diagnostic", value: { level: "warning", message: "Render worker recovered from the last canonical revision" } } satisfies SessionEvent }));
    } finally {
      this.recovering = false;
    }
  }
}

export function moduleWorkerFactory(url = new URL("./worker.js", import.meta.url)): WorkerFactory {
  return () => new Worker(url, { type: "module", name: "layered-graphics-renderer" });
}
