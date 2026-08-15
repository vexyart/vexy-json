# this_file: TODO.md

- [ ] Complete the pattern-based error recovery system (currently stubbed)
- [ ] Implement the ML-based pattern recognition
- [ ] Finish the streaming parser implementation: release consumed input instead of retaining all of `input_buffer`; replace front-removal from `Vec` and unbounded queued events with a constant-memory design
- [ ] Optimize memory pool usage
- [ ] Remove dead code to reduce binary size
- [ ] Optimize hot paths identified by warnings
- [ ] Implement SIMD optimizations where applicable
- [ ] Add integration tests for all language bindings
- [ ] Create property-based tests for edge cases
- [ ] Set up continuous fuzzing
- [ ] Design and implement a plugin architecture
- [ ] Create example plugins
- [ ] Document plugin development
- [ ] Incremental parsing for live editing
- [ ] Schema validation integration
- [ ] Advanced error recovery strategies
- [ ] JSON path query support
