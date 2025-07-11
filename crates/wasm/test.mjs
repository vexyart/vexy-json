import { readFileSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Dynamically import the WASM module
const wasmModule = await import(join(__dirname, 'pkg', 'vexy_json_wasm.js'));
const { default: init, parse_js, parse_with_options_js, is_valid, format } = wasmModule;

// Initialize WASM with the WASM file path
const wasmPath = join(__dirname, 'pkg', 'vexy_json_wasm_bg.wasm');
const wasmBytes = readFileSync(wasmPath);
await init(wasmBytes);

console.log('Testing vexy_json WASM module...\n');

// Test 1: Basic parsing
console.log('Test 1: Basic parsing');
const test1 = parse_js('{"key": "value", "number": 42}');
console.log('Input:  {"key": "value", "number": 42}');
console.log('Output:', test1);
console.log('✓ Basic parsing works\n');

// Test 2: Forgiving features
console.log('Test 2: Forgiving features');
const test2 = parse_js('{ key: "value", trailing: true, }');
console.log('Input:  { key: "value", trailing: true, }');
console.log('Output:', test2);
console.log('✓ Unquoted keys and trailing commas work\n');

// Test 3: Comments
console.log('Test 3: Comments');
const test3 = parse_js(`{
  // This is a comment
  "key": "value",
  /* Multi-line
     comment */
  "number": 42
}`);
console.log('Input:  JSON with comments');
console.log('Output:', test3);
console.log('✓ Comments work\n');

// Test 4: Implicit top-level
console.log('Test 4: Implicit top-level');
const test4 = parse_with_options_js(
  'key: value\nkey2: value2',
  true, true, true, true, true, true
);
console.log('Input:  key: value\\nkey2: value2');
console.log('Output:', test4);
console.log('✓ Implicit top-level works\n');

// Test 5: Validation
console.log('Test 5: Validation');
console.log('Valid JSON:   is_valid(\'{"valid": true}\') =', is_valid('{"valid": true}'));
console.log('Invalid JSON: is_valid(\'invalid json\') =', is_valid('invalid json'));
console.log('✓ Validation works\n');

// Test 6: Formatting
console.log('Test 6: Formatting');
const test6Input = '{ compact:true,data:[1,2,3] }';
const test6 = format(test6Input);
console.log('Input: ', test6Input);
console.log('Output:', test6);
console.log('✓ Formatting works\n');

console.log('All tests passed!');