# this_file: PLAN.md

# Vexy JSON improvement plan

## Parser and performance

- **Bound streaming memory.** Release consumed input and replace front removal from token and event queues. Verify that retained input and queue sizes stay bounded on a long stream fed in small chunks.
- **Remove dead code to reduce binary size.** Identify unused code and compare stripped release binaries before and after removal.
- **Optimize hot paths.** Profile representative strict and forgiving JSON inputs. Compare the same benchmarks before and after changes to measured hot paths.
- **Improve advanced error recovery.** Verify that nested malformed input produces the expected suggestions and that applying a suggested repair yields the intended value. Current recovery tests allow some cases to fail without asserting a repair.

## Tests and extensions

- **Complete binding integration coverage.** The Python, WebAssembly and C smoke checks pass. The C++ example uses outdated C type names. Check that it builds and runs against the C library, and verify a Serde round trip through its public API.
- **Verify continuous fuzzing.** The scheduled workflow is disabled for inactivity. Verify target execution and failure reporting after reactivation. The PR workflow names `json_strings`, while the registered target is `strings`.
- **Document plugin development.** Show how to implement and register a `ParserPlugin` and invoke its hooks. Check the walkthrough against a runnable example.
- **Integrate schema validation with parsing.** The schema plugin can validate an already parsed value. Verify that the intended parsing entry point rejects an input that violates a configured schema.
- **Support incremental parsing for live editing.** Verify that a small edit updates the affected structure and yields the same result as a fresh parse.
- **Support JSON path queries.** Verify queries against nested arrays and objects, including queries with no matches.
