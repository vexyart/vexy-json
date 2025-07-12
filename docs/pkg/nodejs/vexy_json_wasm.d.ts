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
