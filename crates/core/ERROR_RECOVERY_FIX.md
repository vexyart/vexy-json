# Error Recovery Fix for Optimized Parsers

## Issue
The optimized parsers (v1 and v2) were failing to parse JSON with trailing commas in arrays and objects, even when `allow_trailing_commas` was set to true in ParserOptions.

## Root Cause
After consuming a comma in arrays/objects, the parsers were immediately trying to parse the next value without first checking if the container was ending (with `]` or `}`). This caused an error when encountering trailing commas like `[1, 2, 3,]`.

## Solution
Added checks after consuming commas and skipping newlines to detect closing brackets/braces before attempting to parse values:

```rust
// After skipping newlines following a comma
let (next_token, _) = self.peek_token()?;
if next_token == Token::RightBracket && self.options.allow_trailing_commas {
    self.next_token()?;
    break;
}
```

## Files Modified
- `src/parser/optimized.rs` - Fixed array and object parsing
- `src/parser/optimized_v2.rs` - Fixed array and object parsing

## Test Results
Both optimized parsers now successfully parse malformed JSON with:
- Single quotes: `'name': 'John'`
- Unquoted keys: `age: 30`
- Trailing commas: `[1, 2, 3,]`

Example test case:
```json
{'name': 'John', age: 30, "items": [1, 2, 3,]}
```

Both parsers now handle this correctly when the appropriate options are enabled.