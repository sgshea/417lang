/* tslint:disable */
/* eslint-disable */
/**
 * Interprets a string
 * Returns the result of interpreting in string form
 * Expects the string to be valid JSON input
 * @param {string} input
 * @param {boolean} lexical_scope
 * @returns {string}
 */
export function interpret_to_string(input: string, lexical_scope: boolean): string;
/**
 * Parses a string
 * Returns the result of parsing in string form
 * @param {string} input
 * @returns {string}
 */
export function parse_to_string(input: string): string;
/**
 * Parses and then interprets a string
 * Returns the result of parsing and interpreting in string form
 * Same as above but exported for WASM
 * @param {string} input
 * @param {boolean} lexical_scope
 * @returns {string}
 */
export function interpret_with_parser_to_string(input: string, lexical_scope: boolean): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly interpret_to_string: (a: number, b: number, c: number) => Array;
  readonly parse_to_string: (a: number, b: number) => Array;
  readonly interpret_with_parser_to_string: (a: number, b: number, c: number) => Array;
  readonly __wbindgen_export_0: WebAssembly.Table;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
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
