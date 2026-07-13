/* tslint:disable */
/* eslint-disable */

export class Document {
    free(): void;
    [Symbol.dispose](): void;
    execute(operations_json: string, label?: string | null): string;
    exportKgfx(): Uint8Array;
    historyState(): string;
    manifest(): string;
    constructor(width: number, height: number, dpi: number);
    static open(bytes: Uint8Array): Document;
    provideLinkedAsset(id: string, bytes: Uint8Array): void;
    rasterizeLayer(id: string, sampling: string): Uint8Array;
    redo(): string;
    render(format: string, options_json?: string | null): Uint8Array;
    renderRetained(format: string, options_json?: string | null): Uint8Array;
    renderRgba(options_json?: string | null): Uint8Array;
    retainedMetrics(): string;
    undo(): string;
}

export function inspect_kgfx(bytes: Uint8Array): string;

export function render_kgfx_png(bytes: Uint8Array, scale: number): Uint8Array;

export function validate_kgfx(bytes: Uint8Array): string;

export function version(): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_document_free: (a: number, b: number) => void;
    readonly document_execute: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly document_exportKgfx: (a: number) => [number, number, number, number];
    readonly document_historyState: (a: number) => [number, number, number, number];
    readonly document_manifest: (a: number) => [number, number, number, number];
    readonly document_new: (a: number, b: number, c: number) => number;
    readonly document_open: (a: number, b: number) => [number, number, number];
    readonly document_provideLinkedAsset: (a: number, b: number, c: number, d: number, e: number) => [number, number];
    readonly document_rasterizeLayer: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly document_redo: (a: number) => [number, number, number, number];
    readonly document_render: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly document_renderRetained: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly document_renderRgba: (a: number, b: number, c: number) => [number, number, number, number];
    readonly document_retainedMetrics: (a: number) => [number, number, number, number];
    readonly document_undo: (a: number) => [number, number, number, number];
    readonly inspect_kgfx: (a: number, b: number) => [number, number, number, number];
    readonly render_kgfx_png: (a: number, b: number, c: number) => [number, number, number, number];
    readonly validate_kgfx: (a: number, b: number) => [number, number, number, number];
    readonly version: () => [number, number];
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
