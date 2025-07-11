---
layout: default
title: Benchmark Results
nav_order: 6
---

# Benchmark Results

This section presents the parsing performance benchmarks for `vexy_json` (Rust).
Benchmarks were run on the following environment:

*   **CPU**: [e.g., Intel Core i7-10700K]
*   **RAM**: [e.g., 32GB DDR4]
*   **OS**: [e.g., macOS 14.5 Sonoma]
*   **Rust Toolchain**: [e.g., `rustc 1.79.0 (129f3b996 2024-06-10)`]

Lower values (nanoseconds per iteration) are better.

| Test Case | `vexy_json` (ns/iter) |
|---|---|
| simple | 7782 |
| array | 7836 |
| nested | 41319 |
| large_array | 299726294 |
| deeply_nested | 3370 |
| forgiving | 15867 |
| config | 142978 |

**Note:** `ns/iter` means nanoseconds per iteration. The results above are examples and may vary depending on your hardware and software environment.

## How to Run Benchmarks

Benchmarks are implemented using `criterion.rs`. You can run them locally using the following command:

```bash
cargo bench
```

The benchmark definitions are located in the `benches/` directory, with data files in `benches/data/`.
