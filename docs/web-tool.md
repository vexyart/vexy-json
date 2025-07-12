---
layout: default
title: Web Tools
parent: Usage Guide
nav_order: 3
---

# Interactive Web Tools

The vexy_json project provides two interactive web tools for parsing JSON with different libraries, both featuring identical modern interfaces and seamless Jekyll integration.

## Available Tools

### [Vexy JSON Parser]({{ '/vexy_json-tool/' | relative_url }})
- **Library**: vexy_json (Rust compiled to WebAssembly)
- **URL**: [https://twardoch.github.io/vexy_json/vexy_json-tool/](https://twardoch.github.io/vexy_json/vexy_json-tool/)
- **Features**: WebAssembly-powered parsing with all vexy_json forgiving features

### [Jsonic Parser]({{ '/vexy-json-tool/' | relative_url }})
- **Library**: jsonic (JavaScript via CDN)
- **URL**: [https://twardoch.github.io/vexy_json/vexy-json-tool/](https://twardoch.github.io/vexy_json/vexy-json-tool/)
- **Features**: Flexible JSON with object merging and property chains

### [Tools Overview]({{ '/tools/' | relative_url }})
Complete comparison and access to both tools.

## Common Features

Both tools share identical modern interfaces with:

- **Real-time parsing** - See results as you type
- **Syntax highlighting** - Clear visualization of your JSON
- **Error highlighting** - Precise error messages with position indicators
- **Parser options** - Toggle individual features on/off
- **Example templates** - Pre-loaded examples to get started
- **Share URLs** - Share your JSON snippets with others
- **Download results** - Save parsed JSON to a file
- **Dark/light themes** - Choose your preferred color scheme
- **Mobile responsive** - Works on all devices
- **Jekyll integration** - Seamless navigation within documentation

## Privacy & Performance

- **Privacy**: All parsing happens locally in your browser - no data sent to servers
- **Performance**: Optimized for excellent performance on all devices
- **Caching**: Efficient loading with modern web technologies

## Usage

1. Choose your preferred tool from the [tools overview]({{ '/tools/' | relative_url }})
2. Enter JSON or forgiving JSON in the input panel
3. Adjust parser options as needed
4. View the parsed result and any errors instantly
5. Use share/download features as needed

## Technical Details

- **Vexy JSON Tool**: Uses WebAssembly (WASM) to run the same Rust parser that powers the CLI
- **Vexy JSON Tool**: Uses the official Jsonic library loaded via CDN
- **Design**: Both tools use identical DaisyUI + Tailwind CSS styling
- **Integration**: Full Jekyll integration with site navigation