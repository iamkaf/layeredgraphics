import initWasm, {
  inspect_kgfx,
  render_kgfx_png,
  validate_kgfx,
  version,
} from "./generated/lg_wasm.js";

export type WasmInitInput = Parameters<typeof initWasm>[0];

let initialized: Promise<unknown> | undefined;

export function initialize(input?: WasmInitInput): Promise<unknown> {
  initialized ??= initWasm(input);
  return initialized;
}

export function engineVersion(): string {
  return version();
}

export function inspectKgfx(bytes: Uint8Array): unknown {
  return JSON.parse(inspect_kgfx(bytes));
}

export function validateKgfx(bytes: Uint8Array): unknown[] {
  return JSON.parse(validate_kgfx(bytes));
}

export function renderKgfxPng(bytes: Uint8Array, scale = 1): Uint8Array {
  return render_kgfx_png(bytes, scale);
}
