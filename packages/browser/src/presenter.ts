import type { PreviewBackend, PreviewFrame } from "./protocol.js";
import type { Layer, Manifest, PixelBuffer, Sampling } from "@layered-graphics/core";

interface GpuState {
  adapter: any;
  device: any;
  context: any;
  format: string;
  pipeline: any;
  compositePipeline: any;
  sampler: any;
  smoothSampler: any;
  generation: number;
}

export interface GpuLayerInput { layer: Layer; raster: PixelBuffer }

export class PreviewPresenter {
  private gpu?: GpuState;
  private context2d?: OffscreenCanvasRenderingContext2D | CanvasRenderingContext2D | null;
  private lossCount = 0;
  private layerTextures = new Map<string, { data: Uint8Array; texture: any; bytes: number }>();
  private textureBytes = 0;

  readonly memoryBudgetBytes = 256 * 1024 * 1024;

  constructor(private readonly canvas: OffscreenCanvas | HTMLCanvasElement) {}

  async capability(): Promise<{ backend: PreviewBackend; webgpu: boolean; reason?: string }> {
    try {
      await this.ensureGpu();
      return { backend: "webgpu", webgpu: true };
    } catch (error) {
      return { backend: "canvas2d", webgpu: false, reason: error instanceof Error ? error.message : String(error) };
    }
  }

  async present(frame: PreviewFrame): Promise<PreviewBackend> {
    if (!frame.pixels) throw new Error("A pixel buffer is required for presentation");
    this.canvas.width = frame.width;
    this.canvas.height = frame.height;
    try {
      const gpu = await this.ensureGpu();
      this.presentGpu(gpu, frame);
      frame.metrics.gpuBytes = frame.pixels.byteLength;
      return "webgpu";
    } catch {
      this.present2d(frame);
      return "canvas2d";
    }
  }

  async presentLayers(manifest: Manifest, layers: GpuLayerInput[], outputScale: number, sampling: Sampling): Promise<PreviewBackend> {
    const width = Math.max(1, Math.round(manifest.canvas.width * outputScale));
    const height = Math.max(1, Math.round(manifest.canvas.height * outputScale));
    this.canvas.width = width;
    this.canvas.height = height;
    const gpu = await this.ensureGpu();
    const usage = 0x04 | 0x10; // TEXTURE_BINDING | RENDER_ATTACHMENT
    const makeTarget = () => gpu.device.createTexture({ size: [width, height], format: "rgba8unorm", usage });
    let read = makeTarget();
    let write = makeTarget();
    const encoder = gpu.device.createCommandEncoder();
    let pass = encoder.beginRenderPass({ colorAttachments: [{
      view: read.createView(),
      clearValue: rgbaToGpu(manifest.canvas.color),
      loadOp: "clear", storeOp: "store",
    }] });
    pass.end();
    const active = new Set<string>();
    const buffers: any[] = [];
    for (const input of layers) {
      if (!input.layer.visible || input.layer.opacity <= 0) continue;
      const key = `${input.layer.id}:${sampling}`;
      active.add(key);
      const source = this.sourceTexture(gpu, key, input.raster);
      const uniform = gpu.device.createBuffer({ size: 48, usage: 0x40 | 0x08 });
      buffers.push(uniform);
      const raw = new ArrayBuffer(48);
      const values = new Float32Array(raw);
      values.set([
        width, height, input.raster.width, input.raster.height,
        input.layer.transform.x * outputScale, input.layer.transform.y * outputScale,
        input.layer.transform.scaleX * outputScale, input.layer.transform.scaleY * outputScale,
        input.layer.opacity, 0, 0, 0,
      ]);
      new Uint32Array(raw)[9] = input.layer.blendMode === "multiply" ? 1 : 0;
      gpu.device.queue.writeBuffer(uniform, 0, raw);
      const bindGroup = gpu.device.createBindGroup({
        layout: gpu.compositePipeline.getBindGroupLayout(0),
        entries: [
          { binding: 0, resource: sampling === "smooth" ? gpu.smoothSampler : gpu.sampler },
          { binding: 1, resource: read.createView() },
          { binding: 2, resource: source.createView() },
          { binding: 3, resource: { buffer: uniform } },
        ],
      });
      pass = encoder.beginRenderPass({ colorAttachments: [{
        view: write.createView(), clearValue: { r: 0, g: 0, b: 0, a: 0 }, loadOp: "clear", storeOp: "store",
      }] });
      pass.setPipeline(gpu.compositePipeline);
      pass.setBindGroup(0, bindGroup);
      pass.draw(3);
      pass.end();
      [read, write] = [write, read];
    }
    const finalGroup = gpu.device.createBindGroup({
      layout: gpu.pipeline.getBindGroupLayout(0),
      entries: [{ binding: 0, resource: gpu.sampler }, { binding: 1, resource: read.createView() }],
    });
    pass = encoder.beginRenderPass({ colorAttachments: [{
      view: gpu.context.getCurrentTexture().createView(), clearValue: { r: 0, g: 0, b: 0, a: 0 }, loadOp: "clear", storeOp: "store",
    }] });
    pass.setPipeline(gpu.pipeline);
    pass.setBindGroup(0, finalGroup);
    pass.draw(3);
    pass.end();
    gpu.device.queue.submit([encoder.finish()]);
    read.destroy();
    write.destroy();
    for (const buffer of buffers) buffer.destroy();
    for (const [key, cached] of this.layerTextures) if (!active.has(key)) {
      cached.texture.destroy();
      this.layerTextures.delete(key);
      this.textureBytes -= cached.bytes;
    }
    return "webgpu";
  }

