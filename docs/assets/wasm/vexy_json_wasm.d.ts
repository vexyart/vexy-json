/* tslint:disable */
/* eslint-disable */
/**
 * Parse a JSON/Vexy JSON string and return the result as a JSON string
 */
export function parse_json(input: string): string;
/**
 * Parse a JSON/Vexy JSON string with custom options
 */
export function parse_json_with_options(input: string, allow_comments: boolean, allow_trailing_commas: boolean, allow_unquoted_keys: boolean, allow_single_quotes: boolean, implicit_top_level: boolean, newline_as_comma: boolean, enable_repair: boolean, max_depth?: number | null): string;
/**
 * Validate if a string is valid JSON/Vexy JSON
 */
export function validate_json(input: string): boolean;
/**
 * Get parser options as a JSON object
 */
export function get_parser_options(): string;
/**
 * Stringify a JSON value with pretty printing
 */
export function stringify_value(input: string, indent?: number | null): string;
/**
 * Get version information
 */
export function get_version_info(): string;
/**
 * Legacy function names for backward compatibility
 */
export function parse_js(input: string): string;
export function parse_with_options_js(input: string, allow_comments: boolean, allow_trailing_commas: boolean, allow_unquoted_keys: boolean, allow_single_quotes: boolean, implicit_top_level: boolean, newline_as_comma: boolean): string;
export function is_valid(input: string): boolean;
export function format(input: string): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly parse_json: (a: number, b: number) => [number, number, number, number];
  readonly parse_json_with_options: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => [number, number, number, number];
  readonly validate_json: (a: number, b: number) => number;
  readonly get_parser_options: () => [number, number, number, number];
  readonly stringify_value: (a: number, b: number, c: number) => [number, number, number, number];
  readonly get_version_info: () => [number, number, number, number];
  readonly parse_js: (a: number, b: number) => [number, number, number, number];
  readonly parse_with_options_js: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
  readonly is_valid: (a: number, b: number) => number;
  readonly format: (a: number, b: number) => [number, number, number, number];
  readonly __wbindgen_export_0: WebAssembly.Table;
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
