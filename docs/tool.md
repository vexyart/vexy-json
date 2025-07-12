---
layout: default
title: Interactive Tools
nav_order: 4
permalink: /tools/
this_file: docs/tool.md
---

# Interactive Parsing Tools

Choose from our collection of JSON parsing tools with identical design and user experience:

## [∞](#) 
Interactive parser for Vexy JSON with forgiving JSON syntax and WebAssembly-powered performance.

- Comments (`//`, `#`, `/* */`) support
- Unquoted keys and single quotes
- Trailing commas and implicit structures
- Real-time validation and error reporting

## [∞](#) 
Interactive parser for Jsonic with flexible JSON syntax and advanced features.

- Object merging capabilities
- Property chain syntax
- Multi-line string handling
- Plugin system support

<div style="text-align: center; margin: 2em 0;">
  <a href="{{ '/vexy_json-tool/' | relative_url }}" class="btn btn-primary" style="font-size: 1.1em; padding: 0.7em 1.5em; margin: 0.5em;">
    🚀 Vexy JSON Tool
  </a>
  <a href="{{ '/vexy-json-tool/' | relative_url }}" class="btn btn-secondary" style="font-size: 1.1em; padding: 0.7em 1.5em; margin: 0.5em;">
    🔧 Vexy JSON Tool
  </a>
</div>

## [∞](#features-comparison) Features Comparison

Both tools share identical interface design with:

- ✅ **Real-time parsing** - See results as you type
- ✅ **Syntax highlighting** - Clear visualization of your JSON
- ✅ **Error highlighting** - Precise error messages with position indicators
- ✅ **Parser options** - Toggle individual features on/off
- ✅ **Example templates** - Pre-loaded examples to get started
- ✅ **Share URLs** - Share your JSON snippets with others
- ✅ **Download results** - Save parsed JSON to a file
- ✅ **Dark/light themes** - Choose your preferred color scheme
- ✅ **Mobile responsive** - Works on all devices

## [∞](#vexy_json-specific-features) Vexy JSON Specific Features

- Single-line comments (`//` and `#`)
- Multi-line comments (`/* */`)
- Unquoted object keys
- Single-quoted strings
- Trailing commas
- Implicit top-level objects and arrays
- Newline as comma separator
- Extended number formats (hex, octal, binary, underscores)

## [∞](#jsonic-specific-features) Vexy JSON Specific Features

- Object merging: `a:{b:1},a:{c:2}` → `{a:{b:1,c:2}}`
- Property chains: `a:b:c:1` → `{a:{b:{c:1}}}`
- Multi-line strings with backticks and triple quotes
- Plugin architecture for extensions
- Advanced implicit structure handling

## [∞](#technical-details) Technical Details

- **Vexy JSON Tool**: Uses WebAssembly (WASM) to run the same Rust parser that powers the command-line tool
- **Vexy JSON Tool**: Uses the official Jsonic library loaded via CDN
- **Privacy**: All parsing happens locally in your browser - no data sent to servers
- **Performance**: Both tools are optimized for excellent performance on all devices

Both tools maintain identical DaisyUI styling and responsive design for a consistent user experience.