  simulateDeviceLoss(): void {
    this.gpu?.device.destroy();
    this.gpu = undefined;
    this.clearTextures();
  }

  get deviceLossCount(): number { return this.lossCount; }

  private async ensureGpu(): Promise<GpuState> {
    if (this.gpu) return this.gpu;
    const gpuApi = (globalThis.navigator as Navigator & { gpu?: any }).gpu;
    if (!gpuApi) throw new Error("WebGPU is unavailable");
    const adapter = await gpuApi.requestAdapter({ powerPreference: "high-performance" });
    if (!adapter) throw new Error("No WebGPU adapter is available");
    const device = await adapter.requestDevice();
    const context = this.canvas.getContext("webgpu" as never) as any;
    if (!context) throw new Error("The canvas cannot create a WebGPU context");
    const format = gpuApi.getPreferredCanvasFormat();
    context.configure({ device, format, alphaMode: "premultiplied" });
    const module = device.createShaderModule({ code: SHADER });
    const compositeModule = device.createShaderModule({ code: COMPOSITE_SHADER });
    const pipeline = device.createRenderPipeline({
      layout: "auto",
      vertex: { module, entryPoint: "vertex_main" },
      fragment: { module, entryPoint: "fragment_main", targets: [{ format }] },
      primitive: { topology: "triangle-list" },
    });
    const compositePipeline = device.createRenderPipeline({
      layout: "auto",
      vertex: { module: compositeModule, entryPoint: "vertex_main" },
      fragment: { module: compositeModule, entryPoint: "fragment_main", targets: [{ format: "rgba8unorm" }] },
      primitive: { topology: "triangle-list" },
    });
    const generation = this.lossCount + 1;
    const state = {
      adapter, device, context, format, pipeline, compositePipeline,
      sampler: device.createSampler({ magFilter: "nearest", minFilter: "nearest" }),
      smoothSampler: device.createSampler({ magFilter: "linear", minFilter: "linear" }), generation,
    };
    device.lost.then(() => {
      if (this.gpu?.device === device) this.gpu = undefined;
      this.clearTextures();
      this.lossCount++;
    });
    this.gpu = state;
    return state;
  }

  private sourceTexture(gpu: GpuState, key: string, raster: PixelBuffer): any {
    const cached = this.layerTextures.get(key);
    if (cached?.data === raster.data) return cached.texture;
    if (cached) {
      cached.texture.destroy();
      this.textureBytes -= cached.bytes;
    }
    const texture = gpu.device.createTexture({ size: [raster.width, raster.height], format: "rgba8unorm", usage: 0x04 | 0x02 });
    writeTexture(gpu.device, texture, raster.width, raster.height, raster.data);
    const bytes = raster.data.byteLength;
    this.layerTextures.set(key, { data: raster.data, texture, bytes });
    this.textureBytes += bytes;
    while (this.textureBytes > this.memoryBudgetBytes && this.layerTextures.size > 1) {
      const first = this.layerTextures.entries().next().value as [string, { texture: any; bytes: number }] | undefined;
      if (!first || first[0] === key) break;
      first[1].texture.destroy();
      this.layerTextures.delete(first[0]);
      this.textureBytes -= first[1].bytes;
    }
    return texture;
  }

  private clearTextures(): void {
    for (const value of this.layerTextures.values()) value.texture.destroy();
    this.layerTextures.clear();
    this.textureBytes = 0;
  }

  private presentGpu(gpu: GpuState, frame: PreviewFrame): void {
    const texture = gpu.device.createTexture({
      size: [frame.width, frame.height], format: "rgba8unorm",
      usage: 0x04 | 0x02, // TEXTURE_BINDING | COPY_DST
    });
    writeTexture(gpu.device, texture, frame.width, frame.height, frame.pixels!);
    const bindGroup = gpu.device.createBindGroup({
      layout: gpu.pipeline.getBindGroupLayout(0),
      entries: [{ binding: 0, resource: gpu.sampler }, { binding: 1, resource: texture.createView() }],
    });
    const encoder = gpu.device.createCommandEncoder();
    const pass = encoder.beginRenderPass({ colorAttachments: [{ view: gpu.context.getCurrentTexture().createView(), clearValue: { r: 0, g: 0, b: 0, a: 0 }, loadOp: "clear", storeOp: "store" }] });
    pass.setPipeline(gpu.pipeline);
    pass.setBindGroup(0, bindGroup);
    pass.draw(3);
    pass.end();
    gpu.device.queue.submit([encoder.finish()]);
    texture.destroy();
  }

