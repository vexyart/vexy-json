# Vexy JSON Plugin Directory

This directory contains documentation and examples for Vexy JSON plugins.

## Directory Structure

```
docs/plugins/
├── README.md                 # This file
├── builtin/                  # Built-in plugin documentation
│   ├── schema-validation.md
│   ├── datetime.md
│   ├── custom-numbers.md
│   └── comments.md
├── community/                # Community contributed plugins
│   └── README.md
└── examples/                 # Plugin examples
    ├── simple-transform.md
    ├── validation-rules.md
    └── macro-expansion.md
```

## Getting Started

1. Read the [Plugin Development Guide](../plugin-development.md)
2. Check out the [Plugin Registry](../plugin-registry.md)
3. Look at examples in the `examples/` directory
4. Study built-in plugins in the `builtin/` directory

## Contributing

To contribute a plugin:

1. Create documentation in `community/your-plugin.md`
2. Include code examples and usage instructions
3. Add tests and benchmarks
4. Submit a pull request

## Plugin Categories

### Built-in Plugins
- **Schema Validation**: Validate JSON against schemas
- **DateTime**: Parse and transform date/time values
- **Custom Numbers**: Support for hex, binary, and other formats
- **Comments**: Preserve comments during parsing

### Community Plugins
- Submit your plugins here!

## Standards

All plugins should follow these standards:

1. **Documentation**: Clear README with usage examples
2. **Testing**: Comprehensive test suite
3. **Error Handling**: Proper error messages and recovery
4. **Performance**: Benchmarks and optimization
5. **API Stability**: Semantic versioning for breaking changes