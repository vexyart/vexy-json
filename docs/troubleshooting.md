---
title: Troubleshooting
layout: default
---

# Troubleshooting

This page documents common issues and their solutions when using vexy_json, particularly with WebAssembly bindings.

## WebAssembly Issues

### Objects Parsing to Empty Results

**Issue**: Parsed JSON objects appear empty (`{}`) even when the input contains valid data like `{a:1}` or `{"a":1}`.

**Symptoms**:
- `Object.keys(result)` returns an empty array
- `JSON.stringify(result)` returns `"{}"`
- Property access on parsed objects returns `undefined`
- Browser console shows results as `Map(1)` instead of plain objects

**Root Cause**: This was a critical bug in versions prior to 1.2.4 where the WebAssembly bindings used `serde_wasm_bindgen::to_value()` which converted Rust `HashMap` objects to JavaScript `Map` objects instead of plain JavaScript objects.

**Solution**: 
- **Fixed in version 1.2.4**: The WebAssembly bindings now use a custom `value_to_js()` function that creates proper JavaScript objects
- **If using an older version**: Upgrade to version 1.2.4 or later

**Technical Details**:
The fix involved replacing the automatic serde conversion with manual object creation:

```rust
// Before (problematic):
serde_wasm_bindgen::to_value(&value)

// After (fixed):
value_to_js(&value) // Custom function using js_sys::Object
```

### Browser Caching of WASM Modules

**Issue**: Changes to the WASM module are not reflected in the browser even after rebuilding.

**Solution**:
1. Hard refresh your browser (Ctrl+Shift+R or Cmd+Shift+R)
2. Clear browser cache
3. Add cache-busting query parameters to module imports:
   ```javascript
   import init from './pkg/vexy_json_wasm.js?v=' + Date.now();
   ```

### WASM Module Loading Failures

**Issue**: WebAssembly module fails to load with network errors.

**Common Causes & Solutions**:

1. **Incorrect MIME type**: Ensure your web server serves `.wasm` files with `application/wasm` MIME type
2. **CORS issues**: Serve files from a proper HTTP server, not file:// protocol
3. **Path issues**: Verify the path to `pkg/vexy_json_wasm.js` and `pkg/vexy_json_bg.wasm` is correct

**Testing Setup**:
Use a simple HTTP server for testing:
```bash
# Python 3
python -m http.server 8080

# Node.js (with http-server package)
npx http-server -p 8080

# Rust (with basic-http-server)
cargo install basic-http-server
basic-http-server docs/ -a 127.0.0.1:8080
```

## Parser Issues

### Unquoted Keys Not Working

**Issue**: JSON with unquoted keys like `{key: "value"}` fails to parse.

**Solution**: Ensure `allow_unquoted_keys` is enabled in parser options:

```javascript
const options = {
  allow_unquoted_keys: true,
  // ... other options
};
const result = parse_json_with_options(input, options);
```

### Comments Causing Parse Errors

**Issue**: JSON with comments like `// comment` or `/* comment */` fails to parse.

**Solution**: Enable comment support in parser options:

```javascript
const options = {
  allow_comments: true,
  // ... other options
};
const result = parse_json_with_options(input, options);
```

## Debug Tools

### Browser Console Debugging

Enable debug logging by using the debug builds of the WebAssembly module. Debug messages will appear in the browser console showing:

- Token parsing progress
- Value conversion steps  
- Object creation details

### Test Pages

The following test pages are available for debugging:

- `error-debug.html` - Error handling and basic parsing tests
- `console-debug.html` - Console output capture and display
- `token-debug.html` - Token-level parsing analysis
- `deep-debug.html` - Comprehensive parsing verification

### Manual Testing

Test parsing functionality manually:

```javascript
// Test basic object parsing
const result1 = parse_json('{"a": 1}');
console.log('Quoted keys:', result1);

// Test unquoted keys (requires options)
const options = { allow_unquoted_keys: true };
const result2 = parse_json_with_options('{a: 1}', options);
console.log('Unquoted keys:', result2);

// Verify object properties
console.log('Keys:', Object.keys(result2));
console.log('JSON:', JSON.stringify(result2));
```

## Getting Help

If you encounter issues not covered here:

1. Check the [GitHub Issues](https://github.com/twardoch/vexy_json/issues)
2. Review the [API documentation](api.md)
3. Examine the [test files](https://github.com/twardoch/vexy_json/tree/main/tests) for usage examples
4. Create a new issue with:
   - Your vexy_json version
   - Browser and version
   - Minimal reproduction case
   - Expected vs actual behavior
