import assert from "node:assert/strict";
import test from "node:test";
import { BrowserRenderSession, PreviewPresenter } from "../dist/index.js";
import { RenderWorkerHost } from "../dist/worker.js";

test("render worker coalesces obsolete preview requests", async () => {
  const messages = [];
  const host = new RenderWorkerHost((message) => messages.push(message));
  void host.receive({ id: 1, type: "preview", quality: "interactive", includePixels: true });
  void host.receive({ id: 2, type: "preview", quality: "preview", includePixels: true });
  await new Promise((resolve) => setTimeout(resolve, 10));
  assert.deepEqual(messages.find((message) => message.id === 1), {
    id: 1,
    ok: false,
    error: "Preview superseded by a newer request",
  });
  assert.match(messages.find((message) => message.id === 2).error, /Open or create/);
});

test("WebGPU presenter recreates resources after simulated device loss", async () => {
  const originalNavigator = globalThis.navigator;
  let deviceRequests = 0;
  const context = {
    configure() {},
    getCurrentTexture() { return { createView() { return {}; } }; },
  };
  const makeDevice = () => ({
    lost: new Promise(() => {}),
    destroy() {},
    queue: { writeTexture() {}, writeBuffer() {}, submit() {} },
    createShaderModule() { return {}; },
    createRenderPipeline() { return { getBindGroupLayout() { return {}; } }; },
    createSampler() { return {}; },
    createTexture() { return { createView() { return {}; }, destroy() {} }; },
    createBuffer() { return { destroy() {} }; },
    createBindGroup() { return {}; },
    createCommandEncoder() { return {
      beginRenderPass() { return { setPipeline() {}, setBindGroup() {}, draw() {}, end() {} }; },
      finish() { return {}; },
    }; },
  });
  Object.defineProperty(globalThis, "navigator", { configurable: true, value: {
    gpu: {
      async requestAdapter() { return { async requestDevice() { deviceRequests++; return makeDevice(); } }; },
      getPreferredCanvasFormat() { return "rgba8unorm"; },
    },
  } });
  try {
    const presenter = new PreviewPresenter({ width: 1, height: 1, getContext: () => context });
    const frame = {
      revision: 0, width: 1, height: 1, tier: "preview", backend: "webgpu", pixels: new Uint8Array([1, 2, 3, 255]),
      metrics: { gpuBytes: 0 },
    };
    const manifest = { canvas: { width: 1, height: 1, dpi: 72, color: [0, 0, 0, 0] } };
    const layer = { id: "fill", name: "Fill", visible: true, opacity: 1, blendMode: "normal", transform: { x: 0, y: 0, scaleX: 1, scaleY: 1 }, type: "fill", width: 1, height: 1, color: [1, 2, 3, 255] };
    assert.equal(await presenter.presentLayers(manifest, [{ layer, raster: { width: 1, height: 1, data: frame.pixels } }], 1, "nearest"), "webgpu");
    presenter.simulateDeviceLoss();
    assert.equal(await presenter.presentLayers(manifest, [{ layer, raster: { width: 1, height: 1, data: frame.pixels } }], 1, "nearest"), "webgpu");
    assert.equal(deviceRequests, 2);
  } finally {
    Object.defineProperty(globalThis, "navigator", { configurable: true, value: originalNavigator });
  }
});

test("browser session reopens the last canonical snapshot after worker failure", async () => {
  class FakeWorker {
    messages = [];
    terminated = false;
    postMessage(request) {
      this.messages.push(request);
      const value = request.type === "snapshot" ? new Uint8Array([80, 75, 3, 4]) : {};
      queueMicrotask(() => this.onmessage?.({ data: { id: request.id, ok: true, value } }));
    }
    terminate() { this.terminated = true; }
  }
  const workers = [];
  const factory = () => {
    const worker = new FakeWorker();
    workers.push(worker);
    return worker;
  };
  const session = await BrowserRenderSession.create(factory, { width: 8, height: 8 });
  let recovered = false;
  const unsubscribe = session.subscribe((event) => {
    if (event.type === "diagnostic" && event.value.message.includes("recovered")) recovered = true;
  });
  workers[0].onerror?.(new Event("error"));
  await new Promise((resolve) => setTimeout(resolve, 10));
  assert.equal(workers.length, 2);
  assert.equal(workers[0].terminated, true);
  assert.equal(workers[1].messages.some((message) => message.type === "open"), true);
  assert.equal(recovered, true);
  unsubscribe();
  await session.close();
});
