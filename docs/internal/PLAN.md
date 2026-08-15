# this_file: docs/internal/PLAN.md

# Vexy JSON Development Plan - Future Enhancements

## Current status

### Completed emergency repair

`StreamingParser` no longer returns `EndOfInput` forever. EOF is emitted once,
iteration terminates, and regression drains are bounded. The repair also covers
newline handling, empty-array closure, and malformed test fixtures. `build.sh`
now reports `INT`, `TERM`, and failures truthfully; its workflow contracts are
aligned with the verified build path. Stale Python wheel tests and documentation
now use the established forgiving-parser semantics.

Verification completed: workflow contracts passed; `./build.sh` passed the Rust
workspace and packaged Python suite (36 passed, 2 skipped). The final
`./build.sh` run exited 0 in 14.22 s with 429,047,808-byte maximum RSS and
0 swaps.

Known remaining warnings: unstable rustfmt configuration noise and unused
`extract_string_content`.

### Historical v1.5.6 status snapshot

The prior record described the v1.5.6 release as having 200/200 tests passing,
all clippy errors fixed, automated version management, and C API safety
documentation. It is retained as release history; the current verification and
known warnings are recorded above.

## Future Development Roadmap (v1.6+)

### Architecture Improvements

- [ ] Complete the pattern-based error recovery system (currently stubbed)
- [ ] Implement the ML-based pattern recognition
- [ ] Finish the streaming parser implementation
  - The completed EOF repair is not a constant-memory streaming implementation.
  - Release consumed input instead of retaining all input in `input_buffer`.
  - Replace front-removal from `Vec` and queued events with data structures and
    ownership that scale without retaining the whole input or event history.
- [ ] Optimize memory pool usage

### Performance Enhancements

- [ ] Remove dead code to reduce binary size
- [ ] Optimize hot paths identified by warnings
- [ ] Implement SIMD optimizations where applicable

### Testing Infrastructure

- [ ] Add integration tests for all language bindings
- [ ] Create property-based tests for edge cases
- [ ] Set up continuous fuzzing

### Plugin System

- [ ] Design and implement a plugin architecture
- [ ] Create example plugins
- [ ] Document plugin development

### Advanced Features

- [ ] Incremental parsing for live editing
- [ ] Schema validation integration
- [ ] Advanced error recovery strategies
- [ ] JSON path query support

## Implementation Strategy

These features represent substantial architectural improvements and new capabilities that would be implemented in future major releases (v1.6+). Each requires significant design work and implementation time, making them roadmap items rather than immediate fixes.

## Success Metrics

- [ ] Fuzzing works or is properly disabled (currently deferred)
- [x] Build completes without errors
- [x] All tests pass
- [x] Release script validates test success
- [x] Ready for clean releases
