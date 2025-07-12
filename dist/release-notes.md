# Vexy JSON v2.0.0 - Major Performance & Architecture Release

🚀 This release represents a major architectural and performance milestone for VEXY_JSON, featuring comprehensive improvements in parsing speed, memory efficiency, and extensibility.

## ✅ Major Features

### ⚡ Performance & Optimization
- **SIMD-Accelerated Parsing** - 2-3x performance improvement for large files
- **Memory Pool V3** - 80% reduction in allocations with typed arenas
- **Parallel Processing** - Intelligent chunked processing for gigabyte-sized JSON files
- **Zero-copy** parsing paths for simple values

### 🏗️ Architecture & Extensibility
- **Streaming Parser V2** - Event-driven API for processing massive files
- **Plugin System** - Extensible architecture with ParserPlugin trait
- **Modular Architecture** - Clean separation with JsonLexer traits
- **AST Builder & Visitor** - Comprehensive AST manipulation capabilities

### 🛡️ Quality & Reliability
- **Error Recovery V2** - ML-based pattern recognition with actionable suggestions
- **Comprehensive Fuzzing** - 4 specialized targets with extensive coverage
- **Enhanced Error Messages** - Context-aware suggestions and recovery strategies
- **Type-Safe Error Handling** - Comprehensive error taxonomy with structured codes

## 📊 Performance Improvements

- **2-3x faster** string scanning with SIMD optimization
- **80% reduction** in allocations for typical workloads
- **Parallel processing** for files > 1MB with intelligent boundary detection
- **String interning** for common JSON keys
- **Streaming capability** for minimal memory usage on large files

## 🔄 Migration from v1.x

- Core parsing API remains compatible
- New streaming and parallel APIs are additive
- Plugin system is entirely new (opt-in)
- Performance improvements are automatic
- Error types have been restructured (but improved)

## 📦 Installation

```bash
cargo install vexy-json --version 2.0.0
```

Or download pre-built binaries from the assets below.

---

**Full Changelog**: https://github.com/vexyart/vexy-json/compare/v1.5.27...v2.0.0