  private present2d(frame: PreviewFrame): void {
    this.context2d ??= this.canvas.getContext("2d") as OffscreenCanvasRenderingContext2D | CanvasRenderingContext2D | null;
    if (!this.context2d) throw new Error("Neither WebGPU nor Canvas2D is available");
    const pixels = new Uint8ClampedArray(frame.pixels!.byteLength);
    pixels.set(frame.pixels!);
    this.context2d.putImageData(new ImageData(pixels, frame.width, frame.height), 0, 0);
  }
}

const SHADER = `
@group(0) @binding(0) var image_sampler: sampler;
@group(0) @binding(1) var image_texture: texture_2d<f32>;

struct VertexOutput { @builtin(position) position: vec4f, @location(0) uv: vec2f };

@vertex fn vertex_main(@builtin(vertex_index) index: u32) -> VertexOutput {
  var positions = array<vec2f, 3>(vec2f(-1.0, -1.0), vec2f(3.0, -1.0), vec2f(-1.0, 3.0));
  var uvs = array<vec2f, 3>(vec2f(0.0, 1.0), vec2f(2.0, 1.0), vec2f(0.0, -1.0));
  var output: VertexOutput;
  output.position = vec4f(positions[index], 0.0, 1.0);
  output.uv = uvs[index];
  return output;
}

@fragment fn fragment_main(input: VertexOutput) -> @location(0) vec4f {
  return textureSample(image_texture, image_sampler, input.uv);
}`;

function writeTexture(device: any, texture: any, width: number, height: number, pixels: Uint8Array): void {
  const row = width * 4;
  const paddedRow = Math.ceil(row / 256) * 256;
  let bytes = pixels;
  if (paddedRow !== row) {
    bytes = new Uint8Array(paddedRow * height);
    for (let y = 0; y < height; y++) bytes.set(pixels.subarray(y * row, (y + 1) * row), y * paddedRow);
  }
  device.queue.writeTexture({ texture }, bytes, { bytesPerRow: paddedRow, rowsPerImage: height }, [width, height]);
}

function rgbaToGpu(color: [number, number, number, number]): { r: number; g: number; b: number; a: number } {
  return { r: color[0] / 255, g: color[1] / 255, b: color[2] / 255, a: color[3] / 255 };
}

const COMPOSITE_SHADER = `
@group(0) @binding(0) var image_sampler: sampler;
@group(0) @binding(1) var backdrop_texture: texture_2d<f32>;
@group(0) @binding(2) var source_texture: texture_2d<f32>;

struct Params {
  canvas: vec2f,
  source: vec2f,
  position: vec2f,
  scale: vec2f,
  opacity: f32,
  blend_mode: u32,
  padding: vec2f,
};
@group(0) @binding(3) var<uniform> params: Params;

struct VertexOutput { @builtin(position) position: vec4f, @location(0) uv: vec2f };
@vertex fn vertex_main(@builtin(vertex_index) index: u32) -> VertexOutput {
  var positions = array<vec2f, 3>(vec2f(-1.0, -1.0), vec2f(3.0, -1.0), vec2f(-1.0, 3.0));
  var uvs = array<vec2f, 3>(vec2f(0.0, 1.0), vec2f(2.0, 1.0), vec2f(0.0, -1.0));
  var output: VertexOutput;
  output.position = vec4f(positions[index], 0.0, 1.0);
  output.uv = uvs[index];
  return output;
}

@fragment fn fragment_main(input: VertexOutput) -> @location(0) vec4f {
  let backdrop = textureSample(backdrop_texture, image_sampler, input.uv);
  let pixel = input.uv * params.canvas;
  let extent = params.source * abs(params.scale);
  let local_pixel = pixel - params.position;
  if (local_pixel.x < 0.0 || local_pixel.y < 0.0 || local_pixel.x >= extent.x || local_pixel.y >= extent.y) {
    return backdrop;
  }
  var source_uv = local_pixel / extent;
  if (params.scale.x < 0.0) { source_uv.x = 1.0 - source_uv.x; }
  if (params.scale.y < 0.0) { source_uv.y = 1.0 - source_uv.y; }
  let source = textureSample(source_texture, image_sampler, source_uv);
  let source_alpha = source.a * clamp(params.opacity, 0.0, 1.0);
  let output_alpha = source_alpha + backdrop.a * (1.0 - source_alpha);
  if (output_alpha <= 0.000001) { return vec4f(0.0); }
  var blended = source.rgb;
  if (params.blend_mode == 1u) { blended = source.rgb * backdrop.rgb; }
  let premultiplied = (1.0 - source_alpha) * backdrop.a * backdrop.rgb
    + (1.0 - backdrop.a) * source_alpha * source.rgb
    + source_alpha * backdrop.a * blended;
  return vec4f(clamp(premultiplied / output_alpha, vec3f(0.0), vec3f(1.0)), output_alpha);
}`;
